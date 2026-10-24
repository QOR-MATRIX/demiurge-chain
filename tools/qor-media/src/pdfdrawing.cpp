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

#include <fpdf_annot.h>

#include "pdfium.h"

namespace qormedia {

namespace {

// One PDF opened for drawing. Its document is used only under its lock, from whichever thread draws. It reads through
// a file opened here, since QPdfDocument::close does not let go of a file it opened itself; closing this one does.
struct Drawn
{
    QMutex lock;
    QFile file;
    QPdfDocument document;
    // Kept, like the document's own, while it is open: drawing it without some of its marks reads it again.
    QByteArray password;
};

// Every PDF open for drawing, by key. A Drawn goes on the thread its document belongs to, the application's.
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

// The page through QOR Media's own PDFium, with the marks at `hidden` (places among its annotations) hidden for this
// drawing only: nothing is written. Read through the file the document reads, under the same lock.
QImage drawWithout(Drawn &drawn, int page, const QSize &size, const QList<int> &hidden)
{
    PdfiumLock locked;
    PdfSource source(&drawn.file);
    if (!source.open())
        return {};
    PdfiumDocument document{FPDF_LoadCustomDocument(&source.access, drawn.password.isEmpty() ? nullptr : drawn.password.constData())};
    PdfiumPage drawing{document.handle ? FPDF_LoadPage(document.handle, page) : nullptr};
    if (!drawing.handle)
        return {};
    for (const int index : hidden)
        if (FPDF_ANNOTATION annot = FPDFPage_GetAnnot(drawing.handle, index)) {
            FPDFAnnot_SetFlags(annot, FPDFAnnot_GetFlags(annot) | FPDF_ANNOT_FLAG_HIDDEN);
            FPDFPage_CloseAnnot(annot);
        }
    QImage image(size, QImage::Format_ARGB32);
    image.fill(Qt::white);
    FPDF_BITMAP bitmap = FPDFBitmap_CreateEx(size.width(), size.height(), FPDFBitmap_BGRA, image.bits(), int(image.bytesPerLine()));
    if (!bitmap)
        return {};
    FPDF_RenderPageBitmap(bitmap, drawing.handle, 0, 0, size.width(), size.height(), 0, FPDF_ANNOT);
    FPDFBitmap_Destroy(bitmap);
    return image;
}

QImage drawPage(Drawn &drawn, int page, int width, const QList<int> &hidden)
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
    if (!hidden.isEmpty())
        return drawWithout(drawn, page, size, hidden);
    QPdfDocumentRenderOptions options;
    options.setRenderFlags(QPdfDocumentRenderOptions::RenderFlag::Annotations);
    return drawn.document.render(page, size, options);
}

// A page drawn on the image provider's own threads, so scrolling never waits for it.
class PageResponse : public QQuickImageResponse, public QRunnable
{
public:
    PageResponse(std::shared_ptr<Drawn> drawn, int page, int width, QList<int> hidden)
        : m_drawn(std::move(drawn)), m_page(page), m_width(width), m_hidden(std::move(hidden))
    {
        setAutoDelete(false);
    }

    void run() override
    {
        if (m_drawn)
            m_image = drawPage(*m_drawn, m_page, m_width, m_hidden);
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
    QImage m_image;
    QString m_error;
};

class PageProvider : public QQuickAsyncImageProvider
{
public:
    QQuickImageResponse *requestImageResponse(const QString &id, const QSize &requestedSize) override
    {
        // `<key>/<page>`, or `<key>/<page>/<hidden>`: the places of the marks to leave out, by commas.
        const QStringList parts = id.split(QLatin1Char('/'));
        bool pageOk = false;
        const int page = parts.size() == 2 || parts.size() == 3 ? parts[1].toInt(&pageOk) : -1;
        QList<int> hidden;
        for (const QString &place : parts.size() == 3 ? parts[2].split(QLatin1Char(','), Qt::SkipEmptyParts) : QStringList()) {
            bool ok = false;
            if (const int index = place.toInt(&ok); ok && index >= 0)
                hidden << index;
        }
        auto *response = new PageResponse(pageOk ? find(parts[0]) : nullptr, page, requestedSize.width(), hidden);
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
    drawn->document.setPassword(password);
    drawn->password = password.toUtf8();
    drawn->document.load(&drawn->file);
    if (drawn->document.status() != QPdfDocument::Status::Ready)
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
    drawn->document.close();
    drawn->file.close();
}

int PdfDrawing::openCount()
{
    QMutexLocker locked(&registryLock);
    return int(registry().size());
}

QImage PdfDrawing::draw(const QString &key, int page, int width, const QList<int> &hidden)
{
    const std::shared_ptr<Drawn> drawn = find(key);
    return drawn ? drawPage(*drawn, page, width, hidden) : QImage();
}

}  // namespace qormedia
