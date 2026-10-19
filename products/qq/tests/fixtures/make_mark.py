"""Writes mark.png: a 16 x 16 gilt diamond with an ember centre on clear, for the screen kinds' checks (P3.8).

Pixel art, so a ScreenImage draws it `crisp`. Standard library only: python make_mark.py
"""
import pathlib
import struct
import zlib

W = H = 16


def pixel(x, y):
    d = abs(x - 7.5) + abs(y - 7.5)
    if d <= 2.5:
        return (255, 106, 0, 255)
    if d <= 6.5:
        return (201, 164, 92, 255)
    if d <= 7.5:
        return (122, 98, 54, 255)
    return (0, 0, 0, 0)


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)


raw = b"".join(bytes([0]) + b"".join(bytes(pixel(x, y)) for x in range(W)) for y in range(H))
png = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 6, 0, 0, 0))
       + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))
pathlib.Path(__file__).with_name("mark.png").write_bytes(png)
