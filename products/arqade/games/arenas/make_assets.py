"""Writes the arenas' sky and sounds into assets/ (DIRECTION P7.24). Standard library only: python make_assets.py

  arena.hdr    512 x 256, an equirectangular HDR sky: a deep void, a faint band of navy and teal light round the
               horizon, and a warm glow low on one side. Run-length encoded, as tools write Radiance files: Qt misreads
               one written plainly (QQ's README, "Materials").
  drop.wav     0.5 s   a Flux Four pulse coming to rest: a soft low thud and a glass ping.
  place.wav    0.4 s   a Rift Reversi disc set down: a click on glass and its note.
  flip.wav     0.45 s  discs turning over: two quick rising sweeps.
  turn.wav     0.8 s   the player's move: two glass notes rising.
  joined.wav   1.2 s   a challenger arrives: a chord swelling, and a bell.
  won.wav      1.8 s   a match won: the notes of a chord climbing, held.
  over.wav     1.4 s   a match over otherwise: three notes falling, softly.
  arena.wav    16 s    the music, looping: a low, slow drone through D and B flat, quiet under the board.

A sound the game plays from a place is written in one channel, as Qt's spatial sounds take it, and one heard everywhere
in both, as its ambient sounds take it: each warns of every other. The files are the same bytes each time.
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


VOID = (0.002, 0.002, 0.004)
NAVY, TEAL, WARM = (0.006, 0.012, 0.030), (0.004, 0.024, 0.026), (0.036, 0.018, 0.008)


def arena(x, y, w=512, h=256):
    latitude = math.pi / 2 - (y + 0.5) / h * math.pi
    longitude = (x + 0.5) / w * 2 * math.pi
    up = math.sin(latitude)
    # A band round the horizon, its light rising and falling slowly round the sphere, so nothing is sharp.
    band = math.exp(-((up - 0.05) / 0.22) ** 2)
    a = 0.5 + 0.5 * math.sin(longitude * 2 + 0.7) * math.cos(longitude * 3 - up * 2.0)
    b = 0.5 + 0.5 * math.sin(longitude * 3 - 1.9 + up * 2.5)
    warm = math.exp(-((up + 0.1) / 0.25) ** 2) * (0.5 + 0.5 * math.cos(longitude - 2.4))
    return [VOID[i] + NAVY[i] * band * a + TEAL[i] * band * b * b + WARM[i] * warm for i in range(3)]


# The sounds ---------------------------------------------------------------------------------------------------------

def note(name):
    """A note's frequency from its name and octave: A4 is 440; "Bb" is B flat."""
    steps = {"C": -9, "D": -7, "E": -5, "F": -4, "G": -2, "A": 0, "B": 2}[name[0]]
    steps += 1 if "#" in name else -1 if "b" in name[1:-1] else 0
    return 440.0 * 2 ** ((steps + 12 * (int(name[-1]) - 4)) / 12)


def write(name, samples, peak=0.6, placed=False):
    """Scaled so its loudest sample is `peak` of full scale, and written 16-bit: in one channel if the game plays it
    from a place, otherwise the same in both."""
    top = max(abs(s) for s in samples) or 1.0
    scale = peak * 32767 / top
    frames = [int(round(s * scale)) for s in samples for _ in range(1 if placed else 2)]
    with wave.open(str(ASSETS / name), "wb") as w:
        w.setnchannels(1 if placed else 2)
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


def glass(f, seconds):
    """A struck glass bell: its note and its bright, quickly dying partials."""
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        attack = min(1.0, t / 0.004)
        out.append(attack * (math.sin(2 * math.pi * f * t) * math.exp(-t * 3.5)
                             + 0.4 * math.sin(2 * math.pi * 2.01 * f * t) * math.exp(-t * 6)
                             + 0.2 * math.sin(2 * math.pi * 3.98 * f * t) * math.exp(-t * 10)))
    return out


def thud(seconds):
    """A soft low thud: a sine falling from 140 to 60 Hz, gone in a moment."""
    out = []
    phase = 0.0
    for i in range(int(seconds * RATE)):
        t = i / RATE
        f = 60 + 80 * math.exp(-t * 30)
        phase += 2 * math.pi * f / RATE
        out.append(math.sin(phase) * math.exp(-t * 14) * min(1.0, t / 0.002))
    return out


def click(seconds):
    """A tap on glass: a few milliseconds of bright, falling partials, the same each time."""
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        out.append(math.exp(-t * 400) * (math.sin(2 * math.pi * 3100 * t) + 0.6 * math.sin(2 * math.pi * 4700 * t)
                                          + 0.4 * math.sin(2 * math.pi * 6900 * t)))
    return out


def sweep(f0, f1, seconds):
    """A tone gliding from f0 to f1 and fading."""
    out = []
    phase = 0.0
    for i in range(int(seconds * RATE)):
        t = i / RATE
        f = f0 + (f1 - f0) * min(1.0, t / (seconds * 0.6))
        phase += 2 * math.pi * f / RATE
        out.append(math.sin(phase) * math.sin(math.pi * min(1.0, t / seconds)) ** 2)
    return out


def drop():
    out = [0.0] * int(0.5 * RATE)
    add(out, 0, thud(0.4))
    add(out, int(0.01 * RATE), [0.35 * s for s in glass(note("A5"), 0.49)])
    return fade_end(out)


def place():
    out = [0.0] * int(0.4 * RATE)
    add(out, 0, [0.8 * s for s in click(0.05)])
    add(out, int(0.005 * RATE), [0.45 * s for s in glass(note("E5"), 0.39)])
    return fade_end(out)


def flip():
    out = [0.0] * int(0.45 * RATE)
    add(out, 0, sweep(520, 1250, 0.2))
    add(out, int(0.12 * RATE), [0.7 * s for s in sweep(640, 1500, 0.2)])
    return fade_end(out)


def chord(notes, seconds, gap, level=0.7):
    out = [0.0] * int(seconds * RATE)
    for k, n in enumerate(notes):
        add(out, int(k * gap * RATE), [level * s for s in glass(note(n), seconds - k * gap)])
    return fade_end(out)


def joined():
    out = [0.0] * int(1.2 * RATE)
    for n in ("D4", "F#4", "A4"):
        f = note(n)
        tone = []
        for i in range(int(0.9 * RATE)):
            t = i / RATE
            tone.append(math.sin(2 * math.pi * f * t) * math.sin(math.pi * t / 0.9) * 0.4)
        add(out, 0, tone)
    add(out, int(0.75 * RATE), [0.6 * s for s in glass(note("D6"), 0.45)])
    return fade_end(out)


def drone(f, seconds, release):
    """A low held note: two voices a hair apart, and its fifth faintly."""
    out = []
    for i in range(int((seconds + release) * RATE)):
        t = i / RATE
        rise = min(1.0, t / 2.0)
        fall = 1.0 if t < seconds else max(0.0, 1 - (t - seconds) / release)
        out.append(rise * fall * (math.sin(2 * math.pi * f * 0.998 * t) + math.sin(2 * math.pi * f * 1.002 * t)
                                  + 0.15 * math.sin(2 * math.pi * 1.5 * f * t)))
    return out


def music():
    loop = [0.0] * (16 * RATE)
    for k, n in enumerate(["D2", "Bb1"]):
        add(loop, k * 8 * RATE, drone(note(n), 8.0, 3.0), wrap=True)
    return loop


if __name__ == "__main__":
    ASSETS.mkdir(exist_ok=True)
    hdr("arena.hdr", 512, 256, arena)
    write("drop.wav", drop(), 0.6, placed=True)
    write("place.wav", place(), 0.6, placed=True)
    write("flip.wav", flip(), 0.5, placed=True)
    write("turn.wav", chord(["G5", "C6"], 0.8, 0.12), 0.5)
    write("joined.wav", joined(), 0.6)
    write("won.wav", chord(["D5", "F#5", "A5", "D6", "F#6"], 1.8, 0.12), 0.7)
    write("over.wav", chord(["G5", "E5", "C5"], 1.4, 0.16, 0.6), 0.55)
    write("arena.wav", music(), 0.35)
    for name in ["arena.hdr", "drop.wav", "place.wav", "flip.wav", "turn.wav", "joined.wav", "won.wav", "over.wav",
                 "arena.wav"]:
        print(name, (ASSETS / name).stat().st_size, "bytes")
