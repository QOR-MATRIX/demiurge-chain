#include "confinement.h"

#include <QDesktopServices>
#include <QDir>
#include <QFile>
#include <QRegularExpression>

#include <algorithm>
#include <QFileInfo>
#include <QMutex>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QQmlAbstractUrlInterceptor>
#include <QQmlEngine>
#include <QQmlNetworkAccessManagerFactory>

#include "sceneio.h"

namespace qq {

namespace {

QMutex refusedLock;
QStringList refused;

void record(const QString &kind, const QString &what)
{
    {
        QMutexLocker lock(&refusedLock);
        refused << kind + QStringLiteral(": ") + what;
    }
    qWarning("QQ refused a game's %s: %s", qPrintable(kind), qPrintable(what));
}

/// A location to compare: "qrc:/path" for the executable's resources, a clean local path for files, "" otherwise.
QString location(const QUrl &url)
{
    if (url.scheme() == QStringLiteral("qrc"))
        return QStringLiteral("qrc:") + QDir::cleanPath(url.path());
    if (url.isLocalFile())
        return QDir::cleanPath(url.toLocalFile());
    return {};
}

QString rootOf(const QString &importPath)
{
    if (importPath.startsWith(QStringLiteral("qrc:")))
        return QStringLiteral("qrc:") + QDir::cleanPath(importPath.mid(4));
    if (importPath.startsWith(QStringLiteral(":/")))
        return QStringLiteral("qrc:") + QDir::cleanPath(importPath.mid(1));
    return QDir::cleanPath(QDir::fromNativeSeparators(importPath));
}

bool inside(const QString &where, const QString &root)
{
    return !root.isEmpty()
           && (where.compare(root, Qt::CaseInsensitive) == 0
               || where.startsWith(root + QLatin1Char('/'), Qt::CaseInsensitive));
}

/// A URL that names nothing: what a refused load is pointed at, so it fails as a missing file does.
QUrl nowhere(const char *what)
{
    return QUrl(QStringLiteral("qrc:/qq-refused/%1").arg(QLatin1String(what)));
}

/// These are allowed themselves, but not every module under them (QtQuick.LocalStorage, QtQuick.Dialogs, QtQuick3D.Xr).
bool exactOnly(const QString &module)
{
    return module == QStringLiteral("QML") || module == QStringLiteral("QtQuick") || module == QStringLiteral("QtQuick3D");
}

bool allowedModule(const QString &module)
{
    for (const QString &allowed : Confinement::allowedModules()) {
        if (module.compare(allowed, Qt::CaseInsensitive) == 0)
            return true;
        if (!exactOnly(allowed) && module.startsWith(allowed + QLatin1Char('/'), Qt::CaseInsensitive))
            return true;
    }
    return false;
}

class Interceptor : public QQmlAbstractUrlInterceptor
{
public:
    Interceptor(QString package, QStringList roots) : m_package(std::move(package)), m_roots(std::move(roots)) {}

    QUrl intercept(const QUrl &url, DataType type) override
    {
        if (url.isEmpty() || url.scheme() == QStringLiteral("data"))
            return url;  // nothing, or whole in itself: it reaches nothing
        // Qt 6 hands a url property over as written: a relative one is let through here, and is seen again, whole, when
        // it is loaded (tst_lockdown climbs out of the package with one, to show it is).
        if (url.isRelative())
            return url;
        const QString where = location(url);
        if (where.isEmpty()) {
            record(QStringLiteral("load"), url.toString());
            return nowhere("load");
        }
        if (inside(where, m_package))
            return url;
        // The innermost import folder holding it: an application's folder can hold its own qml/ folder of modules.
        for (const QString &root : m_roots) {
            if (!inside(where, root))
                continue;
            if (type != QmldirFile)
                return url;  // a module's own files
            QString module = where.mid(root.size() + 1);
            module.chop(QStringLiteral("/qmldir").size());
            // Qt looks for versioned folders first ("QtQml.6.12/", "QtQml.6/"): the module is the same.
            static const QRegularExpression version(QStringLiteral(R"(\.\d+(\.\d+)?(?=/|$))"));
            module.remove(version);
            if (allowedModule(module))
                return url;
            // Qt asks about places a module might be; only one that is there is worth saying was refused.
            if (exists(where))
                record(QStringLiteral("import"), QString(module).replace(QLatin1Char('/'), QLatin1Char('.')));
            return nowhere("import");
        }
        if (url.scheme() == QStringLiteral("qrc") && type != QmldirFile)
            return url;  // the executable's other resources: harmless to read
        if (exists(where))
            record(QStringLiteral("load"), url.toString());
        return nowhere("load");
    }

    static bool exists(const QString &where)
    {
        return where.startsWith(QStringLiteral("qrc:")) ? QFile::exists(QLatin1Char(':') + where.mid(4))
                                                        : QFile::exists(where);
    }

private:
    QString m_package;
    QStringList m_roots;
};

/// Every request a game's engine makes goes here, and none goes out.
class RefusingAccess : public QNetworkAccessManager
{
public:
    using QNetworkAccessManager::QNetworkAccessManager;

protected:
    QNetworkReply *createRequest(Operation op, const QNetworkRequest &request, QIODevice *outgoing) override
    {
        const QUrl url = request.url();
        // Only the executable's own resources and data URLs are read through here; neither reaches anything.
        if (url.scheme() == QStringLiteral("qrc") || url.scheme() == QStringLiteral("data"))
            return QNetworkAccessManager::createRequest(op, request, outgoing);
        record(QStringLiteral("network request"), url.toString());
        return QNetworkAccessManager::createRequest(op, QNetworkRequest(nowhere("network")), nullptr);
    }
};

class RefusingAccessFactory : public QQmlNetworkAccessManagerFactory
{
public:
    QNetworkAccessManager *create(QObject *parent) override { return new RefusingAccess(parent); }
};

/// Qt.openUrlExternally lands here for every refused scheme.
class UrlRefuser : public QObject
{
    Q_OBJECT
public slots:
    void refuse(const QUrl &url) { record(QStringLiteral("request to open"), url.toString()); }
};

}  // namespace

const QStringList &Confinement::allowedModules()
{
    // Each module, and every module under it (QtQuick/Controls covers its styles), except QML, QtQuick and QtQuick3D,
    // which are allowed only themselves.
    static const QStringList modules{
        QStringLiteral("QML"),
        QStringLiteral("QtQml"),
        QStringLiteral("QtQuick"),
        QStringLiteral("QtQuick3D"),  // which imports QtQml.Models and QtQml.WorkerScript; a worker's requests are refused too
        QStringLiteral("QtQuick/Window"),
        QStringLiteral("QtQuick/Layouts"),
        QStringLiteral("QtQuick/Shapes"),
        QStringLiteral("QtQuick/Effects"),
        QStringLiteral("QtQuick/Particles"),
        QStringLiteral("QtQuick/Templates"),
        QStringLiteral("QtQuick/Controls"),
        QStringLiteral("QtQuick3D/Helpers"),
        QStringLiteral("QtQuick3D/Physics"),
        QStringLiteral("QtQuick3D/Particles3D"),
        QStringLiteral("QtQuick3D/SpatialAudio"),
        QStringLiteral("QtQuick3D/AssetUtils"),
        QStringLiteral("QtQuick3D/Effects"),
        QStringLiteral("QtQuick3D/ParticleEffects"),
        QStringLiteral("QQ"),
    };
    return modules;
}

const QStringList &Confinement::refusedSchemes()
{
    static const QStringList schemes{
        QStringLiteral("file"),    QStringLiteral("http"),         QStringLiteral("https"),   QStringLiteral("ftp"),
        QStringLiteral("ftps"),    QStringLiteral("sftp"),         QStringLiteral("smb"),     QStringLiteral("nfs"),
        QStringLiteral("mailto"),  QStringLiteral("tel"),          QStringLiteral("sms"),     QStringLiteral("callto"),
        QStringLiteral("news"),    QStringLiteral("nntp"),         QStringLiteral("irc"),     QStringLiteral("ssh"),
        QStringLiteral("telnet"),  QStringLiteral("ldap"),         QStringLiteral("ws"),      QStringLiteral("wss"),
        QStringLiteral("data"),    QStringLiteral("javascript"),   QStringLiteral("about"),   QStringLiteral("magnet"),
        QStringLiteral("git"),     QStringLiteral("ms-settings"),  QStringLiteral("search-ms"),
        QStringLiteral("shell"),   QStringLiteral("ms-appinstaller"), QStringLiteral("qor"),
        // Qt.openUrlExternally resolves its URL through the engine's interceptor first, so a refused URL arrives here
        // as the qrc: place refused loads are pointed at; left unhandled, Windows would ask the player what opens it.
        QStringLiteral("qrc"),
    };
    return schemes;
}

QUrl Confinement::packageOf(const QUrl &scene)
{
    const QString path = scene.scheme() == QStringLiteral("qrc") ? scene.path() : scene.toLocalFile();
    QString folder = QFileInfo(path).path();
    if (QFileInfo(folder).fileName().compare(QStringLiteral("scenes"), Qt::CaseInsensitive) == 0)
        folder = QFileInfo(folder).path();
    if (scene.scheme() == QStringLiteral("qrc")) {
        QUrl url;
        url.setScheme(QStringLiteral("qrc"));
        url.setPath(QDir::cleanPath(folder));
        return url;
    }
    return QUrl::fromLocalFile(QDir::cleanPath(folder));
}

void Confinement::apply(QQmlEngine *engine, const QUrl &package)
{
    const QString root = location(package);
    QStringList roots;
    for (const QString &path : engine->importPathList())
        roots << rootOf(path);
    roots << QStringLiteral("qrc:/qt/qml") << QStringLiteral("qrc:/qt-project.org/imports");
    roots.removeDuplicates();
    // Innermost first: a module is named from the import folder nearest to it.
    std::sort(roots.begin(), roots.end(), [](const QString &a, const QString &b) { return a.size() > b.size(); });

    // The engine does not take ownership of an interceptor: it goes when the engine does.
    auto *interceptor = new Interceptor(root, roots);
    engine->addUrlInterceptor(interceptor);
    QObject::connect(engine, &QObject::destroyed, [interceptor] { delete interceptor; });
    static RefusingAccessFactory access;
    engine->setNetworkAccessManagerFactory(&access);
    if (auto *io = engine->singletonInstance<SceneIO *>("QQ", "SceneIO"))
        io->confineTo(package);

    static UrlRefuser *refuser = new UrlRefuser;
    for (const QString &scheme : refusedSchemes())
        QDesktopServices::setUrlHandler(scheme, refuser, "refuse");
}

QStringList Confinement::refusals()
{
    QMutexLocker lock(&refusedLock);
    return refused;
}

}  // namespace qq

#include "confinement.moc"
