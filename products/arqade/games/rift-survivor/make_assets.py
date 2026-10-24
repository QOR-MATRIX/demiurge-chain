"""Writes Rift Survivor's sky and sounds into assets/ (DIRECTION P7.20). Standard library only: python make_assets.py

  rift.hdr      512 x 256, an equirectangular HDR sky: deep teal-black, faint spiral arms of teal light turning out from
                above, and an ember glow at their heart, as the website's backdrop had. Run-length encoded, as tools
                write Radiance files: Qt misreads one written plainly (QQ's README, "Materials").
  shot.wav      0.08 s  a volley: a soft, high tick.
  kill.wav      0.18 s  an enemy destroyed: a bright chirp, falling.
  hit.wav       0.6 s   the hull breached: a low thud under a burst of noise.
  pulse.wav     1.35 s  a rift pulse opening: a hum rising for as long as it takes to arm.
  fragment.wav  0.12 s  a fragment collected: a tiny glassy blip.
  evolve.wav    0.9 s   the signal evolved: a rising run of bells.
  perk.wav      0.4 s   a perk chosen: two bells, a fifth apart.
  complete.wav  1.6 s   the run survived: bells climbing D minor to the octave.
  lost.wav      1.3 s   the hull broken: three low tones falling.
  run.wav       8 s     the music, looping at 120 beats a minute: a dark pulse through D, B flat, F and C under a pad.

A sound the game plays from a place is written in one channel, as Qt's spatial sounds take it, and one heard everywhere
in both, as its ambient sounds take it: each warns of every other. The noise is a fixed sequence, so the files are the
same bytes each time.
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


DEEP, TEAL, EMBER = (0.004, 0.008, 0.011), (0.010, 0.028, 0.032), (0.050, 0.018, 0.008)


def rift(x, y, w=512, h=256):
    latitude = math.pi / 2 - (y + 0.5) / h * math.pi
    longitude = (x + 0.5) / w * 2 * math.pi
    # Seen from inside, the overhead pole is the spiral's heart: r grows from it toward the horizon and beyond.
    r = (math.pi / 2 - latitude) / math.pi
    arms = max(0.0, math.sin(longitude * 5 + r * 18)) ** 8
    rings = max(0.0, math.sin(r * 40)) ** 12
    heart = 0.02 / (r + 0.05)
    return [DEEP[i] + TEAL[i] * (arms * 0.8 + rings * 0.3) * (1 - r) + EMBER[i] * heart for i in range(3)]


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


def fade_end(out, seconds=0.02):
    for i in range(int(seconds * RATE)):
        out[-1 - i] *= i / (seconds * RATE)
    return out


class Noise:
    """A fixed sequence of noise, low-passed by `smooth` (0 none, toward 1 more)."""

    def __init__(self, seed):
        self.seed, self.low = seed, 0.0

    def next(self, smooth=0.0):
        self.seed = (self.seed * 1103515245 + 12345) & 0x7FFFFFFF
        raw = self.seed / 0x3FFFFFFF - 1.0
        self.low += (1 - smooth) * (raw - self.low)
        return self.low


def bell(f, seconds):
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        attack = min(1.0, t / 0.003)
        out.append(attack * (math.sin(2 * math.pi * f * t) * math.exp(-t * 5.0)
                             + 0.35 * math.sin(2 * math.pi * 2 * f * t) * math.exp(-t * 8)
                             + 0.18 * math.sin(2 * math.pi * 2.76 * f * t) * math.exp(-t * 12)))
    return out


def sweep(f_start, f_end, seconds, decay, shape="sine"):
    out, phase = [], 0.0
    for i in range(int(seconds * RATE)):
        t = i / RATE
        f = f_start + (f_end - f_start) * (t / seconds)
        phase += 2 * math.pi * f / RATE
        wave_ = math.sin(phase) if shape == "sine" else (2 / math.pi) * math.asin(math.sin(phase))
        out.append(wave_ * math.exp(-t * decay) * min(1.0, t / 0.003))
    return out


def shot():
    return fade_end(sweep(2400, 1800, 0.08, 40, "triangle"), 0.01)


def kill():
    return fade_end(sweep(1400, 700, 0.18, 18, "triangle"))


def hit():
    out = sweep(110, 45, 0.6, 5)
    noise = Noise(0x2468ACE)
    for i in range(len(out)):
        out[i] += 0.9 * noise.next(0.6) * math.exp(-i / RATE * 9)
    return fade_end(out)


def pulse():
    out = []
    for i in range(int(1.35 * RATE)):
        t = i / RATE
        f = 120 + 160 * (t / 1.35) ** 2
        out.append(math.sin(2 * math.pi * f * t) * (0.2 + 0.8 * t / 1.35) * (0.6 + 0.4 * math.sin(2 * math.pi * 9 * t)))
    return fade_end(out, 0.04)


def fragment():
    return fade_end(sweep(2600, 3200, 0.12, 25))


def evolve():
    out = [0.0] * int(0.9 * RATE)
    for k, n in enumerate(("D5", "F5", "A5", "D6")):
        add(out, int(k * 0.08 * RATE), [0.7 * s for s in bell(note(n), 0.9 - k * 0.08)])
    return fade_end(out)


def perk():
    out = [0.0] * int(0.4 * RATE)
    add(out, 0, bell(note("A5"), 0.4))
    add(out, int(0.05 * RATE), [0.8 * s for s in bell(note("E6"), 0.35)])
    return fade_end(out)


def complete():
    out = [0.0] * int(1.6 * RATE)
    for k, n in enumerate(("D5", "F5", "A5", "D6", "F6")):
        add(out, int(k * 0.11 * RATE), [0.8 * s for s in bell(note(n), 1.6 - k * 0.11)])
    return fade_end(out)


def lost():
    out = [0.0] * int(1.3 * RATE)
    for k, f in enumerate((165, 139, 104)):
        add(out, int(k * 0.24 * RATE), sweep(f, f * 0.96, 0.7, 4))
    return fade_end(out)


def pad(f, seconds, release):
    out = []
    for i in range(int((seconds + release) * RATE)):
        t = i / RATE
        rise = min(1.0, t / 0.6)
        fall = 1.0 if t < seconds else max(0.0, 1 - (t - seconds) / release)
        out.append(rise * fall * (math.sin(2 * math.pi * f * 0.997 * t) + math.sin(2 * math.pi * f * 1.003 * t)))
    return out


def pluck(f, seconds):
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        out.append(min(1.0, t / 0.002) * math.exp(-t * 8) * (math.sin(2 * math.pi * f * t)
                                                             + 0.3 * math.sin(2 * math.pi * 2 * f * t)))
    return out


BASS = ["D2", "Bb1", "F1", "C2"]
CHORDS = [("D3", "F3", "A3"), ("Bb2", "D3", "F3"), ("F2", "A2", "C3"), ("C3", "E3", "G3")]


def music():
    beat = RATE // 2
    loop = [0.0] * (16 * beat)
    noise = Noise(0x5EED5)
    for bar in range(4):
        for half in range(8):
            add(loop, bar * 4 * beat + half * beat // 2, [0.5 * s for s in pluck(note(BASS[bar]), 0.35)], wrap=True)
        for n in CHORDS[bar]:
            add(loop, bar * 4 * beat, [0.1 * s for s in pad(note(n), 2.0, 0.5)], wrap=True)
        for b in range(4):
            tick = [0.15 * noise.next(0.2) * math.exp(-i / RATE * 70) for i in range(int(0.04 * RATE))]
            add(loop, (bar * 4 + b) * beat + beat // 2, tick, wrap=True)
    return loop


if __name__ == "__main__":
    ASSETS.mkdir(exist_ok=True)
    hdr("rift.hdr", 512, 256, rift)
    write("shot.wav", shot(), 0.25, placed=True)
    write("kill.wav", kill(), 0.45, placed=True)
    write("hit.wav", hit(), 0.7, placed=True)
    write("pulse.wav", pulse(), 0.4, placed=True)
    write("fragment.wav", fragment(), 0.3, placed=True)
    write("evolve.wav", evolve(), 0.6)
    write("perk.wav", perk(), 0.6)
    write("complete.wav", complete(), 0.7)
    write("lost.wav", lost(), 0.6)
    write("run.wav", music(), 0.45)
    for name in ("rift.hdr", "shot.wav", "kill.wav", "hit.wav", "pulse.wav", "fragment.wav", "evolve.wav", "perk.wav",
                 "complete.wav", "lost.wav", "run.wav"):
        print(name, (ASSETS / name).stat().st_size, "bytes")
