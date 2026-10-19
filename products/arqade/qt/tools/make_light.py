"""ARQADE's light, as two small images: the sky its gilt reflects (assets/sky.hdr) and an ember's halo (assets/halo.png).

Every 3D view in ARQADE lights its metal from this image. Each view once made its own sky with Qt's
ProceduralSkyTextureData, and six views doing so in one window crashed Qt 6.12 (heap corruption, three runs in three);
one image every view shares is cached once and was stable in five runs in five.

The sky is a dark vault warming to an ember horizon, a dim ground, and a low sun with a broad glow: the homepage's
obsidian and ember as light. It is an uncompressed Radiance RGBE file, 256 by 128, equirectangular.

The halo is the soft light around the portal's core, drawn on a plane that faces the eye and screened over the scene:
ember fading to nothing, its colour already multiplied by its fall-off so its edge adds no light. A 256-pixel PNG.

    python products/arqade/qt/tools/make_light.py
"""

import math
import struct
import zlib
from pathlib import Path

WIDTH, HEIGHT = 256, 128
SUN_LATITUDE, SUN_LONGITUDE = math.radians(22), math.radians(140)


def linear(hex_colour):
    """An sRGB colour as linear light."""
    channels = [int(hex_colour[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return [((c + 0.055) / 1.055) ** 2.4 if c > 0.04045 else c / 12.92 for c in channels]


SKY_TOP, SKY_HORIZON = linear("#05060a"), linear("#3a2410")
GROUND_HORIZON, GROUND_BOTTOM = linear("#1c1208"), linear("#020203")
SUN = [1.0, 0.70, 0.42]


def mix(a, b, t):
    return [a[i] + (b[i] - a[i]) * t for i in range(3)]


def rgbe(r, g, b):
    brightest = max(r, g, b)
    if brightest < 1e-32:
        return bytes(4)
    exponent = math.frexp(brightest)[1]
    scale = math.ldexp(1.0, -exponent) * 256
    return bytes([min(255, int(r * scale)), min(255, int(g * scale)), min(255, int(b * scale)), exponent + 128])


def sky():
    out = bytearray(b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y %d +X %d\n" % (HEIGHT, WIDTH))
    for y in range(HEIGHT):
        latitude = math.pi / 2 - (y + 0.5) / HEIGHT * math.pi
        for x in range(WIDTH):
            longitude = (x + 0.5) / WIDTH * 2 * math.pi
            if latitude >= 0:
                colour = mix(SKY_HORIZON, SKY_TOP, (latitude / (math.pi / 2)) ** 0.45)
            else:
                colour = mix(GROUND_HORIZON, GROUND_BOTTOM, (-latitude / (math.pi / 2)) ** 0.35)
            cosine = (math.sin(latitude) * math.sin(SUN_LATITUDE)
                      + math.cos(latitude) * math.cos(SUN_LATITUDE) * math.cos(longitude - SUN_LONGITUDE))
            away = math.acos(max(-1.0, min(1.0, cosine)))
            glow = math.exp(-(away / 0.5) ** 2) * 0.6 + (30.0 if away < 0.04 else 0.0)
            out += rgbe(*[colour[i] * 3.0 + SUN[i] * glow for i in range(3)])
    return bytes(out)


def halo(size=256):
    ember = (255, 122, 26)
    rows = bytearray()
    for y in range(size):
        rows.append(0)  # no filter
        for x in range(size):
            r = math.hypot((x + 0.5) / size * 2 - 1, (y + 0.5) / size * 2 - 1)
            # A bright inner glow and a long, faint reach.
            fall = min(1.0, 0.85 * math.exp(-(r / 0.22) ** 2) + 0.35 * math.exp(-(r / 0.55) ** 2)) * max(0.0, 1 - r)
            rows += bytes([round(c * fall) for c in ember] + [round(255 * fall)])

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    signature = bytes([0x89]) + b"PNG" + bytes([13, 10, 26, 10])
    return signature + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b"")


if __name__ == "__main__":
    assets = Path(__file__).resolve().parent.parent / "assets"
    for name, data in (("sky.hdr", sky()), ("halo.png", halo())):
        (assets / name).write_bytes(data)
        print(f"{assets / name}: {len(data)} bytes")
