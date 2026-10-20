"""Writes Orb Run's sounds into assets/ (DIRECTION P3.13): mono, 16-bit, 22,050 samples a second.

  chime.wav    1 s     an orb taken: two bell strikes a fifth apart, the second a breath after the first.
  rumble.wav   0.7 s   the boulder rolled: a low thump under a dull rush.
  title.wav    8 s     the title's music, looping: a slow pad through A minor, F, C and G.
  arena.wav    8 s     the arena's music, looping: the same chords at 120 beats a minute, with a plucked arpeggio and
                       a soft beat under it.

The music loops without a seam: what rings past the end of a loop is laid back over its start. The noise is a fixed
sequence, so the files are the same bytes each time. Standard library only: python make_sounds.py
"""
import math
import pathlib
import struct
import wave

HERE = pathlib.Path(__file__).parent
RATE = 22050


def note(name):
    """A note's frequency from its name and octave: A4 is 440."""
    steps = {"C": -9, "D": -7, "E": -5, "F": -4, "G": -2, "A": 0, "B": 2}[name[0]]
    sharp = 1 if "#" in name else 0
    octave = int(name[-1])
    return 440.0 * 2 ** ((steps + sharp + 12 * (octave - 4)) / 12)


def write(name, samples, peak=0.6):
    """Scaled so its loudest sample is `peak` of full scale, and written."""
    top = max(abs(s) for s in samples) or 1.0
    scale = peak * 32767 / top
    frames = struct.pack("<%dh" % len(samples), *(int(round(s * scale)) for s in samples))
    with wave.open(str(HERE / "assets" / name), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(frames)


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


def bell(f, seconds):
    """A struck bell: its partials (1, 2 and 2.76 times the note), the higher ones dying sooner."""
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        attack = min(1.0, t / 0.003)
        out.append(attack * (math.sin(2 * math.pi * f * t) * math.exp(-t * 4.5)
                             + 0.35 * math.sin(2 * math.pi * 2 * f * t) * math.exp(-t * 7)
                             + 0.18 * math.sin(2 * math.pi * 2.76 * f * t) * math.exp(-t * 11)))
    return out


def chime():
    out = [0.0] * RATE
    add(out, 0, bell(note("E6"), 1.0))
    add(out, int(0.07 * RATE), [0.8 * s for s in bell(note("B6"), 0.93)])
    # The last moment fades to nothing, so it ends without a click.
    for i in range(int(0.05 * RATE)):
        out[-1 - i] *= i / (0.05 * RATE)
    return out


def rumble():
    seconds = 0.7
    out = []
    seed = 0x2545F491
    low = 0.0
    for i in range(int(seconds * RATE)):
        t = i / RATE
        # A fixed sequence of noise (a linear congruential generator), made dull by a one-pole low-pass.
        seed = (seed * 1103515245 + 12345) & 0x7FFFFFFF
        noise = seed / 0x3FFFFFFF - 1.0
        low += 0.06 * (noise - low)
        rush = low * min(1.0, t / 0.02) * math.exp(-t * 4)
        # The thump: a sine falling from 90 to 45 cycles a second.
        f = 45 + 45 * math.exp(-t * 9)
        thump = math.sin(2 * math.pi * f * t) * math.exp(-t * 7) * min(1.0, t / 0.004)
        out.append(1.6 * rush + thump)
    for i in range(int(0.04 * RATE)):
        out[-1 - i] *= i / (0.04 * RATE)
    return out


CHORDS = [("A2", "C4", "E4", "A4"), ("F2", "A3", "C4", "F4"), ("C3", "E4", "G4", "C5"), ("G2", "B3", "D4", "G4")]


def pad(f, seconds, release):
    """A soft held note: two voices a hair apart, and the octave quietly, rising in and dying away."""
    out = []
    total = seconds + release
    for i in range(int(total * RATE)):
        t = i / RATE
        rise = min(1.0, t / 0.6)
        fall = 1.0 if t < seconds else max(0.0, 1 - (t - seconds) / release)
        tone = (math.sin(2 * math.pi * f * 0.9985 * t) + math.sin(2 * math.pi * f * 1.0015 * t)
                + 0.3 * math.sin(2 * math.pi * 2 * f * t))
        out.append(rise * fall * tone)
    return out


def pluck(f, seconds):
    """A plucked note: a triangle wave, dying quickly."""
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        phase = (f * t) % 1.0
        triangle = 4 * abs(phase - 0.5) - 1
        out.append(triangle * math.exp(-t * 9) * min(1.0, t / 0.002))
    return out


def kick(seconds=0.35):
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        f = 42 + 38 * math.exp(-t * 30)
        out.append(math.sin(2 * math.pi * f * t) * math.exp(-t * 12) * min(1.0, t / 0.002))
    return out


def title():
    loop = [0.0] * (8 * RATE)
    for c, chord in enumerate(CHORDS):
        for n in chord:
            add(loop, c * 2 * RATE, [0.5 * s for s in pad(note(n), 2.0, 0.8)], wrap=True)
    return loop


def arena():
    loop = [0.0] * (8 * RATE)
    beat = RATE // 2  # 120 beats a minute
    for c, chord in enumerate(CHORDS):
        for n in chord:
            add(loop, c * 2 * RATE, [0.3 * s for s in pad(note(n), 2.0, 0.5)], wrap=True)
        # Eighth notes up and down the chord's upper three notes, an octave above.
        upper = [note(n) * 2 for n in chord[1:]]
        order = [0, 1, 2, 1, 0, 1, 2, 1]
        for k, which in enumerate(order):
            add(loop, c * 2 * RATE + k * beat // 2, [0.45 * s for s in pluck(upper[which], 0.4)], wrap=True)
        for b in range(4):
            add(loop, c * 2 * RATE + b * beat, [0.8 * s for s in kick()], wrap=True)
    return loop


if __name__ == "__main__":
    (HERE / "assets").mkdir(exist_ok=True)
    write("chime.wav", chime(), 0.7)
    write("rumble.wav", rumble(), 0.7)
    write("title.wav", title(), 0.5)
    write("arena.wav", arena(), 0.5)
    for name in ("chime.wav", "rumble.wav", "title.wav", "arena.wav"):
        print(name, (HERE / "assets" / name).stat().st_size, "bytes")
