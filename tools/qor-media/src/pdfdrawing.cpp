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

namespace qormedia {

namespace {

// One PDF opened for drawing. Its document is used only under its lock, from whichever thread draws. It reads through
// a file opened here, since QPdfDocument::close does not let go of a file it opened itself; closing this one does.
struct Drawn
{
    QMutex lock;
    QFile file;
    QPdfDocument document;
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

QImage drawPage(Drawn &drawn, int page, int width)
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
    QPdfDocumentRenderOptions options;
    options.setRenderFlags(QPdfDocumentRenderOptions::RenderFlag::Annotations);
    return drawn.document.render(page, size, options);
}

// A page drawn on the image provider's own threads, so scrolling never waits for it.
class PageResponse : public QQuickImageResponse, public QRunnable
{
public:
    PageResponse(std::shared_ptr<Drawn> drawn, int page, int width) : m_drawn(std::move(drawn)), m_page(page), m_width(width)
    {
        setAutoDelete(false);
    }

    void run() override
    {
        if (m_drawn)
            m_image = drawPage(*m_drawn, m_page, m_width);
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
    QImage m_image;
    QString m_error;
};

class PageProvider : public QQuickAsyncImageProvider
{
public:
    QQuickImageResponse *requestImageResponse(const QString &id, const QSize &requestedSize) override
    {
        // `<key>/<page>`
        const qsizetype slash = id.lastIndexOf(QLatin1Char('/'));
        bool pageOk = false;
        const int page = slash > 0 ? id.mid(slash + 1).toInt(&pageOk) : -1;
        auto *response = new PageResponse(pageOk ? find(id.left(slash)) : nullptr, page, requestedSize.width());
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

QImage PdfDrawing::draw(const QString &key, int page, int width)
{
    const std::shared_ptr<Drawn> drawn = find(key);
    return drawn ? drawPage(*drawn, page, width) : QImage();
}

}  // namespace qormedia
