#include "sceneio.h"

#include <QColor>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJSValue>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMetaProperty>
#include <QQmlComponent>
#include <QQmlContext>
#include <QQmlEngine>
#include <QRegularExpression>
#include <QSaveFile>
#include <QVector3D>
#include <QtQuick3D/qquick3dobject.h>

#include <cmath>
#include <numbers>

namespace qq {

namespace {

QString quoted(const QString &s)
{
    QString out;
    out.reserve(s.size() + 2);
    out += QLatin1Char('"');
    for (const QChar c : s) {
        switch (c.unicode()) {
        case '"': out += QStringLiteral("\\\""); break;
        case '\\': out += QStringLiteral("\\\\"); break;
        case '\n': out += QStringLiteral("\\n"); break;
        case '\r': out += QStringLiteral("\\r"); break;
        case '\t': out += QStringLiteral("\\t"); break;
        default: out += c;
        }
    }
    out += QLatin1Char('"');
    return out;
}

QString colourText(const QColor &c)
{
    return c.alpha() == 255 ? c.name(QColor::HexRgb) : c.name(QColor::HexArgb);
}

/// A URL as an entity holds it, made whole: Qt 6 keeps a URL property as written, so a relative one is resolved
/// against the file the entity was read from.
QUrl wholeUrl(const QObject *entity, const QUrl &url)
{
    if (url.isEmpty() || !url.isRelative())
        return url;
    const QQmlContext *context = qmlContext(entity);
    return context ? context->resolvedUrl(url) : url;
}

/// A URL as a scene file keeps it: relative to the file's folder when both are local files on the same drive, otherwise
/// whole. (Across drives no relative path exists, and a bare "X:/..." would be read back as a URL whose scheme is "x".)
QString urlText(const QUrl &url, const QDir *base)
{
    if (url.isEmpty())
        return quoted(QString());
    if (base && url.isLocalFile()) {
        const QString relative = base->relativeFilePath(url.toLocalFile());
        if (QDir::isRelativePath(relative))
            return quoted(relative);
    }
    return quoted(url.toString());
}

QString valueText(const QObject *entity, const QVariant &v, const QDir *base)
{
    switch (v.metaType().id()) {
    case QMetaType::Double:
    case QMetaType::Float:
        return formatNumber(v.toDouble());
    case QMetaType::Int:
    case QMetaType::LongLong:
        return QString::number(v.toLongLong());
    case QMetaType::Bool:
        return v.toBool() ? QStringLiteral("true") : QStringLiteral("false");
    case QMetaType::QString:
        return quoted(v.toString());
    case QMetaType::QUrl:
        return urlText(wholeUrl(entity, v.toUrl()), base);
    case QMetaType::QColor:
        return quoted(colourText(v.value<QColor>()));
    case QMetaType::QVector3D: {
        const QVector3D p = v.value<QVector3D>();
        return QStringLiteral("Qt.vector3d(%1, %2, %3)").arg(formatNumber(p.x()), formatNumber(p.y()), formatNumber(p.z()));
    }
    default:
        return quoted(v.toString());
    }
}

/// A list a QML type declares (`fields`): a JavaScript array arrives as a QJSValue.
QStringList stringList(const QVariant &v)
{
    if (v.metaType() == QMetaType::fromType<QJSValue>())
        return v.value<QJSValue>().toVariant().toStringList();
    return v.toStringList();
}

bool isEntity(const QObject *o)
{
    return o && !o->property("kind").toString().isEmpty();
}

void writeObject(QString &out, const QObject *o, int depth, const QDir *base)
{
    const QString indent(depth * 4, QLatin1Char(' '));
    out += indent + o->property("kind").toString() + QStringLiteral(" {\n");
    const QStringList fields = stringList(o->property("fields"));
    for (const QString &field : fields)
        out += indent + QStringLiteral("    ") + field + QStringLiteral(": ") + valueText(o, o->property(field.toUtf8()), base)
               + QLatin1Char('\n');
    if (const auto *node = qobject_cast<const QQuick3DObject *>(o)) {
        for (const QQuick3DObject *child : node->childItems()) {
            if (!isEntity(child))
                continue;
            out += QLatin1Char('\n');
            writeObject(out, child, depth + 1, base);
        }
    }
    out += indent + QStringLiteral("}\n");
}

QByteArray digest(const QString &path)
{
    QFile f(path);
    if (!f.open(QIODevice::ReadOnly))
        return {};
    QCryptographicHash h(QCryptographicHash::Sha256);
    h.addData(&f);
    return h.result();
}

/// Copy `from` to `to`, unless `to` already holds the same bytes. False if it holds different ones or the copy fails.
bool copyOnce(const QString &from, const QString &to)
{
    if (QFileInfo::exists(to))
        return digest(from) == digest(to);
    QDir().mkpath(QFileInfo(to).absolutePath());
    return QFile::copy(from, to);
}

}  // namespace

QString formatNumber(double value)
{
    if (!std::isfinite(value))
        return QStringLiteral("0");
    QString s = QString::number(value, 'f', 4);
    while (s.contains(QLatin1Char('.')) && (s.endsWith(QLatin1Char('0')) || s.endsWith(QLatin1Char('.'))))
        s.chop(1);
    if (s == QStringLiteral("-0"))
        s = QStringLiteral("0");
    return s;
}

SceneIO::SceneIO(QObject *parent) : QObject(parent) {}

SceneIO *SceneIO::create(QQmlEngine *qml, QJSEngine *)
{
    auto *io = new SceneIO;
    io->m_engine = qml;
    return io;
}

QQmlEngine *SceneIO::engine() const
{
    return m_engine ? m_engine : qmlEngine(this);
}

void SceneIO::fail(const QString &why)
{
    m_lastError = why;
    emit lastErrorChanged();
}

QString SceneIO::write(QObject *scene, const QUrl &file) const
{
    if (!isEntity(scene))
        return {};
    QString out = QStringLiteral("import QQ\n\n");
    if (file.isLocalFile()) {
        const QDir base = QFileInfo(file.toLocalFile()).absoluteDir();
        writeObject(out, scene, 0, &base);
    } else {
        writeObject(out, scene, 0, nullptr);
    }
    return out;
}

void SceneIO::confineTo(const QUrl &package)
{
    m_confined = true;
    m_package = package.scheme() == QStringLiteral("qrc") ? QStringLiteral("qrc:") + QDir::cleanPath(package.path())
                                                         : QDir::cleanPath(package.toLocalFile());
}

bool SceneIO::outside(const QUrl &url) const
{
    if (!m_confined || url.isEmpty())
        return false;
    if (url.scheme() == QStringLiteral("qrc"))
        return false;  // the executable's own resources: QQ's modules and a bundled game
    if (!url.isLocalFile())
        return true;
    const QString path = QDir::cleanPath(url.toLocalFile());
    return !(path.compare(m_package, Qt::CaseInsensitive) == 0
             || path.startsWith(m_package + QLatin1Char('/'), Qt::CaseInsensitive));
}

QString SceneIO::save(QObject *scene, const QUrl &file) const
{
    if (m_confined)
        return QStringLiteral("A game cannot write files.");
    if (!isEntity(scene))
        return QStringLiteral("There is no scene to save.");
    if (!file.isLocalFile())
        return QStringLiteral("A scene is saved to a file on this computer.");
    const QString path = file.toLocalFile();
    if (!QDir().mkpath(QFileInfo(path).absolutePath()))
        return QStringLiteral("The folder %1 could not be made.").arg(QFileInfo(path).absolutePath());
    QSaveFile out(path);
    if (!out.open(QIODevice::WriteOnly))
        return out.errorString();
    out.write(write(scene, file).toUtf8());
    if (!out.commit())
        return out.errorString();
    return {};
}

QObject *SceneIO::load(const QUrl &file, QObject *parent)
{
    if (outside(file)) {
        fail(QStringLiteral("A game reads only its own files."));
        return nullptr;
    }
    // A file on this computer, or one bundled in the executable (the QQ Player's own game, in a browser).
    const bool bundled = file.scheme() == QStringLiteral("qrc");
    QFile f(bundled ? QLatin1Char(':') + file.path() : file.toLocalFile());
    if ((!file.isLocalFile() && !bundled) || !f.open(QIODevice::ReadOnly)) {
        fail(QStringLiteral("The scene %1 could not be read.").arg(file.toDisplayString()));
        return nullptr;
    }
    return loadText(QString::fromUtf8(f.readAll()), file, parent);
}

QObject *SceneIO::loadText(const QString &text, const QUrl &file, QObject *parent)
{
    if (outside(file)) {
        fail(QStringLiteral("A game builds scenes only among its own files."));
        return nullptr;
    }
    QQmlEngine *qml = engine();
    if (!qml) {
        fail(QStringLiteral("No QML engine to build the scene with."));
        return nullptr;
    }
    // From the text, every time: the engine's cache would hand back a scene as it was when first read.
    QQmlComponent component(qml);
    component.setData(text.toUtf8(), file);
    if (component.isError()) {
        QStringList why;
        for (const QQmlError &e : component.errors())
            why << e.toString();
        fail(why.join(QLatin1Char('\n')));
        return nullptr;
    }
    QObject *scene = component.beginCreate(qml->rootContext());
    if (!scene) {
        fail(component.errorString());
        return nullptr;
    }
    if (auto *node = qobject_cast<QQuick3DObject *>(scene); node && parent) {
        node->setParentItem(qobject_cast<QQuick3DObject *>(parent));
    }
    scene->setParent(parent);
    // Owned by its parent, or by whoever asked for it: never collected out from under the Studio.
    QQmlEngine::setObjectOwnership(scene, QQmlEngine::CppOwnership);
    component.completeCreate();
    if (scene->property("kind").toString() != QStringLiteral("Scene")) {
        delete scene;
        fail(QStringLiteral("That file is not a QQ scene: its root is not a Scene."));
        return nullptr;
    }
    if (!m_lastError.isEmpty())
        fail(QString());
    return scene;
}

QObject *SceneIO::play(QObject *scene, const QUrl &file, QObject *parent)
{
    if (outside(file)) {
        fail(QStringLiteral("A game plays only among its own files."));
        return nullptr;
    }
    if (!isEntity(scene) || scene->property("kind").toString() != QStringLiteral("Scene")) {
        fail(QStringLiteral("There is no scene to play."));
        return nullptr;
    }
    QObject *copy = loadText(write(scene, file), file, parent);
    if (copy)
        copy->setProperty("playing", true);
    return copy;
}

QUrl SceneIO::resolvedUrl(QObject *entity, const QUrl &url) const
{
    return wholeUrl(entity, url);
}

QString SceneIO::fieldType(QObject *object, const QString &name) const
{
    if (!object)
        return {};
    const int index = object->metaObject()->indexOfProperty(name.toUtf8());
    if (index < 0)
        return {};
    switch (object->metaObject()->property(index).metaType().id()) {
    case QMetaType::Double:
    case QMetaType::Float: return QStringLiteral("real");
    case QMetaType::Int:
    case QMetaType::LongLong: return QStringLiteral("int");
    case QMetaType::Bool: return QStringLiteral("bool");
    case QMetaType::QString: return QStringLiteral("string");
    case QMetaType::QUrl: return QStringLiteral("url");
    case QMetaType::QColor: return QStringLiteral("color");
    case QMetaType::QVector3D: return QStringLiteral("vector3d");
    default: return {};
    }
}

QUrl SceneIO::adopt(const QUrl &model, const QUrl &project)
{
    if (m_confined) {
        fail(QStringLiteral("A game cannot copy files."));
        return {};
    }
    const bool bundled = model.scheme() == QStringLiteral("qrc");
    if ((!model.isLocalFile() && !bundled) || !project.isLocalFile()) {
        fail(QStringLiteral("Only files on this computer can be added to a project."));
        return {};
    }
    const QFileInfo source(bundled ? QLatin1Char(':') + model.path() : model.toLocalFile());
    const QDir root(project.toLocalFile());
    if (!source.isFile() || !root.exists()) {
        fail(QStringLiteral("%1 or the project folder does not exist.").arg(source.fileName()));
        return {};
    }
    const QString rootPath = root.canonicalPath() + QLatin1Char('/');
    if (!bundled && source.canonicalFilePath().startsWith(rootPath, Qt::CaseInsensitive))
        return model;

    // Logic is code, kept apart from what it uses.
    const bool logic = source.suffix().compare(QStringLiteral("qml"), Qt::CaseInsensitive) == 0;
    const QDir assets(root.filePath(logic ? QStringLiteral("logic") : QStringLiteral("assets")));
    // A model of the same name but different content is kept beside it, never over it.
    QString name = source.fileName();
    for (int n = 2; QFileInfo::exists(assets.filePath(name)) && digest(assets.filePath(name)) != digest(source.filePath());
         ++n)
        name = QStringLiteral("%1-%2.%3").arg(source.completeBaseName()).arg(n).arg(source.suffix());
    if (!copyOnce(source.filePath(), assets.filePath(name))) {
        fail(QStringLiteral("%1 could not be copied into the project.").arg(source.fileName()));
        return {};
    }

    // A .gltf names its buffers and images by path; they come along, keeping those paths.
    if (source.suffix().compare(QStringLiteral("gltf"), Qt::CaseInsensitive) == 0) {
        QFile f(source.filePath());
        if (f.open(QIODevice::ReadOnly)) {
            const QJsonObject gltf = QJsonDocument::fromJson(f.readAll()).object();
            for (const char *list : {"buffers", "images"}) {
                for (const QJsonValue &entry : gltf.value(QLatin1String(list)).toArray()) {
                    const QString uri = entry.toObject().value(QStringLiteral("uri")).toString();
                    if (uri.isEmpty() || uri.startsWith(QStringLiteral("data:")) || uri.contains(QStringLiteral("..")))
                        continue;
                    const QString from = source.absoluteDir().filePath(QUrl::fromPercentEncoding(uri.toUtf8()));
                    if (!copyOnce(from, assets.filePath(QUrl::fromPercentEncoding(uri.toUtf8())))) {
                        fail(QStringLiteral("%1, which %2 needs, could not be copied.").arg(uri, source.fileName()));
                        return {};
                    }
                }
            }
        }
    }
    return QUrl::fromLocalFile(assets.filePath(name));
}

QUrl SceneIO::writeLogic(const QUrl &project, const QString &name, const QString &text)
{
    if (m_confined) {
        fail(QStringLiteral("A game cannot write files."));
        return {};
    }
    if (!project.isLocalFile() || !QDir(project.toLocalFile()).exists()) {
        fail(QStringLiteral("Logic is written into a project folder."));
        return {};
    }
    QString safe = name.trimmed().toLower();
    safe.replace(QRegularExpression(QStringLiteral("[^a-z0-9-]+")), QStringLiteral("-"));
    safe.remove(QRegularExpression(QStringLiteral("^-+|-+$")));
    if (safe.isEmpty())
        safe = QStringLiteral("logic");
    const QString path = QDir(project.toLocalFile()).filePath(QStringLiteral("logic/%1.qml").arg(safe));
    QDir().mkpath(QFileInfo(path).absolutePath());
    QSaveFile out(path);
    if (!out.open(QIODevice::WriteOnly)) {
        fail(out.errorString());
        return {};
    }
    out.write(text.toUtf8());
    if (!out.commit()) {
        fail(out.errorString());
        return {};
    }
    return QUrl::fromLocalFile(path);
}

QStringList SceneIO::kinds() const
{
    return {QStringLiteral("Shape"),  QStringLiteral("Lamp"),    QStringLiteral("Prop"),
            QStringLiteral("Sun"),    QStringLiteral("Ground"),  QStringLiteral("Player"),
            QStringLiteral("Emitter"), QStringLiteral("Sound"),  QStringLiteral("Behaviour")};
}

QVariantList SceneIO::entities(QObject *scene) const
{
    QVariantList out;
    if (const auto *node = qobject_cast<const QQuick3DObject *>(scene)) {
        for (QQuick3DObject *child : node->childItems()) {
            if (isEntity(child))
                out << QVariant::fromValue(static_cast<QObject *>(child));
        }
    }
    return out;
}

QObject *SceneIO::add(QObject *scene, const QString &kind, const QVariantMap &properties)
{
    auto *parent = qobject_cast<QQuick3DObject *>(scene);
    if (!parent || scene->property("kind").toString() != QStringLiteral("Scene")) {
        fail(QStringLiteral("Entities are added to a scene."));
        return nullptr;
    }
    if (!kinds().contains(kind)) {
        fail(QStringLiteral("QQ has no entity called %1.").arg(kind));
        return nullptr;
    }
    QQmlEngine *qml = engine();
    QQmlComponent component(qml);
    // In the scene's own context, so a relative model URL means what it means in the scene's file.
    QQmlContext *context = qmlContext(scene) ? qmlContext(scene) : qml->rootContext();
    component.setData(QStringLiteral("import QQ\n%1 {}\n").arg(kind).toUtf8(), context->baseUrl());
    QObject *entity = component.beginCreate(context);
    if (!entity) {
        fail(component.errorString());
        return nullptr;
    }
    for (auto it = properties.cbegin(); it != properties.cend(); ++it)
        entity->setProperty(it.key().toUtf8(), it.value());
    qobject_cast<QQuick3DObject *>(entity)->setParentItem(parent);
    entity->setParent(scene);
    QQmlEngine::setObjectOwnership(entity, QQmlEngine::CppOwnership);
    component.completeCreate();
    return entity;
}

bool SceneIO::remove(QObject *entity)
{
    auto *node = qobject_cast<QQuick3DObject *>(entity);
    if (!node || !isEntity(entity) || !node->parentItem()
        || node->parentItem()->property("kind").toString() != QStringLiteral("Scene"))
        return false;
    node->setParentItem(nullptr);
    entity->deleteLater();
    return true;
}

void SceneIO::discard(QObject *scene)
{
    auto *node = qobject_cast<QQuick3DObject *>(scene);
    if (!node || scene->property("kind").toString() != QStringLiteral("Scene"))
        return;
    // The bodies play made go first, while their physics world still stands. Qt Quick 3D Physics frees a body's
    // simulation state with its world; a body that outlives its world, and is destroyed while any other world exists,
    // writes into that freed state (QPhysicsWorld::deregisterNode). A scene's world is its first child, so left to
    // the ordinary order it would go first.
    QList<QQuick3DObject *> bodies;
    QList<QQuick3DObject *> pending = node->childItems();
    while (!pending.isEmpty()) {
        QQuick3DObject *item = pending.takeLast();
        if (item->inherits("QAbstractPhysicsNode"))
            bodies << item;
        else
            pending << item->childItems();
    }
    for (QQuick3DObject *body : std::as_const(bodies))
        delete body;
    node->setParentItem(nullptr);
    scene->deleteLater();
}

QVariantMap SceneIO::project(const QUrl &folder) const
{
    if (outside(folder))
        return {};
    const QDir root(folder.toLocalFile());
    auto names = [&root](const QString &folder) {
        QStringList out;
        for (const QFileInfo &file :
             QDir(root.filePath(folder)).entryInfoList({QStringLiteral("*.qml")}, QDir::Files, QDir::Name))
            out << file.completeBaseName();
        return out;
    };
    return {{QStringLiteral("name"), root.dirName()},
            {QStringLiteral("repository"), QFileInfo(root.filePath(QStringLiteral(".git"))).exists()},
            {QStringLiteral("scenes"), names(QStringLiteral("scenes"))},
            {QStringLiteral("logic"), names(QStringLiteral("logic"))}};
}

QVariantMap SceneIO::importQqJson(const QUrl &file)
{
    QVariantMap result{{QStringLiteral("qml"), QString()},
                       {QStringLiteral("imported"), 0},
                       {QStringLiteral("skipped"), QStringList()},
                       {QStringLiteral("error"), QString()}};
    auto refuse = [&result](const QString &why) {
        result[QStringLiteral("error")] = why;
        return result;
    };

    if (outside(file))
        return refuse(QStringLiteral("A game reads only its own files."));
    QFile f(file.toLocalFile());
    if (!file.isLocalFile() || !f.open(QIODevice::ReadOnly))
        return refuse(QStringLiteral("The file could not be read."));
    QJsonParseError parse{};
    const QJsonObject json = QJsonDocument::fromJson(f.readAll(), &parse).object();
    if (parse.error != QJsonParseError::NoError)
        return refuse(QStringLiteral("The file is not JSON."));
    if (json.value(QStringLiteral("qq")).toInt() != 1)
        return refuse(QStringLiteral("This reads QQ scenes of format 1."));

    const double h = json.value(QStringLiteral("size")).toObject().value(QStringLiteral("h")).toDouble(540);
    // A 2D scene stands upright on the ground, facing the eye: 100 pixels to a unit, its centre at x 0, its bottom at y 0.
    auto unit = [](double px) { return px / 100.0; };

    QString qml = QStringLiteral("import QQ\nScene {\n    name: %1\n    Sun {}\n    Ground {}\n")
                      .arg(quoted(json.value(QStringLiteral("name")).toString(QStringLiteral("imported"))));
    QStringList skipped;
    int imported = 0;
    for (const QJsonValue &value : json.value(QStringLiteral("entities")).toArray()) {
        const QJsonObject e = value.toObject();
        const QString name = e.value(QStringLiteral("name")).toString(e.value(QStringLiteral("id")).toString());
        for (const char *later : {"motion", "follow"}) {
            if (e.contains(QLatin1String(later)))
                skipped << QStringLiteral("%1: %2, which has no 3D form yet").arg(name, QLatin1String(later));
        }
        const QJsonObject t = e.value(QStringLiteral("transform")).toObject();
        const double x = unit(t.value(QStringLiteral("x")).toDouble());
        const double y = unit(h / 2 - t.value(QStringLiteral("y")).toDouble());
        if (e.contains(QStringLiteral("emitter"))) {
            // A 2D spring of particles, in metres: its speed in pixels a second is a hundredth of that in metres, and a
            // spread of most of a circle becomes a burst every way, a narrow one a plume.
            const QJsonObject m = e.value(QStringLiteral("emitter")).toObject();
            const double speed = unit(m.value(QStringLiteral("speed")).toDouble(100));
            const double spread = m.value(QStringLiteral("spread")).toDouble(60);
            const bool burst = spread >= 180;
            qml += QStringLiteral("    Emitter {\n        name: %1\n        position: Qt.vector3d(%2, %3, 0)\n"
                                  "        colour: %4\n        rate: %5\n        life: %6\n        size: %7\n"
                                  "        speed: %8\n        spread: %9\n    }\n")
                       .arg(quoted(name + QStringLiteral(" sparks")), formatNumber(x), formatNumber(y),
                            quoted(m.value(QStringLiteral("colour")).toString(QStringLiteral("#ffb15c"))),
                            formatNumber(m.value(QStringLiteral("rate")).toDouble(60)),
                            formatNumber(m.value(QStringLiteral("life")).toDouble(2)),
                            formatNumber(qMax(0.05, unit(m.value(QStringLiteral("size")).toDouble(4)) * 1.5)),
                            formatNumber(burst ? speed * 0.2 : speed),
                            formatNumber(burst ? speed : speed * std::sin(spread / 2 * std::numbers::pi / 180)));
            ++imported;
        }
        if (!e.contains(QStringLiteral("shape")))
            continue;
        const QJsonObject s = e.value(QStringLiteral("shape")).toObject();
        const double scale = t.value(QStringLiteral("scale")).toDouble(1);
        const double w = unit(s.value(QStringLiteral("w")).toDouble(64)) * scale;
        const double hh = unit(s.value(QStringLiteral("h")).toDouble(64)) * scale;
        const bool round = s.value(QStringLiteral("kind")).toString() == QStringLiteral("circle");
        const double depth = round ? qMin(w, hh) : 0.25;
        const QString colour = s.value(QStringLiteral("colour")).toString(QStringLiteral("#5ad1ff"));
        const double glow = s.value(QStringLiteral("glow")).toDouble(0);
        qml += QStringLiteral("    Shape {\n        name: %1\n        form: %2\n"
                              "        position: Qt.vector3d(%3, %4, 0)\n        eulerRotation: Qt.vector3d(0, 0, %5)\n"
                              "        scale: Qt.vector3d(%6, %7, %8)\n        colour: %9\n")
                   .arg(quoted(name), quoted(round ? QStringLiteral("sphere") : QStringLiteral("cube")),
                        formatNumber(x), formatNumber(y),
                        formatNumber(-t.value(QStringLiteral("rotation")).toDouble()), formatNumber(w),
                        formatNumber(hh), formatNumber(depth), quoted(colour));
        if (glow > 0)
            qml += QStringLiteral("        emissive: %1\n        emissivePower: %2\n").arg(quoted(colour), formatNumber(glow * 3));
        qml += QStringLiteral("    }\n");
        ++imported;
    }
    qml += QStringLiteral("}\n");

    // Built, then written by the one writer, so an imported scene is canonical like any other.
    QObject *scene = loadText(qml, file, nullptr);
    if (!scene)
        return refuse(QStringLiteral("The converted scene did not build: %1").arg(m_lastError));
    result[QStringLiteral("qml")] = write(scene);
    delete scene;
    result[QStringLiteral("imported")] = imported;
    result[QStringLiteral("skipped")] = skipped;
    return result;
}

}  // namespace qq
