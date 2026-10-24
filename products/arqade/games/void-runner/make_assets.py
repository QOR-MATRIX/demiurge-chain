"""Writes Void Runner's sky and sounds into assets/ (DIRECTION P7.20). Standard library only: python make_assets.py

  collapse.hdr  1024 x 512, an equirectangular HDR sky: the void overhead, and all round the horizon the ember light of
                the collapse the track runs into, in slow bands. Run-length encoded, as tools write Radiance files: Qt
                misreads one written plainly (QQ's README, "Materials").
  gem.wav       0.5 s   a gem caught: two bright bell notes, a fifth apart.
  hit.wav       0.7 s   the hull breached: a low thud falling under a burst of noise.
  held.wav      0.3 s   a hazard met while the hull holds: a glancing, muffled tick.
  shift.wav     0.16 s  a change of lane: a short rush of air.
  complete.wav  1.6 s   the run survived: bells climbing A minor to the octave.
  lost.wav      1.3 s   the run ended: three low tones falling.
  run.wav       8 s     the music, looping at 120 beats a minute: a pulsing bass through A, F, C and G under a soft pad,
                        with a tick on each off-beat.

Every sound is written in both channels, as Qt's ambient sounds take it. The noise is a fixed sequence, so the files are
the same bytes each time.
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


VOID, ZENITH = (0.002, 0.002, 0.004), (0.004, 0.005, 0.016)
EMBER, FLARE = (0.090, 0.030, 0.008), (0.030, 0.008, 0.002)


def collapse(x, y, w=1024, h=512):
    latitude = math.pi / 2 - (y + 0.5) / h * math.pi
    longitude = (x + 0.5) / w * 2 * math.pi
    up = math.sin(latitude)
    if up < 0:
        # Below the horizon the track's floor hides the sky; a dim ember keeps reflections warm.
        return [VOID[i] + EMBER[i] * 0.25 * math.exp(up * 6) for i in range(3)]
    colour = [VOID[i] + (ZENITH[i] - VOID[i]) * up for i in range(3)]
    # The collapse's light, all round the horizon, brighter where its slow bands meet.
    bands = 0.75 + 0.25 * math.sin(longitude * 3 + 0.7) * math.sin(longitude * 5 - 1.1)
    glow = math.exp(-up / 0.09) * bands
    haze = math.exp(-up / 0.35) * 0.5
    return [colour[i] + EMBER[i] * glow + FLARE[i] * haze for i in range(3)]


# The sounds ---------------------------------------------------------------------------------------------------------

def note(name):
    """A note's frequency from its name and octave: A4 is 440; "Bb" is B flat."""
    steps = {"C": -9, "D": -7, "E": -5, "F": -4, "G": -2, "A": 0, "B": 2}[name[0]]
    steps += 1 if "#" in name else -1 if "b" in name[1:-1] else 0
    return 440.0 * 2 ** ((steps + 12 * (int(name[-1]) - 4)) / 12)


def write(name, samples, peak=0.6):
    """Scaled so its loudest sample is `peak` of full scale, and written 16-bit, the same in both channels."""
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


def fade_end(out, seconds=0.03):
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
    """A struck bell: its partials (1, 2 and 2.76 times the note), the higher ones dying sooner."""
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        attack = min(1.0, t / 0.003)
        out.append(attack * (math.sin(2 * math.pi * f * t) * math.exp(-t * 5.0)
                             + 0.35 * math.sin(2 * math.pi * 2 * f * t) * math.exp(-t * 8)
                             + 0.18 * math.sin(2 * math.pi * 2.76 * f * t) * math.exp(-t * 12)))
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


def gem():
    out = [0.0] * int(0.5 * RATE)
    add(out, 0, bell(note("E6"), 0.5))
    add(out, int(0.05 * RATE), [0.8 * s for s in bell(note("B6"), 0.45)])
    return fade_end(out)


def hit():
    out = tone(110, 42, 0.7, 5)
    noise = Noise(0x2468ACE)
    for i in range(len(out)):
        out[i] += 0.9 * noise.next(0.6) * math.exp(-i / RATE * 9)
    return fade_end(out)


def held():
    noise = Noise(0x13579BD)
    out = [noise.next(0.85) * math.exp(-i / RATE * 22) for i in range(int(0.3 * RATE))]
    return fade_end(out)


def shift():
    noise = Noise(0x0F1E2D3)
    n = int(0.16 * RATE)
    return fade_end([noise.next(0.5) * math.sin(math.pi * i / n) for i in range(n)], 0.01)


def complete():
    out = [0.0] * int(1.6 * RATE)
    for k, n in enumerate(("A4", "C5", "E5", "A5", "C6")):
        add(out, int(k * 0.1 * RATE), [0.8 * s for s in bell(note(n), 1.6 - k * 0.1)])
    return fade_end(out)


def lost():
    out = [0.0] * int(1.3 * RATE)
    for k, f in enumerate((175, 147, 110)):
        add(out, int(k * 0.24 * RATE), tone(f, f * 0.96, 0.7, 4))
    return fade_end(out)


def pad(f, seconds, release):
    """A soft, wide held note: two voices a hair apart, and the octave quietly."""
    out = []
    for i in range(int((seconds + release) * RATE)):
        t = i / RATE
        rise = min(1.0, t / 0.6)
        fall = 1.0 if t < seconds else max(0.0, 1 - (t - seconds) / release)
        out.append(rise * fall * (math.sin(2 * math.pi * f * 0.997 * t) + math.sin(2 * math.pi * f * 1.003 * t)
                                  + 0.2 * math.sin(2 * math.pi * 2 * f * t)))
    return out


def pluck(f, seconds):
    """A bass note plucked: its fundamental and a little of the next two partials, dying fast."""
    out = []
    for i in range(int(seconds * RATE)):
        t = i / RATE
        out.append(min(1.0, t / 0.002) * math.exp(-t * 7) * (math.sin(2 * math.pi * f * t)
                                                             + 0.3 * math.sin(2 * math.pi * 2 * f * t)
                                                             + 0.1 * math.sin(2 * math.pi * 3 * f * t)))
    return out


BASS = ["A1", "F1", "C2", "G1"]
CHORDS = [("A3", "C4", "E4"), ("F3", "A3", "C4"), ("C3", "E3", "G3"), ("G3", "B3", "D4")]


def music():
    beat = RATE // 2  # 120 beats a minute
    loop = [0.0] * (16 * beat)
    noise = Noise(0x5EED)
    for bar in range(4):
        for half in range(8):
            add(loop, (bar * 4) * beat + half * beat // 2, [0.55 * s for s in pluck(note(BASS[bar]), 0.4)], wrap=True)
        for n in CHORDS[bar]:
            add(loop, bar * 4 * beat, [0.12 * s for s in pad(note(n), 2.0, 0.5)], wrap=True)
        for b in range(4):
            tick = [0.18 * noise.next(0.2) * math.exp(-i / RATE * 60) for i in range(int(0.05 * RATE))]
            add(loop, (bar * 4 + b) * beat + beat // 2, tick, wrap=True)
    return loop


if __name__ == "__main__":
    ASSETS.mkdir(exist_ok=True)
    hdr("collapse.hdr", 1024, 512, collapse)
    write("gem.wav", gem(), 0.6)
    write("hit.wav", hit(), 0.7)
    write("held.wav", held(), 0.4)
    write("shift.wav", shift(), 0.3)
    write("complete.wav", complete(), 0.7)
    write("lost.wav", lost(), 0.6)
    write("run.wav", music(), 0.45)
    for name in ("collapse.hdr", "gem.wav", "hit.wav", "held.wav", "shift.wav", "complete.wav", "lost.wav", "run.wav"):
        print(name, (ASSETS / name).stat().st_size, "bytes")
