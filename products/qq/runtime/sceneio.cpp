#include "sceneio.h"
#include "kinematics.h"
#include "run.h"
#include "tilemap.h"

#include <QColor>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QImageReader>
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

    // A Tiled map names its tilesets and their images by path; they come along, keeping those paths. One outside the
    // map's folder could not keep its path in the project, so the map is refused, before anything is copied.
    const bool tiled = source.suffix().compare(QStringLiteral("tmj"), Qt::CaseInsensitive) == 0
                    || source.suffix().compare(QStringLiteral("json"), Qt::CaseInsensitive) == 0;
    const QStringList mapNeeds = tiled ? tiledMapNeeds(source.filePath()) : QStringList();
    for (const QString &needed : mapNeeds) {
        if (needed.startsWith(QStringLiteral("..")) || QDir::isAbsolutePath(needed)) {
            fail(QStringLiteral("%1 uses %2, outside its folder: keep a map's tilesets and images beside it, or in "
                                "folders under it, to add it to a project.").arg(source.fileName(), needed));
            return {};
        }
    }

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
    for (const QString &needed : std::as_const(mapNeeds)) {
        if (!copyOnce(source.absoluteDir().filePath(needed), assets.filePath(needed))) {
            fail(QStringLiteral("%1, which %2 needs, could not be copied.").arg(needed, source.fileName()));
            return {};
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
            QStringLiteral("Emitter"), QStringLiteral("Sound"),  QStringLiteral("Behaviour"),
            QStringLiteral("Trigger"), QStringLiteral("ScreenText"), QStringLiteral("ScreenBar"),
            QStringLiteral("ScreenButton"), QStringLiteral("ScreenImage"), QStringLiteral("ScreenPanel"),
            QStringLiteral("InputAction"), QStringLiteral("Sprite"), QStringLiteral("TileMap"), QStringLiteral("Player2D"),
            QStringLiteral("Music")};
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
    // A body play made sits beside its entity in the scene (Shape.qml, Trigger.qml, Player.qml): it leaves the scene with
    // it at once, taking any model that rides on it, and is deleted when control returns to the event loop, while its
    // world still stands (see discard()).
    if (auto *body = qobject_cast<QQuick3DObject *>(entity->property("physics").value<QObject *>());
        body && body->inherits("QAbstractPhysicsNode")) {
        body->setParentItem(nullptr);
        body->deleteLater();
    }
    node->setParentItem(nullptr);
    entity->deleteLater();
    return true;
}

QObject *SceneIO::viewOf(QObject *scene) const
{
    // Up the 3D tree to the node a World puts its scenes under, which names the View3D (World.qml).
    for (auto *o = qobject_cast<QQuick3DObject *>(scene); o; o = o->parentItem()) {
        if (auto *view = o->property("qqView").value<QObject *>())
            return view;
    }
    return nullptr;
}

QSize SceneIO::imageSize(const QUrl &image) const
{
    if (image.isEmpty() || outside(image))
        return {};
    const QString path = image.isLocalFile() ? image.toLocalFile()
                       : image.scheme() == QStringLiteral("qrc") ? QStringLiteral(":") + image.path()
                                                                 : QString();
    if (path.isEmpty())
        return {};
    QImageReader reader(path);
    const QSize size = reader.size();
    return size.isValid() ? size : QSize();
}

QVariantMap SceneIO::readTiledMap(const QUrl &file, const QString &layer) const
{
    return qq::readTiledMap(file, layer, [this](const QUrl &url) { return !outside(url); });
}

namespace {

/// An entity's box as QQ measures it (P3.11): never from PhysX's own shapes, only from the entity's fields and, for a
/// dynamic body, where its body is now.
Box boxOf(QObject *entity)
{
    const QString kind = entity->property("kind").toString();
    auto around = [](double x, double y, double z, double hx, double hy, double hz) {
        return Box{x - hx, x + hx, y - hy, y + hy, z - hz, z + hz};
    };
    if (kind == QStringLiteral("Player2D")) {
        return around(entity->property("atX").toDouble(), entity->property("atY").toDouble(),
                      entity->property("atZ").toDouble(), entity->property("width").toDouble() / 2,
                      entity->property("height").toDouble() / 2, 0.5);
    }
    QVector3D at = entity->property("position").value<QVector3D>();
    auto *body = entity->property("physics").value<QObject *>();
    if (body && (entity->property("body").toString() == QStringLiteral("dynamic") || kind == QStringLiteral("Player")))
        at = body->property("scenePosition").value<QVector3D>();
    if (kind == QStringLiteral("Player"))
        return around(at.x(), at.y(), at.z(), 0.3, 0.9, 0.3);  // its capsule: 0.6 across, 1.8 tall
    const QVector3D scale = entity->property("scale").value<QVector3D>();
    if (kind == QStringLiteral("Sprite")) {
        const QVector3D size = entity->property("size").value<QVector3D>();
        return around(at.x(), at.y(), at.z(), std::abs(size.x() * scale.x()) / 2, std::abs(size.y() * scale.y()) / 2,
                      0.5);
    }
    // A shape or a trigger: a unit across before its scale, turned about the depth axis at most (its box on the plane
    // grows with the turn); a sphere, a cylinder or a cone is measured by its box.
    const double turn = double(entity->property("eulerRotation").value<QVector3D>().z()) * std::numbers::pi / 180;
    const double c = std::abs(std::cos(turn));
    const double s = std::abs(std::sin(turn));
    const double hx = std::abs(scale.x()) / 2;
    const double hy = std::abs(scale.y()) / 2;
    return around(at.x(), at.y(), at.z(), c * hx + s * hy, s * hx + c * hy, std::abs(scale.z()) / 2);
}

/// What a 2D mover stands on and runs into in `scene`: solid tile maps, the boxes of static and kinematic shapes and
/// sprites, and the ground. Never `except`, nor another Player2D, nor anything PhysX moves.
QList<Box> solidsOf(const QVariantList &entities, QObject *except)
{
    QList<Box> solids;
    for (const QVariant &v : entities) {
        QObject *e = v.value<QObject *>();
        if (!e || e == except)
            continue;
        const QString kind = e->property("kind").toString();
        if (kind == QStringLiteral("TileMap")) {
            if (!e->property("solid").toBool())
                continue;
            const QVector3D at = e->property("position").value<QVector3D>();
            const double tw = e->property("tileWidth").toDouble();
            const double th = e->property("tileHeight").toDouble();
            for (const QVariant &r : e->property("map").toMap().value(QStringLiteral("solids")).toList()) {
                const QVariantList rect = r.toList();
                const double left = at.x() + rect.value(0).toDouble() * tw;
                const double top = at.y() - rect.value(1).toDouble() * th;
                solids.append(Box{left, left + rect.value(2).toDouble() * tw, top - rect.value(3).toDouble() * th, top,
                                  at.z() - 0.5, at.z() + 0.5});
            }
        } else if (kind == QStringLiteral("Shape") || kind == QStringLiteral("Sprite")) {
            const QString body = e->property("body").toString();
            if (body == QStringLiteral("static") || body == QStringLiteral("kinematic"))
                solids.append(boxOf(e));
        } else if (kind == QStringLiteral("Ground")) {
            const double half = e->property("extent").toDouble() / 2;
            solids.append(Box{-half, half, -2, 0, -half, half});
        }
    }
    return solids;
}

}  // namespace

QVariantMap SceneIO::move2D(QObject *scene, QObject *mover, double x, double y, double z, double halfWidth,
                            double halfHeight, double dx, double dy) const
{
    const Moved m = sweep2D(solidsOf(entities(scene), mover), x, y, z, halfWidth, halfHeight, dx, dy);
    return {{QStringLiteral("x"), m.x},       {QStringLiteral("y"), m.y},
            {QStringLiteral("left"), m.left}, {QStringLiteral("right"), m.right},
            {QStringLiteral("down"), m.down}, {QStringLiteral("up"), m.up}};
}

bool SceneIO::overlaps(QObject *a, QObject *b) const
{
    return a && b && a != b && isEntity(a) && isEntity(b) && qq::overlaps(boxOf(a), boxOf(b));
}

int SceneIO::freshSeed() const
{
    return Random::freshSeed();
}

QObject *SceneIO::entityOf(QObject *scene, QObject *object) const
{
    for (auto *o = qobject_cast<QQuick3DObject *>(object); o && o != scene; o = o->parentItem()) {
        // A body names its entity; a model riding on a body is found through the body.
        if (auto *named = o->property("entity").value<QObject *>(); named && isEntity(named))
            return named;
        if (isEntity(o) && o->parentItem() == scene)
            return o;
    }
    return nullptr;
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
    // A 2D scene stays one (P3.10): seen flat, 100 pixels to a unit, its centre at x 0 and its bottom on the ground at
    // y 0, the view exactly its height (its shapes are not pixel art).
    auto unit = [](double px) { return px / 100.0; };

    // Its background is the flat view's backdrop, top to bottom.
    const QString background = quoted(json.value(QStringLiteral("background")).toString(QStringLiteral("#07080c")));
    QString qml = QStringLiteral("import QQ\nScene {\n    name: %1\n    view: \"2d\"\n    pixelsPerUnit: 100\n"
                                 "    viewHeight: %2\n    pixelArt: false\n    skyTop: %3\n    skyHorizon: %3\n"
                                 "    groundHorizon: %3\n    Sun {}\n    Ground {}\n")
                      .arg(quoted(json.value(QStringLiteral("name")).toString(QStringLiteral("imported"))),
                           formatNumber(qBound(1.0, unit(h), 200.0)), background);
    QStringList skipped;
    int imported = 0;
    for (const QJsonValue &value : json.value(QStringLiteral("entities")).toArray()) {
        const QJsonObject e = value.toObject();
        const QString name = e.value(QStringLiteral("name")).toString(e.value(QStringLiteral("id")).toString());
        for (const char *later : {"motion", "follow"}) {
            if (e.contains(QLatin1String(later)))
                skipped << QStringLiteral("%1: %2, which QQ does not bring in yet").arg(name, QLatin1String(later));
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
