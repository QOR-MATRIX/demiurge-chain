#include "pdfdrawing.h"

#include <QCoreApplication>
#include <QFile>
#include <QHash>
#include <QMutex>
#include <QPdfDocument>
#include <QPdfDocumentRenderOptions>
#include <QQmlEngine>
#include <QQuickAsyncImageProvider>
#include <QQuickTextureFactory>
#include <QRunnable>
#include <QThread>
#include <QThreadPool>
#include <QUuid>

#include <memory>
#include <utility>

#include <fpdf_annot.h>

#include "pdfforms.h"
#include "pdfium.h"
#include "qmlvalues.h"

namespace qormedia {

namespace {

// One PDF opened for drawing, read through a file opened here, since QPdfDocument::close does not let go of a file it
// opened itself; closing this one does. A PDF without a form is drawn by Qt PDF, whose PDFium is its own, so drawing it
// never waits on QOR Media's, which reads marks and fields and saves on the application's thread. One with a form is
// drawn by QOR Media's PDFium through that form, as nothing in Qt PDF draws a field. All of it is used only under its
// lock, and QOR Media's PDFium under PdfiumLock too, from whichever thread draws; release() lets go of all of it.
struct Drawn
{
    QMutex lock;
    QFile file;
    QPdfDocument document;
    // Kept, like the document's own, while it is open: drawing it with marks hidden or fields filled reads it again.
    QByteArray password;
    // QOR Media's PDFium reading the file, and the form its fields are drawn through, when it has one.
    std::unique_ptr<PdfSource> source;
    PdfiumDocument withForm;
    std::unique_ptr<PdfiumForm> form;
    // What the reader has filled in, as plain data, for the drawings that show it.
    QVariantList filled;

    ~Drawn() { release(); }
    // The form goes before its document, and both before the file they read.
    void release()
    {
        if (form || withForm.handle) {
            PdfiumLock locked;
            form.reset();
            withForm.close();
        }
        source.reset();
        document.close();
        file.close();
    }
};

// Every PDF open for drawing, by key. A Drawn goes on the application's thread, where its file was opened.
QMutex registryLock;
QHash<QString, std::shared_ptr<Drawn>> &registry()
{
    static QHash<QString, std::shared_ptr<Drawn>> drawn;
    return drawn;
}

std::shared_ptr<Drawn> find(const QString &key)
{
    QMutexLocker locked(&registryLock);
    return registry().value(key);
}

std::shared_ptr<Drawn> makeDrawn()
{
    return std::shared_ptr<Drawn>(new Drawn, [](Drawn *drawn) {
        QCoreApplication *app = QCoreApplication::instance();
        if (app && QThread::currentThread() != app->thread())
            QMetaObject::invokeMethod(app, [drawn] { delete drawn; }, Qt::QueuedConnection);
        else
            delete drawn;
    });
}

// Opened by Qt PDF; and by QOR Media's PDFium too, with its form, when it has one. A form PDFium cannot read is drawn
// by Qt PDF without its fields rather than not at all.
bool load(Drawn &drawn, const QString &password)
{
    drawn.document.setPassword(password);
    drawn.document.load(&drawn.file);
    if (drawn.document.status() != QPdfDocument::Status::Ready)
        return false;
    PdfiumLock locked;
    auto source = std::make_unique<PdfSource>(&drawn.file);
    if (!source->open())
        return true;
    PdfiumDocument document{FPDF_LoadCustomDocument(&source->access, drawn.password.isEmpty() ? nullptr : drawn.password.constData())};
    if (!document.handle || FPDF_GetFormType(document.handle) == FORMTYPE_NONE)
        return true;
    drawn.source = std::move(source);
    drawn.withForm.handle = std::exchange(document.handle, nullptr);
    drawn.form = std::make_unique<PdfiumForm>(drawn.withForm.handle);
    return true;
}

// The page with its marks, then its form's fields over them, on white, as the reader's paper is.
QImage render(FPDF_PAGE page, FPDF_FORMHANDLE form, const QSize &size)
{
    QImage image(size, QImage::Format_ARGB32);
    image.fill(Qt::white);
    FPDF_BITMAP bitmap = FPDFBitmap_CreateEx(size.width(), size.height(), FPDFBitmap_BGRA, image.bits(), int(image.bytesPerLine()));
    if (!bitmap)
        return {};
    FPDF_RenderPageBitmap(bitmap, page, 0, 0, size.width(), size.height(), 0, FPDF_ANNOT);
    if (form)
        FPDF_FFLDraw(form, bitmap, page, 0, 0, size.width(), size.height(), 0, FPDF_ANNOT);
    FPDFBitmap_Destroy(bitmap);
    return image;
}

// The page drawn by QOR Media's PDFium from a copy read for this drawing alone, with the marks at `hidden` (places
// among its annotations) hidden and the fields filled as `filled` says. Nothing is written. Under both locks.
QImage drawChanged(Drawn &drawn, int page, const QSize &size, const QList<int> &hidden, const QVariantList &filled)
{
    PdfSource source(&drawn.file);
    if (!source.open())
        return {};
    PdfiumDocument document{FPDF_LoadCustomDocument(&source.access, drawn.password.isEmpty() ? nullptr : drawn.password.constData())};
    if (!document.handle)
        return {};
    PdfiumForm form(document.handle);
    // A filling this cannot follow leaves the fields as the file has them; saving it says why.
    QString why;
    fillFields(document.handle, form.handle(), filled, &why);
    PdfiumPage drawing{FPDF_LoadPage(document.handle, page), form.handle()};
    if (!drawing.handle)
        return {};
    for (const int index : hidden)
        if (PdfiumAnnot annot{FPDFPage_GetAnnot(drawing.handle, index)}; annot.handle)
            FPDFAnnot_SetFlags(annot.handle, FPDFAnnot_GetFlags(annot.handle) | FPDF_ANNOT_FLAG_HIDDEN);
    return render(drawing.handle, form.handle(), size);
}

QImage drawPage(Drawn &drawn, int page, int width, const QList<int> &hidden, bool filled)
{
    QMutexLocker locked(&drawn.lock);
    if (drawn.document.status() != QPdfDocument::Status::Ready || page < 0 || page >= drawn.document.pageCount())
        return {};
    const QSizeF points = drawn.document.pagePointSize(page);
    if (points.width() <= 0 || points.height() <= 0)
        return {};
    if (width <= 0)
        width = qRound(points.width() * 96 / 72);
    width = qMin(width, PdfDrawing::WidthLimit);
    const QSize size(width, qMax(1, qRound(width * points.height() / points.width())));
    if (!hidden.isEmpty() || (filled && !drawn.filled.isEmpty())) {
        PdfiumLock pdfium;
        return drawChanged(drawn, page, size, hidden, filled ? drawn.filled : QVariantList());
    }
    if (drawn.form) {
        PdfiumLock pdfium;
        PdfiumPage drawing{FPDF_LoadPage(drawn.withForm.handle, page), drawn.form->handle()};
        return drawing.handle ? render(drawing.handle, drawn.form->handle(), size) : QImage();
    }
    QPdfDocumentRenderOptions options;
    options.setRenderFlags(QPdfDocumentRenderOptions::RenderFlag::Annotations);
    return drawn.document.render(page, size, options);
}

// A page drawn on the image provider's own threads, so scrolling never waits for it.
class PageResponse : public QQuickImageResponse, public QRunnable
{
public:
    PageResponse(std::shared_ptr<Drawn> drawn, int page, int width, QList<int> hidden, bool filled)
        : m_drawn(std::move(drawn)), m_page(page), m_width(width), m_hidden(std::move(hidden)), m_filled(filled)
    {
        setAutoDelete(false);
    }

    void run() override
    {
        if (m_drawn)
            m_image = drawPage(*m_drawn, m_page, m_width, m_hidden, m_filled);
        m_drawn.reset();
        if (m_image.isNull())
            m_error = QStringLiteral("This page could not be drawn.");
        emit finished();
    }

    QQuickTextureFactory *textureFactory() const override { return QQuickTextureFactory::textureFactoryForImage(m_image); }
    QString errorString() const override { return m_error; }

private:
    std::shared_ptr<Drawn> m_drawn;
    int m_page;
    int m_width;
    QList<int> m_hidden;
    bool m_filled;
    QImage m_image;
    QString m_error;
};

class PageProvider : public QQuickAsyncImageProvider
{
public:
    QQuickImageResponse *requestImageResponse(const QString &id, const QSize &requestedSize) override
    {
        // `<key>/<page>`; `<key>/<page>/<hidden>`, the places of the marks to leave out, by commas; or
        // `<key>/<page>/<hidden>/<filled>`, any word naming a filling, for the fields as the reader last filled them.
        const QStringList parts = id.split(QLatin1Char('/'));
        bool pageOk = false;
        const int page = parts.size() >= 2 && parts.size() <= 4 ? parts[1].toInt(&pageOk) : -1;
        QList<int> hidden;
        for (const QString &place : parts.size() >= 3 ? parts[2].split(QLatin1Char(','), Qt::SkipEmptyParts) : QStringList()) {
            bool ok = false;
            if (const int index = place.toInt(&ok); ok && index >= 0)
                hidden << index;
        }
        const bool filled = parts.size() == 4 && !parts[3].isEmpty();
        auto *response = new PageResponse(pageOk ? find(parts[0]) : nullptr, page, requestedSize.width(), hidden, filled);
        m_pool.start(response);
        return response;
    }

private:
    QThreadPool m_pool;
};

}  // namespace

PdfDrawing::~PdfDrawing()
{
    for (const QString &key : std::as_const(m_opened))
        close(key);
}

QString PdfDrawing::open(const QString &path, const QString &password)
{
    if (QQmlEngine *engine = qmlEngine(this); engine && !engine->imageProvider(QLatin1String(Provider)))
        engine->addImageProvider(QLatin1String(Provider), new PageProvider);
    std::shared_ptr<Drawn> drawn = makeDrawn();
    drawn->file.setFileName(path);
    if (!drawn->file.open(QIODevice::ReadOnly))
        return {};
    drawn->password = password.toUtf8();
    if (!load(*drawn, password))
        return {};
    const QString key = QUuid::createUuid().toString(QUuid::Id128);
    {
        QMutexLocker locked(&registryLock);
        registry().insert(key, drawn);
    }
    m_opened.insert(key);
    return key;
}

void PdfDrawing::close(const QString &key)
{
    std::shared_ptr<Drawn> drawn;
    {
        QMutexLocker locked(&registryLock);
        drawn = registry().take(key);
    }
    m_opened.remove(key);
    if (!drawn)
        return;
    // The file is let go of now, even while a page of it is still being drawn elsewhere.
    QMutexLocker locked(&drawn->lock);
    drawn->release();
}

void PdfDrawing::fill(const QString &key, const QVariantList &filled)
{
    const std::shared_ptr<Drawn> drawn = find(key);
    if (!drawn)
        return;
    // Plain data now, on the engine's thread: the pages are drawn on others.
    const QVariantList plain = values::plainAll(QVariant(filled)).toList();
    QMutexLocker locked(&drawn->lock);
    drawn->filled = plain;
}

int PdfDrawing::openCount()
{
    QMutexLocker locked(&registryLock);
    return int(registry().size());
}

QImage PdfDrawing::draw(const QString &key, int page, int width, const QList<int> &hidden, bool filled)
{
    const std::shared_ptr<Drawn> drawn = find(key);
    return drawn ? drawPage(*drawn, page, width, hidden, filled) : QImage();
}

}  // namespace qormedia
