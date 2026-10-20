"""Writes Ember Climb's art and level (DIRECTION P3.10): pixel art drawn here, and the level as a Tiled map.

  assets/tiles.png   16-pixel tiles, 4 x 2: rock, rock with a lit top, planks, a cracked rock, and four dark tiles for
                     the wall behind (two bricks, a grate glowing from the forge, a chain)
  assets/tiles.tsj   the tileset, as Tiled saves one
  assets/level.tmj   the level, 48 x 16 tiles: "back" (the wall behind, drawn, never touched) and "ground" (what the
                     hero stands on and runs into)
  assets/hero.png    the hero, 16-pixel frames, 4 x 2: idle twice, jump, fall; then four frames of running
  assets/ember.png   an ember turning, 8-pixel frames, 4 x 1
  assets/torch.png   a torch on the wall, 8 x 16 frames, 4 x 1

Open level.tmj in Tiled (https://www.mapeditor.org) to change the level, or change LEVEL here and run this again.
Standard library only: python make_art.py
"""
import json
import pathlib
import struct
import zlib

HERE = pathlib.Path(__file__).parent
ASSETS = HERE / "assets"


def png(width, height, pixel):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    raw = b"".join(bytes([0]) + b"".join(bytes(pixel(x, y)) for x in range(width)) for y in range(height))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def rgb(text):
    return (int(text[1:3], 16), int(text[3:5], 16), int(text[5:7], 16), 255)


CLEAR = (0, 0, 0, 0)


class Noise:
    """The same speckles every run: a small linear congruential generator."""

    def __init__(self, seed):
        self.state = seed

    def next(self):
        self.state = (self.state * 1103515245 + 12345) & 0x7FFFFFFF
        return self.state / 0x7FFFFFFF


# ── tiles ──

ROCK = [rgb("#241d2e"), rgb("#2c2438"), rgb("#352b43"), rgb("#1b1622")]
LIT = [rgb("#ffd27a"), rgb("#c9a45c"), rgb("#8a6a3a")]
WOOD = [rgb("#6b4a2b"), rgb("#7d5733"), rgb("#4a321d"), rgb("#2e1f12")]
BACK = [rgb("#1a1522"), rgb("#201a29"), rgb("#110d15"), rgb("#272031")]
EMBER = [rgb("#ff6a00"), rgb("#ff9a2e"), rgb("#7a2a08")]


def rock(x, y, noise):
    v = noise.next()
    if v < 0.08:
        return ROCK[3]
    if v < 0.22:
        return ROCK[2]
    if v < 0.55:
        return ROCK[1]
    return ROCK[0]


def tile_rock(seed):
    noise = Noise(seed)
    grid = [[rock(x, y, noise) for x in range(16)] for y in range(16)]
    return lambda x, y: grid[y][x]


def tile_rock_top(seed):
    base = tile_rock(seed)

    def pixel(x, y):
        if y == 0:
            return LIT[0] if x % 5 else LIT[1]
        if y == 1:
            return LIT[1]
        if y == 2:
            return LIT[2] if (x * 7) % 3 else ROCK[3]
        return base(x, y)
    return pixel


def tile_planks(x, y):
    if y in (0, 15):
        return WOOD[3]
    if y in (5, 10):
        return WOOD[2]
    if x in (2, 13) and y in (2, 7, 12):
        return rgb("#b8a27a")  # nails
    return WOOD[1] if (x + y // 5 * 3) % 7 == 0 else WOOD[0]


def tile_cracked(seed):
    base = tile_rock(seed)
    crack = {(3, 2), (4, 3), (5, 4), (5, 5), (6, 6), (7, 6), (8, 7), (9, 8), (9, 9), (10, 10), (11, 11), (12, 11)}

    def pixel(x, y):
        if (x, y) in crack:
            return EMBER[2]
        return base(x, y)
    return pixel


def tile_brick(offset, seed):
    noise = Noise(seed)
    grid = [[BACK[1] if noise.next() < 0.3 else BACK[0] for _ in range(16)] for _ in range(16)]

    def pixel(x, y):
        if y % 8 == 7:
            return BACK[2]
        if (x + (offset if y // 8 % 2 else 0)) % 8 == 7:
            return BACK[2]
        return grid[y][x]
    return pixel


def tile_grate(x, y):
    if x in (0, 15) or y in (0, 15):
        return BACK[3]
    if x % 4 == 1:
        return BACK[3]
    # The forge's light through the bars, brightest low down.
    return EMBER[0] if y > 10 else EMBER[1] if y > 6 else EMBER[2]


def tile_chain(x, y):
    link = y % 6
    if x in (7, 8) and link in (0, 5):
        return rgb("#3a3444")
    if x in (6, 9) and link in (1, 2, 3, 4):
        return rgb("#4a4356")
    return BACK[0] if (x + y) % 9 else BACK[1]


TILES = [tile_rock(11), tile_rock_top(23), tile_planks, tile_cracked(37),
         tile_brick(4, 41), tile_brick(0, 53), tile_grate, tile_chain]
ROCK_FILL, ROCK_TOP, PLANKS, CRACKED, BRICK_A, BRICK_B, GRATE, CHAIN = (i + 1 for i in range(8))


def tiles_pixel(x, y):
    return TILES[(y // 16) * 4 + x // 16](x % 16, y % 16)


# ── the level ──

W, H = 48, 16
# Floating platforms of planks: (row, first column, last column). A row's top is 16 - row units up.
PLATFORMS = [(11, 5, 9), (8, 12, 16), (5, 15, 20), (11, 22, 28), (8, 30, 34), (5, 24, 29), (5, 36, 41), (3, 43, 46)]
GAPS = [range(11, 14), range(25, 27), range(37, 40)]


def build_level():
    solid = [[0] * W for _ in range(H)]
    for c in range(W):
        solid[0][c] = 1                       # the roof
        if not any(c in g for g in GAPS):
            solid[14][c] = solid[15][c] = 1   # the floor
    for r in range(H):
        solid[r][0] = solid[r][W - 1] = 1     # the walls
    for c in (19, 20):
        solid[12][c] = solid[13][c] = 1       # a step
    ground = [[0] * W for _ in range(H)]
    for r in range(H):
        for c in range(W):
            if solid[r][c]:
                open_above = r > 0 and not solid[r - 1][c]
                ground[r][c] = ROCK_TOP if open_above else CRACKED if (r * 31 + c * 17) % 23 == 0 else ROCK_FILL
    for r, c0, c1 in PLATFORMS:
        for c in range(c0, c1 + 1):
            ground[r][c] = PLANKS
    back = [[0] * W for _ in range(H)]
    noise = Noise(97)
    for r in range(1, 14):
        for c in range(1, W - 1):
            back[r][c] = BRICK_B if noise.next() < 0.3 else BRICK_A
    for c in (8, 27, 40):
        back[12][c] = back[13][c] = GRATE
    for c in (17, 33):
        for r in range(1, 5):
            back[r][c] = CHAIN
    return [v for row in back for v in row], [v for row in ground for v in row]


# ── the hero, facing right: 16 x 16 frames ──

HERO_PALETTE = {".": CLEAR, "k": rgb("#0d0b12"), "c": rgb("#3a4152"), "C": rgb("#59627a"), "g": rgb("#c9a45c"),
                "o": rgb("#ff6a00"), "y": rgb("#ffd27a"), "b": rgb("#1d1a24")}

UPPER = [
    "................",
    ".....kkkkkk.....",
    "....kcccccck....",
    "...kcCcccccck...",
    "...kcCckkkkkk...",
    "...kcckooyyyk...",
    "...kcCckkkkkk...",
    "...kcccccccck...",
    "..kgcccccccgk...",
    "..kcgccccccgck..",
    "..kccgggggggck..",
    "...kccccccck....",
]
LEGS = {
    "stand": ["...kccckccck....", "...kcck.kcck....", "...kbbk.kbbk....", "...kkkk.kkkk...."],
    "stride": ["...kcccccck.....", "..kcck..kcck....", ".kbbk....kbbk...", ".kkk......kkk...."[:16]],
    "pass": ["...kccccck......", "....kcckck......", "....kbbbbk......", "....kkkkkk......"],
    "stride2": ["...kcccccck.....", "...kcck.kcck....", "..kbbk...kbbk...", "..kkk.....kkk..."],
    "tuck": ["...kcccccck.....", "....kbbkbbk.....", "....kkkkkkk.....", "................"],
}


def hero_frame(legs, sink=0, lean=0):
    upper = ["." * 16] * sink + UPPER[:len(UPPER) - sink]
    if lean:
        upper = [row[-lean:] + row[:-lean] if i < 8 else row for i, row in enumerate(upper)]
    rows = upper + LEGS[legs]
    assert len(rows) == 16 and all(len(r) == 16 for r in rows), legs
    return rows


HERO = [hero_frame("stand"), hero_frame("stand", sink=1), hero_frame("tuck"), hero_frame("stride2"),
        hero_frame("stride", lean=1), hero_frame("pass", lean=1), hero_frame("stride2", lean=1), hero_frame("pass", lean=1)]


def sheet_pixel(frames, palette, fw, fh, columns):
    def pixel(x, y):
        frame = frames[(y // fh) * columns + x // fw]
        return palette[frame[y % fh][x % fw]]
    return pixel


# ── the ember turning, and a torch: 8-pixel frames ──

EMBER_PALETTE = {".": CLEAR, "R": rgb("#fff1c8"), "r": rgb("#ffd27a"), "o": rgb("#ff9a2e"), "y": rgb("#c9480f")}
EMBER_FRAMES = [
    ["...yy...", "..yooy..", ".yorroy.", "yorRRroy", "yorRRroy", ".yorroy.", "..yooy..", "...yy..."],
    ["...yy...", "...oy...", "..yrro..", "..orRo..", "..oRro..", "..orry..", "...yo...", "...yy..."],
    ["...yy...", "...ry...", "...Ro...", "...Rr...", "...rR...", "...oR...", "...yr...", "...yy..."],
    ["...yy...", "...yo...", "..orry..", "..oRro..", "..orRo..", "..yrro..", "...oy...", "...yy..."],
]

TORCH_PALETTE = {".": CLEAR, "k": rgb("#0d0b12"), "m": rgb("#4a4356"), "w": rgb("#5a3a22"), "R": rgb("#fff1c8"),
                 "r": rgb("#ffd27a"), "o": rgb("#ff9a2e"), "y": rgb("#ff6a00")}
FLAMES = [
    ["...y....", "..yoy...", "..yry...", ".yorry..", ".yrRry..", "..orRo..", "..yRry.."],
    ["....y...", "...yoy..", "..yory..", "..yrRo..", ".yorRry.", ".yoRRoy.", "..yrry.."],
    ["..y.....", "..yy....", ".yory...", ".yrrRy..", "..orRry.", ".yoRRoy.", "..yrRy.."],
    ["...yy...", "..yooy..", "..yrRy..", ".yorry..", ".yrRRry.", "..oRRo..", "..yrry.."],
]
TORCH_BASE = ["..kmmk..", "..kwwk..", "..kwwk..", "...kk...", "..kmmk..", "..kmmk..", ".kmmmmk..", "..kkkk.."]


def torch_frame(i):
    rows = ["........"] + FLAMES[i] + [r[:8] for r in TORCH_BASE]
    assert len(rows) == 16 and all(len(r) == 8 for r in rows)
    return rows


TORCH = [torch_frame(i) for i in range(4)]


def write_json(name, value, rows=0):
    """Indented, with a layer's tiles one map row to a line (`rows` wide), so a change to the level reads as one."""
    layers = value.get("layers", []) if rows else []
    datas = [layer["data"] for layer in layers]
    for i, layer in enumerate(layers):
        layer["data"] = f"@data{i}@"
    text = json.dumps(value, indent=1)
    for i, data in enumerate(datas):
        lines = [", ".join(str(v) for v in data[j:j + rows]) for j in range(0, len(data), rows)]
        text = text.replace(f'"@data{i}@"', "[\n    " + ",\n    ".join(lines) + "\n   ]")
        layers[i]["data"] = data
    (ASSETS / name).write_text(text + "\n", encoding="utf-8", newline="\n")


def main():
    ASSETS.mkdir(exist_ok=True)
    (ASSETS / "tiles.png").write_bytes(png(64, 32, tiles_pixel))
    (ASSETS / "hero.png").write_bytes(png(64, 32, sheet_pixel(HERO, HERO_PALETTE, 16, 16, 4)))
    (ASSETS / "ember.png").write_bytes(png(32, 8, sheet_pixel(EMBER_FRAMES, EMBER_PALETTE, 8, 8, 4)))
    (ASSETS / "torch.png").write_bytes(png(32, 16, sheet_pixel(TORCH, TORCH_PALETTE, 8, 16, 4)))
    write_json("tiles.tsj", {
        "columns": 4, "image": "tiles.png", "imageheight": 32, "imagewidth": 64, "margin": 0, "name": "forge",
        "spacing": 0, "tilecount": 8, "tileheight": 16, "tilewidth": 16, "type": "tileset", "version": "1.10",
    })
    back, ground = build_level()
    layer = lambda id, name, data: {"id": id, "name": name, "type": "tilelayer", "width": W, "height": H, "x": 0,
                                    "y": 0, "opacity": 1, "visible": True, "data": data}
    write_json("level.tmj", {
        "compressionlevel": -1, "height": H, "infinite": False, "orientation": "orthogonal",
        "renderorder": "right-down", "tiledversion": "1.11.0", "tileheight": 16, "tilewidth": 16, "type": "map",
        "version": "1.10", "width": W, "nextlayerid": 3, "nextobjectid": 1,
        "layers": [layer(1, "back", back), layer(2, "ground", ground)],
        "tilesets": [{"firstgid": 1, "source": "tiles.tsj"}],
    }, rows=W)


if __name__ == "__main__":
    main()
