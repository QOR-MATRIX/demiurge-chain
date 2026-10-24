"""Writes Orbital's sky and sounds into assets/ (DIRECTION P7.20). Standard library only: python make_assets.py

  space.hdr     1024 x 512, an equirectangular HDR sky: the void, deep blue toward the zenith, and a faint ember band of
                dust along the galaxy's plane, smooth, since a sky image's pixels show as blocks this close. The stars
                are points the game draws (logic/orbital.qml), sharp at any size. Run-length encoded, as tools write
                Radiance files: Qt misreads one written plainly (QQ's README, "Materials").
  lock.wav      0.9 s   a lock holds: two bell strikes a fourth apart, rising.
  miss.wav      0.5 s   a lock misses: a low tone falling under a soft rush.
  complete.wav  1.6 s   the orbit completes: five bells climbing the chord.
  lost.wav      1.2 s   the orbit is lost: three low tones falling.
  orbit.wav     12 s    the music, looping: a slow, wide pad through D minor, B flat, F and C.

The noise is a fixed sequence, so the files are the same bytes each time.
"""
import math
import pathlib
import struct
import wave

HERE = pathlib.Path(__file__).parent
ASSETS = HERE / "assets"
RATE = 22050


# The sky ------------------------------------------------------------------------------------------------------------

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


ZENITH, VOID = (0.006, 0.010, 0.030), (0.002, 0.002, 0.004)
EMBER, GILT = (0.060, 0.022, 0.006), (0.030, 0.024, 0.016)


def space(x, y, w=1024, h=512):
    latitude = math.pi / 2 - (y + 0.5) / h * math.pi
    longitude = (x + 0.5) / w * 2 * math.pi
    up = max(0.0, math.sin(latitude))
    colour = [VOID[i] + (ZENITH[i] - VOID[i]) * up for i in range(3)]
    # The galaxy's plane, tilted across the sky: a band of warm dust, brighter toward its heart, in soft clouds.
    band = math.sin(latitude) - 0.35 * math.cos(latitude) * math.sin(longitude)
    clouds = 0.7 + 0.3 * math.sin(longitude * 5 + 1.3) * math.sin(longitude * 3 - band * 9)
    dust = math.exp(-(band / 0.16) ** 2) * (0.55 + 0.45 * math.cos(longitude - 2.2)) * clouds
    return [colour[i] + EMBER[i] * dust + GILT[i] * dust * dust for i in range(3)]


# The sounds ---------------------------------------------------------------------------------------------------------

def note(name):
    """A note's frequency from its name and octave: A4 is 440; "Bb" is B flat."""
    steps = {"C": -9, "D": -7, "E": -5, "F": -4, "G": -2, "A": 0, "B": 2}[name[0]]
    steps += 1 if "#" in name else -1 if "b" in name[1:-1] else 0
    return 440.0 * 2 ** ((steps + 12 * (int(name[-1]) - 4)) / 12)


def write(name, samples, peak=0.6):
    """Scaled so its loudest sample is `peak` of full scale, and written 16-bit, the same in both channels: Qt's ambient
    sounds take stereo as it is and warn of every mono sound they spread."""
    top = max(abs(s) for s in samples) or 1.0
    scale = peak * 32767 / top
    frames = [int(round(s * scale)) for s in samples for _ in (0, 1)]
    with wave.open(str(ASSETS / name), "wb") as w:
        w.setnchannels(2)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(struct.pack("<%dh" % len(frames), *frames))


def add(buffer, start, sound, wrap=False):
    """Mix `sound` into `buffer` from sample `start`; past the end it is laid over the start when the buffer loops."""
    n = len(buffer)
    for i, s in enumerate(sound):
        at = start + i
        if at >= n:
            if not wrap:
                break
            at %= n
        buffer[at] += s


def fade_end(out, seconds=0.04):
    for i in range(int(seconds * RATE)):
        out[-1 - i] *= i / (seconds * RATE)
    return out


def bell(f, seconds):
    """A struck bell: its partials (1, 2 and 2.76 times the note), the higher ones dying sooner."""
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        attack = min(1.0, t / 0.003)
        out.append(attack * (math.sin(2 * math.pi * f * t) * math.exp(-t * 4.0)
                             + 0.35 * math.sin(2 * math.pi * 2 * f * t) * math.exp(-t * 7)
                             + 0.18 * math.sin(2 * math.pi * 2.76 * f * t) * math.exp(-t * 11)))
    return out


def tone(f_start, f_end, seconds, decay):
    """A sine sliding from one pitch to another, dying away."""
    out, phase = [], 0.0
    for i in range(int(seconds * RATE)):
        t = i / RATE
        f = f_end + (f_start - f_end) * math.exp(-t * 6)
        phase += 2 * math.pi * f / RATE
        out.append(math.sin(phase) * math.exp(-t * decay) * min(1.0, t / 0.004))
    return out


def lock():
    out = [0.0] * int(0.9 * RATE)
    add(out, 0, bell(note("A5"), 0.9))
    add(out, int(0.06 * RATE), [0.85 * s for s in bell(note("D6"), 0.84)])
    return fade_end(out)


def miss():
    out = tone(220, 110, 0.5, 6)
    seed, low = 0x1234567, 0.0
    for i in range(len(out)):
        seed = (seed * 1103515245 + 12345) & 0x7FFFFFFF
        low += 0.05 * (seed / 0x3FFFFFFF - 1.0 - low)
        out[i] += 0.8 * low * math.exp(-i / RATE * 8)
    return fade_end(out)


def complete():
    out = [0.0] * int(1.6 * RATE)
    for k, n in enumerate(("D5", "F5", "A5", "D6", "F6")):
        add(out, int(k * 0.11 * RATE), [0.8 * s for s in bell(note(n), 1.6 - k * 0.11)])
    return fade_end(out)


def lost():
    out = [0.0] * int(1.2 * RATE)
    for k, f in enumerate((196, 165, 131)):
        add(out, int(k * 0.22 * RATE), tone(f, f * 0.97, 0.7, 4))
    return fade_end(out)


def pad(f, seconds, release):
    """A soft, wide held note: two voices a hair apart, and the fifth and octave quietly."""
    out = []
    for i in range(int((seconds + release) * RATE)):
        t = i / RATE
        rise = min(1.0, t / 1.2)
        fall = 1.0 if t < seconds else max(0.0, 1 - (t - seconds) / release)
        out.append(rise * fall * (math.sin(2 * math.pi * f * 0.998 * t) + math.sin(2 * math.pi * f * 1.002 * t)
                                  + 0.25 * math.sin(2 * math.pi * 1.5 * f * t) + 0.2 * math.sin(2 * math.pi * 2 * f * t)))
    return out


CHORDS = [("D3", "A3", "F4"), ("Bb2", "F3", "D4"), ("F2", "C4", "A4"), ("C3", "G3", "E4")]


def orbit():
    loop = [0.0] * (12 * RATE)
    for c, chord in enumerate(CHORDS):
        for n in chord:
            add(loop, c * 3 * RATE, [0.4 * s for s in pad(note(n), 3.0, 1.5)], wrap=True)
    return loop


if __name__ == "__main__":
    ASSETS.mkdir(exist_ok=True)
    hdr("space.hdr", 1024, 512, space)
    write("lock.wav", lock(), 0.7)
    write("miss.wav", miss(), 0.6)
    write("complete.wav", complete(), 0.7)
    write("lost.wav", lost(), 0.6)
    write("orbit.wav", orbit(), 0.45)
    for name in ("space.hdr", "lock.wav", "miss.wav", "complete.wav", "lost.wav", "orbit.wav"):
        print(name, (ASSETS / name).stat().st_size, "bytes")
