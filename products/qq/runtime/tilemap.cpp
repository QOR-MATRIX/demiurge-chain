#include "tilemap.h"

#include <QByteArray>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QHash>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QVector3D>
#include <QXmlStreamReader>
#include <QtEndian>

#include <utility>

namespace qq {

namespace {

constexpr quint32 flippedHorizontally = 0x80000000u;
constexpr quint32 flippedVertically = 0x40000000u;
constexpr quint32 flipBits = 0xE0000000u;

QString localPath(const QUrl &url)
{
    if (url.isLocalFile())
        return url.toLocalFile();
    if (url.scheme() == QStringLiteral("qrc"))
        return QStringLiteral(":") + url.path();
    return {};
}

/// `name` beside `file`: a tileset beside its map, an image beside its tileset.
QUrl besides(const QUrl &file, const QString &name)
{
    return file.resolved(QUrl(name));
}

/// The tile layer called `name`, or the first one, searching inside groups.
QJsonObject findLayer(const QJsonArray &layers, const QString &name)
{
    for (const QJsonValue &v : layers) {
        const QJsonObject layer = v.toObject();
        const QString type = layer.value(QStringLiteral("type")).toString();
        if (type == QStringLiteral("tilelayer") && (name.isEmpty() || layer.value(QStringLiteral("name")).toString() == name))
            return layer;
        if (type == QStringLiteral("group")) {
            const QJsonObject inner = findLayer(layer.value(QStringLiteral("layers")).toArray(), name);
            if (!inner.isEmpty())
                return inner;
        }
    }
    return {};
}

/// A layer's tiles, as Tiled writes them: a JSON array, or Base64 of little-endian 32-bit values, uncompressed or zlib.
bool layerGids(const QJsonObject &layer, qsizetype count, QList<quint32> &gids, QString &why)
{
    const QJsonValue data = layer.value(QStringLiteral("data"));
    if (data.isArray()) {
        for (const QJsonValue &v : data.toArray())
            gids << quint32(v.toDouble());
    } else if (layer.value(QStringLiteral("encoding")).toString() == QStringLiteral("base64")) {
        QByteArray bytes = QByteArray::fromBase64(data.toString().toLatin1());
        const QString compression = layer.value(QStringLiteral("compression")).toString();
        if (compression == QStringLiteral("zlib")) {
            // qUncompress wants the expected size first, big-endian; Tiled's stream is plain zlib.
            QByteArray sized(4, '\0');
            qToBigEndian<quint32>(quint32(count * 4), sized.data());
            bytes = qUncompress(sized + bytes);
        } else if (!compression.isEmpty()) {
            why = QStringLiteral("Layers compressed with %1 are not read: save the map's layers as CSV, or Base64 with "
                                 "zlib, in Tiled.").arg(compression);
            return false;
        }
        for (qsizetype i = 0; i + 4 <= bytes.size(); i += 4)
            gids << qFromLittleEndian<quint32>(bytes.constData() + i);
    } else {
        why = QStringLiteral("The layer's tiles are in a form that is not read.");
        return false;
    }
    if (gids.size() != count) {
        why = QStringLiteral("The layer has %1 tiles where the map has room for %2.").arg(gids.size()).arg(count);
        return false;
    }
    return true;
}

/// A tileset as JSON keys: from a `.tsj` or `.json` file, or an embedded one, or translated from a `.tsx`.
bool readTileset(const QUrl &file, QJsonObject &tileset, QString &why)
{
    QFile f(localPath(file));
    if (!f.open(QIODevice::ReadOnly)) {
        why = QStringLiteral("The map's tileset could not be read.");
        return false;
    }
    if (file.path().endsWith(QStringLiteral(".tsx"), Qt::CaseInsensitive)) {
        QXmlStreamReader xml(&f);
        while (!xml.atEnd()) {
            if (!xml.readNextStartElement())
                continue;
            const auto attr = [&xml](const char *name) { return xml.attributes().value(QLatin1String(name)).toString(); };
            if (xml.name() == QStringLiteral("tileset")) {
                for (const char *key : {"tilewidth", "tileheight", "tilecount", "columns", "margin", "spacing"})
                    tileset.insert(QLatin1String(key), attr(key).toInt());
            } else if (xml.name() == QStringLiteral("image")) {
                tileset.insert(QStringLiteral("image"), attr("source"));
                tileset.insert(QStringLiteral("imagewidth"), attr("width").toInt());
                tileset.insert(QStringLiteral("imageheight"), attr("height").toInt());
            }
        }
        if (xml.hasError()) {
            why = QStringLiteral("The map's tileset is not readable XML.");
            return false;
        }
        return true;
    }
    QJsonParseError parse{};
    tileset = QJsonDocument::fromJson(f.readAll(), &parse).object();
    if (parse.error != QJsonParseError::NoError) {
        why = QStringLiteral("The map's tileset is not JSON.");
        return false;
    }
    return true;
}

}  // namespace

QStringList tiledMapNeeds(const QString &path)
{
    QStringList needs;
    QFile f(path);
    if (!f.open(QIODevice::ReadOnly))
        return needs;
    const QJsonObject map = QJsonDocument::fromJson(f.readAll()).object();
    const QDir folder = QFileInfo(path).absoluteDir();
    for (const QJsonValue &v : map.value(QStringLiteral("tilesets")).toArray()) {
        QJsonObject tileset = v.toObject();
        QString base = QStringLiteral(".");  // where the tileset's own paths start, from the map's folder
        if (tileset.contains(QStringLiteral("source"))) {
            const QString source = tileset.value(QStringLiteral("source")).toString();
            needs << QDir::cleanPath(source);
            base = QFileInfo(source).path();
            QJsonObject external;
            QString why;
            if (!readTileset(QUrl::fromLocalFile(folder.filePath(source)), external, why))
                continue;
            tileset = external;
        }
        const QString image = tileset.value(QStringLiteral("image")).toString();
        if (!image.isEmpty())
            needs << QDir::cleanPath(base + QLatin1Char('/') + image);
    }
    needs.removeDuplicates();
    return needs;
}

QList<QList<int>> mergedSolids(const QList<int> &gids, int width, int height)
{
    QList<QList<int>> done;
    // Rectangles still growing downward, by their run (x, w): their index in `done`.
    QHash<QPair<int, int>, qsizetype> growing;
    for (int y = 0; y < height; ++y) {
        QHash<QPair<int, int>, qsizetype> next;
        int x = 0;
        while (x < width) {
            if (gids.value(y * width + x) == 0) {
                ++x;
                continue;
            }
            const int start = x;
            while (x < width && gids.value(y * width + x) != 0)
                ++x;
            const QPair<int, int> run(start, x - start);
            if (const auto it = growing.constFind(run); it != growing.cend()) {
                done[*it][3] += 1;
                next.insert(run, *it);
            } else {
                done.append({start, y, x - start, 1});
                next.insert(run, done.size() - 1);
            }
        }
        growing = next;
    }
    return done;
}

QVariantMap readTiledMap(const QUrl &file, const QString &layer, const std::function<bool(const QUrl &)> &allowed)
{
    QVariantMap out{{QStringLiteral("error"), QString()}};
    auto fail = [&out](const QString &why) {
        out[QStringLiteral("error")] = why;
        return out;
    };
    if (file.isEmpty())
        return fail(QStringLiteral("No map."));
    if (!allowed(file))
        return fail(QStringLiteral("A game reads only its own files."));
    QFile f(localPath(file));
    if (!f.open(QIODevice::ReadOnly))
        return fail(QStringLiteral("The map could not be read."));
    QJsonParseError parse{};
    const QJsonObject map = QJsonDocument::fromJson(f.readAll(), &parse).object();
    if (parse.error != QJsonParseError::NoError)
        return fail(QStringLiteral("The map is not JSON: save it from Tiled as a JSON map (.tmj)."));
    if (map.value(QStringLiteral("orientation")).toString(QStringLiteral("orthogonal")) != QStringLiteral("orthogonal"))
        return fail(QStringLiteral("Only orthogonal maps are read."));
    if (map.value(QStringLiteral("infinite")).toBool())
        return fail(QStringLiteral("Infinite maps are not read: give the map a fixed size in Tiled."));

    const int width = map.value(QStringLiteral("width")).toInt();
    const int height = map.value(QStringLiteral("height")).toInt();
    const int tileWidth = map.value(QStringLiteral("tilewidth")).toInt();
    const int tileHeight = map.value(QStringLiteral("tileheight")).toInt();
    if (width <= 0 || height <= 0 || tileWidth <= 0 || tileHeight <= 0)
        return fail(QStringLiteral("The map has no size."));

    const QJsonObject found = findLayer(map.value(QStringLiteral("layers")).toArray(), layer);
    if (found.isEmpty())
        return fail(layer.isEmpty() ? QStringLiteral("The map has no tile layer.")
                                    : QStringLiteral("The map has no tile layer called %1.").arg(layer));
    QList<quint32> gids;
    QString why;
    if (!layerGids(found, qsizetype(width) * height, gids, why))
        return fail(why);

    const QJsonArray tilesets = map.value(QStringLiteral("tilesets")).toArray();
    if (tilesets.isEmpty())
        return fail(QStringLiteral("The map has no tileset."));
    QJsonObject tileset = tilesets.first().toObject();
    const int firstGid = tileset.value(QStringLiteral("firstgid")).toInt(1);
    QUrl tilesetFile = file;
    if (tileset.contains(QStringLiteral("source"))) {
        tilesetFile = besides(file, tileset.value(QStringLiteral("source")).toString());
        if (!allowed(tilesetFile))
            return fail(QStringLiteral("A game reads only its own files."));
        QJsonObject external;
        if (!readTileset(tilesetFile, external, why))
            return fail(why);
        tileset = external;
    }
    if (!tileset.contains(QStringLiteral("image")))
        return fail(QStringLiteral("The tileset must be one image: a collection of images is not read yet."));
    const QUrl image = besides(tilesetFile, tileset.value(QStringLiteral("image")).toString());
    if (!allowed(image))
        return fail(QStringLiteral("A game reads only its own files."));

    const int tileCount = tileset.value(QStringLiteral("tilecount")).toInt();
    QVariantList list;
    QList<int> solid;
    int skipped = 0;
    for (quint32 gid : std::as_const(gids)) {
        const quint32 id = gid & ~flipBits;
        if (id != 0 && (int(id) < firstGid || (tileCount > 0 && int(id) >= firstGid + tileCount))) {
            ++skipped;  // a tile of another tileset
            gid = 0;
        }
        list << QVariant::fromValue(qint64(gid));
        solid << ((gid & ~flipBits) != 0 ? 1 : 0);
    }
    QVariantList rects;
    for (const QList<int> &r : mergedSolids(solid, width, height))
        rects << QVariant::fromValue(QVariantList{r[0], r[1], r[2], r[3]});

    out[QStringLiteral("width")] = width;
    out[QStringLiteral("height")] = height;
    out[QStringLiteral("tileWidth")] = tileWidth;
    out[QStringLiteral("tileHeight")] = tileHeight;
    out[QStringLiteral("image")] = image;
    out[QStringLiteral("imageWidth")] = tileset.value(QStringLiteral("imagewidth")).toInt();
    out[QStringLiteral("imageHeight")] = tileset.value(QStringLiteral("imageheight")).toInt();
    out[QStringLiteral("sheetTileWidth")] = tileset.value(QStringLiteral("tilewidth")).toInt(tileWidth);
    out[QStringLiteral("sheetTileHeight")] = tileset.value(QStringLiteral("tileheight")).toInt(tileHeight);
    out[QStringLiteral("tileColumns")] = qMax(1, tileset.value(QStringLiteral("columns")).toInt(1));
    out[QStringLiteral("tileCount")] = tileCount;
    out[QStringLiteral("firstGid")] = firstGid;
    out[QStringLiteral("margin")] = tileset.value(QStringLiteral("margin")).toInt();
    out[QStringLiteral("spacing")] = tileset.value(QStringLiteral("spacing")).toInt();
    out[QStringLiteral("gids")] = list;
    out[QStringLiteral("solids")] = rects;
    out[QStringLiteral("skipped")] = skipped;
    return out;
}

TileGeometry::TileGeometry(QQuick3DObject *parent) : QQuick3DGeometry(parent) {}

void TileGeometry::setMap(const QVariantMap &map)
{
    m_map = map;
    rebuild();
    emit mapChanged();
}

void TileGeometry::setPixelsPerUnit(double ppu)
{
    if (qFuzzyCompare(ppu, m_pixelsPerUnit) || ppu <= 0)
        return;
    m_pixelsPerUnit = ppu;
    rebuild();
    emit pixelsPerUnitChanged();
}

void TileGeometry::rebuild()
{
    clear();
    m_tiles = 0;
    const int width = m_map.value(QStringLiteral("width")).toInt();
    const int height = m_map.value(QStringLiteral("height")).toInt();
    const QVariantList gids = m_map.value(QStringLiteral("gids")).toList();
    const double imageWidth = m_map.value(QStringLiteral("imageWidth")).toDouble();
    const double imageHeight = m_map.value(QStringLiteral("imageHeight")).toDouble();
    if (!m_map.value(QStringLiteral("error")).toString().isEmpty() || width <= 0 || height <= 0
        || gids.size() != qsizetype(width) * height || imageWidth <= 0 || imageHeight <= 0) {
        update();
        return;
    }
    const double tileWidth = m_map.value(QStringLiteral("tileWidth")).toDouble();
    const double tileHeight = m_map.value(QStringLiteral("tileHeight")).toDouble();
    const double sheetWidth = m_map.value(QStringLiteral("sheetTileWidth")).toDouble();
    const double sheetHeight = m_map.value(QStringLiteral("sheetTileHeight")).toDouble();
    const int columns = m_map.value(QStringLiteral("tileColumns")).toInt();
    const int firstGid = m_map.value(QStringLiteral("firstGid")).toInt();
    const double margin = m_map.value(QStringLiteral("margin")).toDouble();
    const double spacing = m_map.value(QStringLiteral("spacing")).toDouble();
    const float unitWidth = float(tileWidth / m_pixelsPerUnit);
    const float unitHeight = float(tileHeight / m_pixelsPerUnit);
    // A hundredth of a texel in from each edge, so a neighbouring tile never bleeds in.
    constexpr double inset = 0.01;

    struct Vertex {
        float x, y, z, nx, ny, nz, u, v;
    };
    QList<Vertex> vertices;
    QList<quint32> indices;
    for (int row = 0; row < height; ++row) {
        for (int column = 0; column < width; ++column) {
            const quint32 gid = quint32(gids.at(qsizetype(row) * width + column).toLongLong());
            const quint32 id = gid & ~flipBits;
            if (id == 0)
                continue;
            const int tile = int(id) - firstGid;
            const double px = margin + (tile % columns) * (sheetWidth + spacing);
            const double py = margin + (tile / columns) * (sheetHeight + spacing);
            float u0 = float((px + inset) / imageWidth);
            float u1 = float((px + sheetWidth - inset) / imageWidth);
            // Texture coordinates count up from the image's foot.
            float v0 = float(1.0 - (py + sheetHeight - inset) / imageHeight);
            float v1 = float(1.0 - (py + inset) / imageHeight);
            if (gid & flippedHorizontally)
                std::swap(u0, u1);
            if (gid & flippedVertically)
                std::swap(v0, v1);
            const float x0 = column * unitWidth;
            const float x1 = x0 + unitWidth;
            const float top = -row * unitHeight;
            const float bottom = top - unitHeight;
            const auto base = quint32(vertices.size());
            vertices << Vertex{x0, bottom, 0, 0, 0, 1, u0, v0} << Vertex{x1, bottom, 0, 0, 0, 1, u1, v0}
                     << Vertex{x1, top, 0, 0, 0, 1, u1, v1} << Vertex{x0, top, 0, 0, 0, 1, u0, v1};
            indices << base << base + 1 << base + 2 << base << base + 2 << base + 3;
            ++m_tiles;
        }
    }
    if (vertices.isEmpty()) {
        update();
        return;
    }
    setVertexData(QByteArray(reinterpret_cast<const char *>(vertices.constData()),
                             vertices.size() * qsizetype(sizeof(Vertex))));
    setIndexData(QByteArray(reinterpret_cast<const char *>(indices.constData()),
                            indices.size() * qsizetype(sizeof(quint32))));
    setStride(int(sizeof(Vertex)));
    setPrimitiveType(QQuick3DGeometry::PrimitiveType::Triangles);
    addAttribute(QQuick3DGeometry::Attribute::PositionSemantic, 0, QQuick3DGeometry::Attribute::F32Type);
    addAttribute(QQuick3DGeometry::Attribute::NormalSemantic, 12, QQuick3DGeometry::Attribute::F32Type);
    addAttribute(QQuick3DGeometry::Attribute::TexCoord0Semantic, 24, QQuick3DGeometry::Attribute::F32Type);
    addAttribute(QQuick3DGeometry::Attribute::IndexSemantic, 0, QQuick3DGeometry::Attribute::U32Type);
    setBounds(QVector3D(0, -height * unitHeight, -0.01f), QVector3D(width * unitWidth, 0, 0.01f));
    update();
}

}  // namespace qq
