// Tile maps (DIRECTION P3.10, ADR-105): a map drawn in Tiled, the editor 2D games are built with, read from its JSON
// (`.tmj` or `.json`) and drawn as one mesh.
//
// readTiledMap() reads an orthogonal map's tile layer and its tileset (embedded, or external as `.tsj` or `.tsx`):
// the tiles, the tileset's image and its layout, and the solid tiles merged into as few rectangles as cover them, for
// colliders. TileGeometry turns that into one mesh, a quad per tile with its tile's corner of the image, so a whole
// level is one draw call and every tile lands on exact pixels.

#pragma once

#include <QString>
#include <QStringList>
#include <QUrl>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>
#include <QtQuick3D/qquick3dgeometry.h>

#include <functional>

namespace qq {

/// The map in `file`, its layer called `layer` (the first tile layer if empty): { error, width, height, tileWidth,
/// tileHeight, image (a whole URL), imageWidth, imageHeight, tileColumns, tileCount, firstGid, margin, spacing,
/// gids (row by row from the top-left, 0 for none), solids ([x, y, w, h] in tiles, merged), skipped (tiles of other
/// tilesets, not drawn) }. `allowed` says whether a file may be read (a confined game reads only its own).
QVariantMap readTiledMap(const QUrl &file, const QString &layer, const std::function<bool(const QUrl &)> &allowed);

/// The files the map at `path` needs, relative to its folder: its external tilesets and their images, or the image of
/// a tileset kept inside it. A map copied into a project brings these with it.
QStringList tiledMapNeeds(const QString &path);

/// The fewest rectangles (x, y, w, h, in tiles) that cover exactly the non-empty cells of a width x height grid: runs
/// along each row, joined down while they match.
QList<QList<int>> mergedSolids(const QList<int> &gids, int width, int height);

class TileGeometry : public QQuick3DGeometry
{
    Q_OBJECT
    QML_ELEMENT

    /// A map as readTiledMap() reads it.
    Q_PROPERTY(QVariantMap map READ map WRITE setMap NOTIFY mapChanged)
    /// Pixels of art to a unit: a 16-pixel tile is a unit across at 16.
    Q_PROPERTY(double pixelsPerUnit READ pixelsPerUnit WRITE setPixelsPerUnit NOTIFY pixelsPerUnitChanged)
    /// How many tiles the mesh has.
    Q_PROPERTY(int tiles READ tiles NOTIFY mapChanged)

public:
    explicit TileGeometry(QQuick3DObject *parent = nullptr);

    QVariantMap map() const { return m_map; }
    void setMap(const QVariantMap &map);
    double pixelsPerUnit() const { return m_pixelsPerUnit; }
    void setPixelsPerUnit(double ppu);
    int tiles() const { return m_tiles; }

signals:
    void mapChanged();
    void pixelsPerUnitChanged();

private:
    void rebuild();

    QVariantMap m_map;
    double m_pixelsPerUnit = 16;
    int m_tiles = 0;
};

}  // namespace qq
