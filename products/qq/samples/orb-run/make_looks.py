"""Writes Orb Run's surfaces and sky into assets/ (DIRECTION P3.14). Standard library only: python make_looks.py

  stone.png         128 x 128, one copy of the arena's wall: two courses of dressed stone blocks, staggered, each its own
                    shade, in pale mortar. A copy is 1.5 units in the scene, so a block is 0.75 units by 0.75.
  stone-normal.png  its relief: each block raised and bevelled, the mortar sunk, the faces lightly pitted.
  floor.png         128 x 128, one copy of the floor: four slate flags, each its own shade, in dark grout.
  dusk.hdr          512 x 256, an equirectangular HDR sky at dusk: peach at the horizon, violet above it, deep blue
                    overhead, amber toward the sun, and the sun itself, far brighter than white, where the scene's Sun
                    shines from.

The noise is a fixed sequence, so the files are the same bytes each time.
"""
import math
import pathlib
import struct
import zlib

HERE = pathlib.Path(__file__).parent
ASSETS = HERE / "assets"


def png(name, w, h, pixel):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    raw = b"".join(bytes([0]) + b"".join(bytes(pixel(x, y)) for x in range(w)) for y in range(h))
    data = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))
    (ASSETS / name).write_bytes(data)


def hash01(*n):
    """A fixed pseudo-random number in [0, 1) for whole numbers n."""
    h = 2166136261
    for v in n:
        h = ((h ^ (v & 0xFFFFFFFF)) * 16777619) & 0xFFFFFFFF
    h ^= h >> 13
    h = (h * 1274126177) & 0xFFFFFFFF
    return (h ^ (h >> 16)) / 4294967296.0


def noise(x, y, period, seed):
    """Smooth value noise that repeats every `period` cells, so a picture made of it tiles without a seam."""
    x0, y0 = math.floor(x), math.floor(y)
    fx, fy = x - x0, y - y0
    sx, sy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)

    def at(i, j):
        return hash01(i % period, j % period, seed)

    top = at(x0, y0) * (1 - sx) + at(x0 + 1, y0) * sx
    bottom = at(x0, y0 + 1) * (1 - sx) + at(x0 + 1, y0 + 1) * sx
    return top * (1 - sy) + bottom * sy


def grain(x, y, size, seed):
    """Two octaves of noise over a picture `size` pixels across, tiling."""
    return (0.65 * noise(x / size * 8, y / size * 8, 8, seed) + 0.35 * noise(x / size * 16, y / size * 16, 16, seed + 1))


# ── the wall: two courses of two blocks, the second course offset by half a block ──

S = 128
MORTAR = 3  # pixels each side of a joint


def stone_block(x, y):
    """Which block a pixel is in, and how far it is from that block's nearest edge, in pixels."""
    course = y // (S // 2)
    shift = (S // 4) if course % 2 else 0
    bx = ((x + shift) % S) // (S // 2)
    lx = (x + shift) % (S // 2)
    ly = y % (S // 2)
    edge = min(lx, S // 2 - 1 - lx, ly, S // 2 - 1 - ly)
    return course * 2 + bx, edge


def stone_height(x, y):
    block, edge = stone_block(x % S, y % S)
    if edge < MORTAR:
        return 0.0
    bevel = min(1.0, (edge - MORTAR) / 5.0)
    return 0.55 + 0.45 * bevel - 0.12 * grain(x, y, S, 7)


def stone(x, y):
    block, edge = stone_block(x, y)
    if edge < MORTAR:
        m = 150 + int(30 * grain(x, y, S, 3))
        return (m, m - 6, m - 14)
    shade = 0.82 + 0.3 * hash01(block, 11)
    g = 0.78 + 0.32 * grain(x, y, S, 5)
    base = (128, 118, 104)
    return tuple(max(0, min(255, int(c * shade * g))) for c in base)


def stone_normal(x, y):
    # The slope of the height, both ways, as a normal: (n + 1) / 2 in each channel.
    dx = stone_height(x + 1, y) - stone_height(x - 1, y)
    dy = stone_height(x, y + 1) - stone_height(x, y - 1)
    strength = 2.2
    nx, ny, nz = -dx * strength, dy * strength, 1.0
    length = math.sqrt(nx * nx + ny * ny + nz * nz)
    return tuple(int(round((c / length + 1) * 127.5)) for c in (nx, ny, nz))


# ── the floor: four slate flags ──

def floor(x, y):
    fx, fy = x // (S // 2), y // (S // 2)
    lx, ly = x % (S // 2), y % (S // 2)
    edge = min(lx, S // 2 - 1 - lx, ly, S // 2 - 1 - ly)
    if edge < 2:
        return (22, 22, 26)
    shade = 0.8 + 0.35 * hash01(fx, fy, 23)
    g = 0.8 + 0.3 * grain(x, y, S, 29)
    base = (74, 78, 88)
    return tuple(max(0, min(255, int(c * shade * g))) for c in base)


# ── the sky ──

def rgbe(r, g, b):
    v = max(r, g, b)
    if v < 1e-32:
        return (0, 0, 0, 0)
    m, e = math.frexp(v)
    scale = m * 256.0 / v
    return (int(r * scale), int(g * scale), int(b * scale), e + 128)


def runs(values):
    out = bytearray()
    i = 0
    while i < len(values):
        run = 1
        while i + run < len(values) and run < 127 and values[i + run] == values[i]:
            run += 1
        if run > 2:
            out += bytes([128 + run, values[i]])
            i += run
            continue
        start = i
        while i < len(values) and i - start < 128:
            if i + 2 < len(values) and values[i] == values[i + 1] == values[i + 2]:
                break
            i += 1
        out += bytes([i - start]) + bytes(values[start:i])
    return bytes(out)


def hdr(name, w, h, pixel):
    out = bytearray(b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y %d +X %d\n" % (h, w))
    for y in range(h):
        row = [rgbe(*pixel(x, y)) for x in range(w)]
        out += bytes([2, 2, w >> 8, w & 0xFF])
        for c in range(4):
            out += runs([p[c] for p in row])
    (ASSETS / name).write_bytes(bytes(out))


# The scene's Sun shines from elevation 38, azimuth -35; the sky's sun stands where that light comes from. Qt lays an
# equirectangular sky round with its middle column toward +x and a quarter of the way across toward -z.
SUN_ELEVATION, SUN_AZIMUTH = math.radians(38), math.radians(-35)
# Toward the sun: the way the Sun's light travels (Sun.qml's turn of -z), reversed.
SUN = (math.sin(SUN_AZIMUTH) * math.cos(SUN_ELEVATION), math.sin(SUN_ELEVATION),
       math.cos(SUN_AZIMUTH) * math.cos(SUN_ELEVATION))


HORIZON, VIOLET, ZENITH = (0.62, 0.32, 0.22), (0.20, 0.15, 0.30), (0.025, 0.045, 0.13)


def mix(a, b, t):
    return tuple(a[i] + (b[i] - a[i]) * t for i in range(3))


def sky(x, y, w=512, h=256):
    u = (x + 0.5) / w
    v = (y + 0.5) / h
    longitude = (u - 0.5) * 2 * math.pi
    latitude = (0.5 - v) * math.pi
    d = (math.cos(latitude) * math.cos(longitude), math.sin(latitude), math.cos(latitude) * math.sin(longitude))
    toward = max(0.0, d[0] * SUN[0] + d[1] * SUN[1] + d[2] * SUN[2])
    if latitude >= 0:
        # Peach at the horizon, violet a little above it, deep blue overhead.
        a = latitude / (math.pi / 2)
        colour = mix(HORIZON, VIOLET, min(1.0, a / 0.22)) if a < 0.22 else mix(VIOLET, ZENITH, ((a - 0.22) / 0.78) ** 0.6)
        # Amber toward the sun, a bright halo close round it, and the sun itself.
        colour = tuple(c + k * toward ** 5 for c, k in zip(colour, (1.3, 0.62, 0.22)))
        colour = tuple(c + k * toward ** 160 for c, k in zip(colour, (6.0, 4.2, 2.4)))
        if toward > math.cos(math.radians(1.6)):
            colour = (90.0, 70.0, 48.0)
    else:
        t = min(1.0, -latitude / (math.pi / 10))
        colour = mix((0.16, 0.09, 0.07), (0.010, 0.009, 0.012), t)
    return colour


if __name__ == "__main__":
    ASSETS.mkdir(exist_ok=True)
    png("stone.png", S, S, stone)
    png("stone-normal.png", S, S, stone_normal)
    png("floor.png", S, S, floor)
    hdr("dusk.hdr", 512, 256, sky)
    for name in ("stone.png", "stone-normal.png", "floor.png", "dusk.hdr"):
        print(name, (ASSETS / name).stat().st_size, "bytes")
