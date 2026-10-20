"""Writes the 2D fixtures (P3.10): a tileset, a Tiled map drawn with it, and a sheet of sprite frames.

  tiles.png   2 x 2 tiles of 16 px: red, green, blue, yellow, each with a white top-left pixel so a tile drawn
              upside down or mirrored is seen to be
  tiles.tsj   the tileset, as Tiled saves one beside its map
  level.tmj   a 10 x 5 map using it: a floor along the foot (one green tile mirrored), a ledge, and a wall at the right;
              a second layer, "decor", holds one tile
  sheet.png   4 x 2 frames of 8 px, each its own colour with a white 2 x 2 block at its top-left

The checks read these colours back from the screen, so they are listed in FRAMES and TILES. Standard library only:
python make_2d.py
"""
import json
import pathlib
import struct
import zlib

HERE = pathlib.Path(__file__).parent
WHITE = (255, 255, 255, 255)
TILES = [(192, 48, 40), (48, 160, 80), (48, 80, 192), (224, 192, 64)]
FRAMES = [(200, 40, 40), (40, 200, 40), (40, 40, 200), (200, 200, 40),
          (200, 40, 200), (40, 200, 200), (240, 120, 20), (120, 20, 240)]
MIRRORED = 0x80000000


def png(width, height, pixel):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    raw = b"".join(bytes([0]) + b"".join(bytes(pixel(x, y)) for x in range(width)) for y in range(height))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def tile_pixel(x, y):
    tile = (y // 16) * 2 + x // 16
    if x % 16 == 0 and y % 16 == 0:
        return WHITE
    return TILES[tile] + (255,)


def frame_pixel(x, y):
    frame = (y // 8) * 4 + x // 8
    if x % 8 < 2 and y % 8 < 2:
        return WHITE
    return FRAMES[frame] + (255,)


def write_json(name, value):
    (HERE / name).write_text(json.dumps(value, indent=1) + "\n", encoding="utf-8", newline="\n")


(HERE / "tiles.png").write_bytes(png(32, 32, tile_pixel))
(HERE / "sheet.png").write_bytes(png(32, 16, frame_pixel))

write_json("tiles.tsj", {
    "columns": 2, "image": "tiles.png", "imageheight": 32, "imagewidth": 32, "margin": 0, "name": "tiles",
    "spacing": 0, "tilecount": 4, "tileheight": 16, "tilewidth": 16, "type": "tileset", "version": "1.10",
})

W, H = 10, 5
ground = [0] * (W * H)
for row in range(4):
    ground[row * W + 9] = 3                       # the wall, blue
ground[1 * W + 5] = ground[1 * W + 6] = 4         # the ledge, yellow
for column in range(W):
    ground[4 * W + column] = 1                    # the floor, red...
ground[4 * W + 3] = 2                             # ...with a green tile
ground[4 * W + 4] = 2 | MIRRORED                  # ...and a mirrored one
decor = [0] * (W * H)
decor[2 * W + 2] = 3

write_json("level.tmj", {
    "compressionlevel": -1, "height": H, "infinite": False, "orientation": "orthogonal", "renderorder": "right-down",
    "tiledversion": "1.11.0", "tileheight": 16, "tilewidth": 16, "type": "map", "version": "1.10", "width": W,
    "nextlayerid": 3, "nextobjectid": 1,
    "layers": [
        {"id": 1, "name": "ground", "type": "tilelayer", "width": W, "height": H, "x": 0, "y": 0, "opacity": 1,
         "visible": True, "data": ground},
        {"id": 2, "name": "decor", "type": "tilelayer", "width": W, "height": H, "x": 0, "y": 0, "opacity": 1,
         "visible": True, "data": decor},
    ],
    "tilesets": [{"firstgid": 1, "source": "tiles.tsj"}],
})
