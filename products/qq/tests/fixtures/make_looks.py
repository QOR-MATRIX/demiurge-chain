"""Writes the fixtures for materials' checks (DIRECTION P3.14). Standard library only: python make_looks.py

  checker.png   2 x 2: red and blue squares, a checker, so its copies can be counted and its edges seen sharp or soft.
  grey.png      1 x 1 mid-grey (128): a picture that, on a white shape, should draw as a plain shape of that grey does,
                so a pictured shape can be held against a plain one, its colours converted the same way.
  bumps.png     4 x 4 of one relief, every normal leaning toward +x, so the light picks the face out differently.
  sky.hdr       64 x 32, equirectangular: half the sky round a bright red, the other half a bright blue, both above 1
                (an HDR sky), so which half is ahead says how it is turned. The halves meet an eighth of the way round
                from the image's edge and middle, so no quarter direction lies on their edge, however the image is laid
                round. Written run-length encoded, as Radiance files are.
  dim.hdr       16 x 8 of one grey, 0.4, dimmer than white: a sky that bloom should leave as it is.
"""
import math
import pathlib
import struct
import zlib

HERE = pathlib.Path(__file__).parent


def png(name, w, h, pixel, alpha=False):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    raw = b"".join(bytes([0]) + b"".join(bytes(pixel(x, y)) for x in range(w)) for y in range(h))
    data = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6 if alpha else 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))
    (HERE / name).write_bytes(data)


def rgbe(r, g, b):
    v = max(r, g, b)
    if v < 1e-32:
        return (0, 0, 0, 0)
    m, e = math.frexp(v)
    scale = m * 256.0 / v
    return (int(r * scale), int(g * scale), int(b * scale), e + 128)


def runs(values):
    """One channel of a scanline, run-length encoded the Radiance way: (128 + n, v) for a run, (n, v...) for others."""
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
    (HERE / name).write_bytes(bytes(out))


RED, BLUE = (220, 40, 40), (40, 60, 220)
png("checker.png", 2, 2, lambda x, y: RED if (x + y) % 2 == 0 else BLUE)
png("grey.png", 1, 1, lambda x, y: (128, 128, 128))
# A normal leaning toward +x: (0.6, 0, 0.8), stored as (n + 1) / 2.
png("bumps.png", 4, 4, lambda x, y: (204, 128, 230))
hdr("sky.hdr", 64, 32, lambda x, y: (2.0, 0.08, 0.08) if 8 <= x < 40 else (0.08, 0.1, 2.0))
hdr("dim.hdr", 16, 8, lambda x, y: (0.4, 0.4, 0.4))
for name in ("checker.png", "grey.png", "bumps.png", "sky.hdr", "dim.hdr"):
    print(name, (HERE / name).stat().st_size, "bytes")
