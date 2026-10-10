#include "logicfile.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QFileSystemWatcher>
#include <QQmlComponent>
#include <QQmlContext>
#include <QQmlEngine>
#include <QTimer>

namespace qq {

namespace {

/// The bytes at a URL: a file on this computer or one bundled in qrc:.
bool readUrl(const QUrl &url, QByteArray &out)
{
    QFile f(url.scheme() == QStringLiteral("qrc") ? QLatin1Char(':') + url.path() : url.toLocalFile());
    if ((!url.isLocalFile() && url.scheme() != QStringLiteral("qrc")) || !f.open(QIODevice::ReadOnly))
        return false;
    out = f.readAll();
    return true;
}

}  // namespace

LogicFile::LogicFile(QObject *parent) : QObject(parent)
{
    m_settle = new QTimer(this);
    m_settle->setSingleShot(true);
    // An editor's save can arrive as several writes, or as a new file renamed over the old one: wait for it to finish.
    m_settle->setInterval(120);
    connect(m_settle, &QTimer::timeout, this, &LogicFile::reload);
}

LogicFile::~LogicFile()
{
    dropObject();
}

void LogicFile::setSource(const QUrl &source)
{
    if (source == m_source)
        return;
    m_source = source;
    emit sourceChanged();
    restart();
}

void LogicFile::setActive(bool active)
{
    if (active == m_active)
        return;
    m_active = active;
    emit activeChanged();
    restart();
}

void LogicFile::setProperties(const QVariantMap &properties)
{
    if (properties == m_properties)
        return;
    m_properties = properties;
    emit propertiesChanged();
    if (m_object) {
        for (auto it = m_properties.cbegin(); it != m_properties.cend(); ++it)
            m_object->setProperty(it.key().toUtf8(), it.value());
    }
}

void LogicFile::restart()
{
    delete m_watcher;
    m_watcher = nullptr;
    m_settle->stop();
    m_attempted.clear();
    const bool had = m_object;
    dropObject();
    m_builds = 0;
    if (had)
        emit objectChanged();
    setError(QString());
    if (!m_active || m_source.isEmpty())
        return;
    watch();
    reload();
}

void LogicFile::watch()
{
    if (!m_source.isLocalFile())
        return;
    m_watcher = new QFileSystemWatcher(this);
    const QString path = m_source.toLocalFile();
    m_watcher->addPath(QFileInfo(path).absolutePath());
    if (QFileInfo::exists(path))
        m_watcher->addPath(path);
    auto changed = [this, path] {
        // A file replaced by a rename is no longer watched; watch the new one.
        if (m_watcher && !m_watcher->files().contains(path) && QFileInfo::exists(path))
            m_watcher->addPath(path);
        m_settle->start();
    };
    connect(m_watcher, &QFileSystemWatcher::fileChanged, this, changed);
    connect(m_watcher, &QFileSystemWatcher::directoryChanged, this, changed);
}

void LogicFile::reload()
{
    if (!m_active || m_source.isEmpty())
        return;
    QByteArray bytes;
    if (!readUrl(m_source, bytes)) {
        setError(QStringLiteral("%1 could not be read.").arg(m_source.fileName()));
        return;
    }
    if (bytes == m_attempted)
        return;
    m_attempted = bytes;

    QQmlEngine *engine = qmlEngine(this);
    QQmlContext *context = qmlContext(this);
    if (!engine || !context) {
        setError(QStringLiteral("Logic is built inside a QQ scene."));
        return;
    }
    // From the bytes, every time: the engine's cache would hand back the logic as it was when first read.
    QQmlComponent component(engine);
    component.setData(bytes, m_source);
    if (component.isError()) {
        QStringList why;
        for (const QQmlError &e : component.errors())
            why << e.toString();
        setError(why.join(QLatin1Char('\n')));
        return;
    }
    QObject *made = component.beginCreate(context);
    if (!made) {
        setError(component.errorString());
        return;
    }
    const bool isLogic = made->property("qqLogic").toBool();
    made->setParent(this);
    QQmlEngine::setObjectOwnership(made, QQmlEngine::CppOwnership);
    if (isLogic) {
        for (auto it = m_properties.cbegin(); it != m_properties.cend(); ++it)
            made->setProperty(it.key().toUtf8(), it.value());
    }
    component.completeCreate();
    if (!isLogic) {
        delete made;
        setError(QStringLiteral("%1 is not QQ logic: its root is not a Logic.").arg(m_source.fileName()));
        return;
    }

    dropObject();
    m_object = made;
    ++m_builds;
    setError(QString());
    emit objectChanged();
}

void LogicFile::setError(const QString &error)
{
    if (error == m_error)
        return;
    m_error = error;
    emit errorChanged();
}

void LogicFile::dropObject()
{
    if (m_object)
        m_object->deleteLater();
    m_object = nullptr;
}

}  // namespace qq
