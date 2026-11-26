"""
Generate procedural Ogg Vorbis (.ogg) SFX and ambient audio assets for Startup Tycoon.
Follows docs/ASSETS_PREPARATION.md Section 9 & 11.
"""

import math
import os
import random
from typing import List

import soundfile as sf

SR = 44100


def clamp_samples(samples: List[float]) -> List[float]:
    return [max(-0.95, min(0.95, s)) for s in samples]


def write_ogg(path: str, samples: List[float]) -> None:
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    sf.write(path, clamp_samples(samples), SR, format="OGG", subtype="VORBIS")


def gen_ui_click() -> List[float]:
    dur = 0.06
    n = int(SR * dur)
    out = []
    for i in range(n):
        t = i / SR
        env = math.exp(-t * 65.0)
        freq = 780.0 - 260.0 * (t / dur)
        out.append(0.32 * env * math.sin(2.0 * math.pi * freq * t))
    return out


def gen_door() -> List[float]:
    dur = 0.25
    n = int(SR * dur)
    rng = random.Random(42)
    out = []
    for i in range(n):
        t = i / SR
        latch = 0.35 * math.exp(-t * 45.0) * math.sin(2.0 * math.pi * 240.0 * t)
        thud = 0.35 * math.exp(-max(0.0, t - 0.05) * 18.0) * math.sin(2.0 * math.pi * 95.0 * t)
        wood = 0.08 * math.exp(-t * 20.0) * (rng.random() * 2.0 - 1.0)
        out.append(latch + thud + wood)
    return out


def gen_notify() -> List[float]:
    dur = 0.34
    n = int(SR * dur)
    out = []
    for i in range(n):
        t = i / SR
        s1 = 0.24 * math.exp(-t * 10.0) * math.sin(2.0 * math.pi * 587.33 * t)
        s2 = 0.0
        if t >= 0.09:
            t2 = t - 0.09
            s2 = 0.28 * math.exp(-t2 * 9.0) * math.sin(2.0 * math.pi * 880.0 * t2)
        out.append(s1 + s2)
    return out


def gen_cash() -> List[float]:
    dur = 0.32
    n = int(SR * dur)
    out = []
    for i in range(n):
        t = i / SR
        s1 = 0.22 * math.exp(-t * 14.0) * math.sin(2.0 * math.pi * 987.77 * t)
        s2 = 0.0
        if t >= 0.07:
            t2 = t - 0.07
            s2 = 0.26 * math.exp(-t2 * 10.0) * math.sin(2.0 * math.pi * 1318.51 * t2)
        out.append(s1 + s2)
    return out


def gen_step() -> List[float]:
    dur = 0.11
    n = int(SR * dur)
    rng = random.Random(7)
    out = []
    lp = 0.0
    for i in range(n):
        t = i / SR
        env = math.exp(-t * 38.0)
        noise = rng.random() * 2.0 - 1.0
        lp = lp * 0.85 + noise * 0.15
        thump = 0.22 * env * math.sin(2.0 * math.pi * 115.0 * t)
        out.append(thump + 0.28 * env * lp)
    return out


def gen_typing() -> List[float]:
    dur = 0.15
    n = int(SR * dur)
    rng = random.Random(99)
    out = []
    for i in range(n):
        t = i / SR
        tap1 = math.exp(-t * 95.0) * (0.20 * math.sin(2.0 * math.pi * 1400.0 * t) + 0.12 * (rng.random() * 2.0 - 1.0))
        tap2 = 0.0
        if t >= 0.065:
            t2 = t - 0.065
            tap2 = math.exp(-t2 * 95.0) * (0.18 * math.sin(2.0 * math.pi * 1620.0 * t2) + 0.10 * (rng.random() * 2.0 - 1.0))
        out.append(tap1 + tap2)
    return out


def gen_office_ambience() -> List[float]:
    # Gentle 4-second seamless room tone (soft air + warm pad)
    dur = 4.0
    n = int(SR * dur)
    rng = random.Random(123)
    out = []
    lp = 0.0
    for i in range(n):
        t = i / SR
        noise = rng.random() * 2.0 - 1.0
        lp = lp * 0.985 + noise * 0.015
        pad = 0.018 * math.sin(2.0 * math.pi * 110.0 * t) + 0.010 * math.sin(2.0 * math.pi * 165.0 * t)
        out.append(0.18 * lp + pad)
    return out


def main() -> None:
    targets = [
        ("assets/audio/sfx/sfx_ui_click.ogg", gen_ui_click()),
        ("assets/audio/sfx/sfx_door.ogg", gen_door()),
        ("assets/audio/sfx/sfx_notify.ogg", gen_notify()),
        ("assets/audio/sfx/sfx_cash.ogg", gen_cash()),
        ("assets/audio/sfx/sfx_step.ogg", gen_step()),
        ("assets/audio/sfx/sfx_typing.ogg", gen_typing()),
        ("assets/audio/ambient/ambient_office.ogg", gen_office_ambience()),
    ]
    for path, samples in targets:
        write_ogg(path, samples)
        print(f"Generated {path} ({os.path.getsize(path)} bytes)")


if __name__ == "__main__":
    main()
