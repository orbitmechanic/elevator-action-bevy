#!/usr/bin/env python3
"""Generate the chiptune soundtrack and SFX as 22.05 kHz mono WAV files."""

import math
import random
import struct
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "audio"
SR = 22050


def env(i, n, attack=0.01, release=0.25):
    a = int(n * attack) or 1
    r = int(n * release) or 1
    if i < a:
        return i / a
    if i > n - r:
        return max(0.0, (n - i) / r)
    return 1.0


def square(freq, dur, vol=0.3, duty=0.5, slide=None, attack=0.01, release=0.25):
    n = int(SR * dur)
    out = []
    phase = 0.0
    for i in range(n):
        f = freq if slide is None else freq + (slide - freq) * (i / max(1, n - 1))
        phase += f / SR
        s = vol if (phase % 1.0) < duty else -vol
        out.append(s * env(i, n, attack, release))
    return out


def triangle(freq, dur, vol=0.3, attack=0.01, release=0.25):
    n = int(SR * dur)
    out = []
    phase = 0.0
    for i in range(n):
        phase += freq / SR
        p = phase % 1.0
        s = 4 * abs(p - 0.5) - 1
        out.append(s * vol * env(i, n, attack, release))
    return out


def noise(dur, vol=0.2, decay=3.0, lowpass=0.5):
    n = int(SR * dur)
    out = []
    rnd = random.Random(1234)
    prev = 0.0
    for i in range(n):
        raw = rnd.uniform(-1, 1)
        prev = prev * lowpass + raw * (1 - lowpass)
        out.append(prev * vol * math.exp(-decay * i / n))
    return out


def mix(*tracks):
    n = max(len(t) for t in tracks)
    out = [0.0] * n
    for t in tracks:
        for i, v in enumerate(t):
            out[i] += v
    return out


def seq(*parts):
    out = []
    for p in parts:
        out.extend(p)
    return out


def rest(dur):
    return [0.0] * int(SR * dur)


def write(name, samples, gain=0.8):
    OUT.mkdir(parents=True, exist_ok=True)
    peak = max(1e-6, max(abs(s) for s in samples))
    scale = min(gain, 0.95) / peak
    data = b"".join(struct.pack("<h", int(max(-1, min(1, s * scale)) * 32767)) for s in samples)
    path = OUT / name
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(data)
    print(f"{name}: {len(samples)/SR:.2f}s {len(data)//1024} KiB")


def sfx():
    write("shoot.wav", mix(
        square(980, 0.09, 0.26, 0.3, slide=220, release=0.5),
        noise(0.05, 0.10, 6.0, 0.3),
    ))
    write("enemy_shoot.wav", mix(
        square(520, 0.10, 0.22, 0.25, slide=180, release=0.5),
        noise(0.04, 0.06, 7.0, 0.4),
    ))
    write("hit.wav", mix(
        noise(0.28, 0.55, 4.5, 0.65),
        square(120, 0.20, 0.25, 0.5, slide=55, release=0.7),
    ))
    write("pickup.wav", seq(
        square(1046, 0.055, 0.20, 0.5, release=0.2),
        square(1318, 0.055, 0.20, 0.5, release=0.2),
        square(1568, 0.055, 0.20, 0.5, release=0.2),
        square(2093, 0.12, 0.22, 0.5, release=0.6),
    ))
    write("death.wav", mix(
        square(440, 0.75, 0.28, 0.5, slide=70, release=0.4),
        noise(0.5, 0.14, 2.5, 0.8),
    ))
    write("ding.wav", seq(
        triangle(1318, 0.12, 0.24, release=0.5),
        rest(0.03),
        triangle(1760, 0.22, 0.22, release=0.7),
    ))
    write("door.wav", mix(
        noise(0.08, 0.20, 5.0, 0.5),
        square(180, 0.07, 0.12, 0.3, slide=120, release=0.5),
    ))
    write("call.wav", seq(
        square(660, 0.05, 0.16, 0.5),
        rest(0.03),
        square(880, 0.05, 0.16, 0.5),
    ))
    write("level_clear.wav", seq(
        square(523, 0.11, 0.22, 0.5), square(659, 0.11, 0.22, 0.5),
        square(784, 0.11, 0.22, 0.5), square(1046, 0.16, 0.24, 0.5),
        rest(0.04),
        square(784, 0.11, 0.22, 0.5), square(1046, 0.28, 0.26, 0.5),
    ))
    write("game_over.wav", seq(
        square(392, 0.22, 0.24, 0.5), square(330, 0.22, 0.24, 0.5),
        square(262, 0.22, 0.24, 0.5), square(196, 0.6, 0.26, 0.5, release=0.7),
    ))
    write("victory.wav", seq(
        square(523, 0.12, 0.24, 0.5), square(659, 0.12, 0.24, 0.5),
        square(784, 0.12, 0.24, 0.5), square(1046, 0.18, 0.26, 0.5),
        square(784, 0.08, 0.20, 0.5), square(1046, 0.5, 0.28, 0.5, release=0.6),
    ))


def music():
    """A 16-second, 8-bar spy-jazz-ish loop (parallel lanes, then mixed)."""
    bpm = 118.0
    beat = 60.0 / bpm
    bass_notes = [55, 55, 43.65, 43.65, 65.41, 65.41, 49.0, 49.0]
    chords = [(220, 261.63, 329.63), (174.61, 220, 261.63),
              (261.63, 329.63, 392.0), (196.0, 246.94, 293.66)]
    bass, arp, hats, lead = [], [], [], []
    for b in range(8):
        root = bass_notes[b]
        chord = chords[b % 4]
        for k in range(8):
            f = root if k % 2 == 0 else root * 2
            bass.extend(square(f, beat / 2, 0.13, 0.5, release=0.18))
        for k in range(4):
            f = chord[k % 3] * 2
            arp.extend(square(f, beat / 2, 0.038, 0.25, release=0.3))
            arp.extend(rest(beat / 2))
        for k in range(4):
            hats.extend(noise(0.022, 0.032, 9.0, 0.92))
            hats.extend(rest(beat - 0.022))
        # two-note answering motif
        f1 = chord[1] * 4
        f2 = chord[2] * 4
        lead.extend(rest(beat * 2))
        lead.extend(square(f1, beat / 2, 0.05, 0.5, release=0.4))
        lead.extend(rest(beat / 2))
        lead.extend(square(f2, beat / 2, 0.045, 0.5, release=0.5))
        lead.extend(rest(beat / 2))
    # trim every lane to the same length
    n = min(len(bass), len(arp), len(hats), len(lead))
    write("music.wav", mix(bass[:n], arp[:n], hats[:n], lead[:n]), gain=0.46)


if __name__ == "__main__":
    sfx()
    music()
