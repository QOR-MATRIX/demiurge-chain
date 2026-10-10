"""Write chime.wav: one second of a soft bell (A5 and its fifth, fading), 16-bit mono at 22050 Hz.

The sound QQ's tests give a Sound entity. Made by this script so the fixture is small and its origin is plain.
"""

import math
import pathlib
import struct
import wave

RATE = 22050
SECONDS = 1.0


def sample(t: float) -> float:
    fade = math.exp(-4.0 * t)
    attack = min(1.0, t / 0.005)
    return attack * fade * (0.6 * math.sin(2 * math.pi * 880 * t) + 0.3 * math.sin(2 * math.pi * 1320 * t))


def main() -> None:
    frames = b"".join(
        struct.pack("<h", round(32767 * 0.8 * sample(i / RATE))) for i in range(int(RATE * SECONDS))
    )
    out = pathlib.Path(__file__).with_name("chime.wav")
    with wave.open(str(out), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(frames)


if __name__ == "__main__":
    main()
