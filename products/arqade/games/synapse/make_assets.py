"""Writes Synapse's sky and sounds into assets/ (DIRECTION P7.20). Standard library only: python make_assets.py

  mind.hdr      512 x 256, an equirectangular HDR sky: the dark of a mind, with faint clouds of teal and violet drifting
                through it and a warm glow low on one side. Run-length encoded, as tools write Radiance files: Qt misreads
                one written plainly (QQ's README, "Materials").
  node1.wav     0.7 s   Origin's note, C5: a struck glass bell.
  node2.wav     0.7 s   Echo's, E5.
  node3.wav     0.7 s   Pulse's, G5.
  node4.wav     0.7 s   Aether's, C6.
  signal.wav    0.9 s   a signal repeated: the four notes rising quickly.
  wrong.wav     0.7 s   a node out of turn: two notes a semitone apart, beating, falling away.
  mastered.wav  1.8 s   the game mastered: the notes in a climbing chord, held.
  run.wav       12 s    the music, looping: a low, slow drone through C and A flat, quiet under the nodes.

Every sound is written in both channels, as Qt's ambient sounds take it. The files are the same bytes each time.
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
TEAL, VIOLET, WARM = (0.004, 0.022, 0.026), (0.016, 0.008, 0.030), (0.040, 0.016, 0.006)


def mind(x, y, w=512, h=256):
    latitude = math.pi / 2 - (y + 0.5) / h * math.pi
    longitude = (x + 0.5) / w * 2 * math.pi
    up = math.sin(latitude)
    # Two layers of soft cloud, each a product of slow waves round the sphere, so nothing is sharp.
    a = 0.5 + 0.5 * math.sin(longitude * 2 + up * 3.1) * math.cos(longitude * 3 - up * 2.3 + 0.8)
    b = 0.5 + 0.5 * math.sin(longitude * 3 - up * 2.7 + 2.1) * math.cos(longitude - up * 4.2 + 0.3)
    near = math.exp(-abs(up) / 0.5)
    warm = math.exp(-((up + 0.15) / 0.25) ** 2) * (0.5 + 0.5 * math.cos(longitude - 1.0))
    return [VOID[i] + TEAL[i] * a * a * near + VIOLET[i] * b * b * near + WARM[i] * warm for i in range(3)]


# The sounds ---------------------------------------------------------------------------------------------------------

def note(name):
    """A note's frequency from its name and octave: A4 is 440; "Ab" is A flat."""
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


NOTES = ["C5", "E5", "G5", "C6"]


def signal():
    out = [0.0] * int(0.9 * RATE)
    for k, n in enumerate(NOTES):
        add(out, int(k * 0.07 * RATE), [0.7 * s for s in glass(note(n), 0.9 - k * 0.07)])
    return fade_end(out)


def wrong():
    out = [0.0] * int(0.7 * RATE)
    for f in (note("E4"), note("F4")):
        tone = []
        for i in range(len(out)):
            t = i / RATE
            tone.append(math.sin(2 * math.pi * f * t) * math.exp(-t * 4) * min(1.0, t / 0.01))
        add(out, 0, tone)
    return fade_end(out)


def mastered():
    out = [0.0] * int(1.8 * RATE)
    for k, n in enumerate(["C5", "E5", "G5", "C6", "E6"]):
        add(out, int(k * 0.12 * RATE), [0.7 * s for s in glass(note(n), 1.8 - k * 0.12)])
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
    loop = [0.0] * (12 * RATE)
    for k, n in enumerate(["C2", "Ab1"]):
        add(loop, k * 6 * RATE, drone(note(n), 6.0, 3.0), wrap=True)
    return loop


if __name__ == "__main__":
    ASSETS.mkdir(exist_ok=True)
    hdr("mind.hdr", 512, 256, mind)
    for k, n in enumerate(NOTES):
        write("node%d.wav" % (k + 1), fade_end(glass(note(n), 0.7)), 0.6)
    write("signal.wav", signal(), 0.6)
    write("wrong.wav", wrong(), 0.6)
    write("mastered.wav", mastered(), 0.7)
    write("run.wav", music(), 0.35)
    for name in ["mind.hdr"] + ["node%d.wav" % k for k in range(1, 5)] + ["signal.wav", "wrong.wav", "mastered.wav",
                                                                          "run.wav"]:
        print(name, (ASSETS / name).stat().st_size, "bytes")
