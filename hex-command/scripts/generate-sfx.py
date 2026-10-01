#!/usr/bin/env python3
"""Rebuild the game's small material-inspired sound library.

Requires numpy and scipy only when rebuilding; the browser plays ordinary WAVs.
Run from any directory: python scripts/generate-sfx.py
"""
from pathlib import Path
import wave

import numpy as np
from scipy.signal import butter, sosfilt

RATE = 24_000
OUTPUT = Path(__file__).resolve().parents[1] / 'dist' / 'assets' / 'audio'
FAMILIES = (
    'attack-bow', 'attack-melee', 'unit-soldier', 'unit-archer',
    'unit-scout', 'unit-commander', 'terrain-village', 'terrain-water',
    'terrain-forest', 'terrain-hill', 'terrain-plains', 'terrain-keep',
    'select', 'recruit', 'turn', 'victory', 'defeat',
)


def noise(rng, duration, low=0, high=10_000):
    values = rng.normal(0, 1, round(duration * RATE))
    if low and high:
        filt = butter(2, [low, high], btype='bandpass', fs=RATE, output='sos')
    else:
        filt = butter(2, high or low, btype='lowpass' if high else 'highpass',
                      fs=RATE, output='sos')
    return sosfilt(filt, values)


def envelope(length, attack=0.008, decay=0.10):
    time = np.arange(length) / RATE
    return (1 - np.exp(-time / attack)) * np.exp(-time / decay)


def add(output, sound, offset=0, amount=1):
    start = round(offset * RATE)
    count = min(len(sound), len(output) - start)
    if count > 0:
        output[start:start + count] += sound[:count] * amount


def resonator(rng, duration, base, metal=True, decay=0.10):
    time = np.arange(round(duration * RATE)) / RATE
    result = np.zeros(len(time))
    ratios = [1, 1.47, 2.09, 2.77, 3.64, 4.93] if metal else [1, 2, 3, 4]
    for index, ratio in enumerate(ratios):
        freq = base * ratio * rng.uniform(0.97, 1.03)
        result += np.sin(2 * np.pi * freq * time + rng.uniform(0, 6.28)) * (
            envelope(len(time), 0.0015, decay / (1 + index * 0.24))
            / (index + 1) ** 1.2)
    return result


def footfall(rng, duration, weight=1, armor=False):
    time = np.arange(round(duration * RATE)) / RATE
    # A damped low impact, sole/grit contact and (for soldiers) separate links.
    result = np.sin(2 * np.pi * (110 - 40 * time) * time)
    result *= envelope(len(time), 0.002, 0.028) * weight
    result += noise(rng, duration, 180, 1800) * envelope(len(time), 0.003, 0.035)
    if armor:
        for offset in [0.018, rng.uniform(0.055, 0.09)]:
            add(result, resonator(rng, 0.13, rng.uniform(550, 850), decay=0.035),
                offset, 0.20)
    return result


def brass(rng, duration, freq):
    time = np.arange(round(duration * RATE)) / RATE
    # Breath-shaped, muted herald horn: slight vibrato and a rounded harmonic body.
    phase = 2 * np.pi * freq * time + 0.06 * np.sin(2 * np.pi * 5 * time)
    result = sum(np.sin(phase * h) / h ** 1.55 for h in range(1, 6))
    shape = np.sin(np.pi * np.clip(time / duration, 0, 1)) ** 0.8
    return result * shape + noise(rng, duration, 400, 1600) * shape * 0.025


def generate(family, variant, seed):
    rng = np.random.default_rng(seed)
    durations = {'select': .13, 'victory': .83, 'defeat': .68,
                 'terrain-hill': .65, 'attack-bow': .55}
    duration = durations.get(family, .48) + (variant - 1) * .017
    result = np.zeros(round(duration * RATE))
    if family == 'attack-bow':
        # Taut fiber scrape followed by a plucked string and departing air.
        draw = noise(rng, .19, 700, 4200)
        draw *= np.sin(np.linspace(0, np.pi, len(draw))) ** 1.3
        add(result, draw, 0, .24)
        release = .13 + variant * .013
        add(result, resonator(rng, .25, 190 + variant * 23, False, .055), release)
        air = noise(rng, .23, 1800, 7500)
        air *= envelope(len(air), .010, .050)
        add(result, air, release + .010, .85)
    elif family == 'attack-melee':
        strike = .035 + variant * .009
        swing = noise(rng, .12, 450, 5000)
        add(result, swing * envelope(len(swing), .028, .05), 0, .3)
        add(result, footfall(rng, .2, 1.3), strike)
        add(result, resonator(rng, .36, 670 + variant * 76, decay=.10), strike, .85)
        add(result, noise(rng, .08, 1500, 7000) * envelope(round(.08 * RATE), .001, .013), strike, .8)
    elif family in ('unit-soldier', 'unit-archer', 'unit-scout'):
        armored = family == 'unit-soldier'
        weight = 1.1 if armored else .75 if family == 'unit-archer' else .48
        for offset in [.01, .15 + .027 * variant]:
            add(result, footfall(rng, .21, weight, armored), offset)
        if not armored:
            brush = noise(rng, .16, 800, 4500)
            add(result, brush * envelope(len(brush), .025, .045), .09, .15)
    elif family == 'unit-commander':
        add(result, footfall(rng, .18, .8, True), 0, .35)
        add(result, brass(rng, .27 + variant * .014, [196, 220, 185][variant]), .09, .7)
    elif family in ('terrain-village', 'recruit'):
        count = 3 if family == 'terrain-village' else 2
        for index in range(count):
            offset = .012 + index * (.067 + variant * .018)
            add(result, resonator(rng, .25, rng.uniform(1400, 2200), decay=.075), offset, 1 / (index + 1) ** .4)
        if family == 'recruit':
            add(result, brass(rng, .23, 294 + variant * 18), .20, .45)
    elif family == 'terrain-water':
        splash = noise(rng, duration, 180, 6500)
        add(result, splash * envelope(len(splash), .018, .09), 0)
        # Smaller separated droplets with curving bubble pitches.
        for index in range(4 + variant):
            length = round(.075 * RATE)
            time = np.arange(length) / RATE
            freq = rng.uniform(550, 1300)
            bubble = np.sin(2 * np.pi * (freq * time - freq * 2.5 * time ** 2))
            bubble *= envelope(length, .002, .015)
            add(result, bubble, rng.uniform(.08, .32), .16)
    elif family in ('terrain-forest', 'terrain-plains'):
        forest = family == 'terrain-forest'
        for index in range(4 if forest else 2):
            rustle = noise(rng, .17, 650 if forest else 400, 6200 if forest else 3400)
            rustle *= envelope(len(rustle), .023, .042)
            add(result, rustle, .007 + index * (.072 + .012 * variant), rng.uniform(.5, 1))
        if forest:
            twig = noise(rng, .025, 800, 5000)
            add(result, twig * envelope(len(twig), .001, .006), .10 + variant * .04, .2)
    elif family == 'terrain-hill':
        time = np.arange(len(result)) / RATE
        gust = noise(rng, duration, 110, 1400)
        swell = np.sin(np.pi * time / duration) ** 1.8
        result += gust * swell
        whistle = np.sin(2 * np.pi * ((340 + 43 * variant) * time + 16 * time ** 2))
        result += whistle * swell * .09
    elif family == 'terrain-keep':
        drag = noise(rng, .28, 80, 1800)
        add(result, drag * envelope(len(drag), .04, .10), 0, .65)
        add(result, footfall(rng, .18, 1.4), .16 + variant * .027)
        add(result, resonator(rng, .22, 330 + variant * 50, decay=.07), .18 + variant * .027, .4)
    elif family == 'select':
        tap = noise(rng, .08, 250, 2200)
        add(result, tap * envelope(len(tap), .001, .010), .003)
        add(result, resonator(rng, .10, 440 + variant * 73, False, .012), .003, .17)
    elif family in ('turn', 'victory', 'defeat'):
        notes = {'turn': [294, 392], 'victory': [262, 330, 392],
                 'defeat': [220, 165]}[family]
        spacing = .17 if family == 'victory' else .15
        for index, freq in enumerate(notes):
            add(result, brass(rng, .28 if family != 'defeat' else .37,
                              freq * [1, .944, 1.059][variant]),
                index * (spacing + variant * .012), .65)
    else:
        raise ValueError(family)
    # Remove DC; short edge ramps prevent clicks when sources stop or are replaced.
    result -= result.mean()
    edge = round(.006 * RATE)
    result[:edge] *= np.linspace(0, 1, edge)
    result[-edge:] *= np.linspace(1, 0, edge)
    peak = float(np.max(np.abs(result)))
    assert peak > 0
    result *= .65 / peak
    return np.round(result * 32767).astype('<i2')


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    rms_values = []
    lengths = []
    for family_index, family in enumerate(FAMILIES):
        variants = []
        for variant in range(3):
            samples = generate(family, variant, 917_003 + family_index * 307 + variant)
            assert np.max(np.abs(samples.astype(np.int32))) < 32767
            rms = float(np.sqrt(np.mean((samples.astype(float) / 32767) ** 2)))
            assert rms > .015, (family, variant, rms)
            assert .1 <= len(samples) / RATE <= .9
            path = OUTPUT / f'{family}-{variant + 1}.wav'
            with wave.open(str(path), 'wb') as out:
                out.setnchannels(1)
                out.setsampwidth(2)
                out.setframerate(RATE)
                out.writeframes(samples.tobytes())
            variants.append(samples.tobytes())
            rms_values.append(rms)
            lengths.append(len(samples) / RATE)
        assert len(set(variants)) == 3
    print(f'{len(FAMILIES) * 3} WAVs: mono {RATE} Hz, signed 16-bit; '
          f'duration {min(lengths):.3f}–{max(lengths):.3f}s; '
          f'RMS {min(rms_values):.3f}–{max(rms_values):.3f}; peak 0.65, no clipping.')


if __name__ == '__main__':
    main()
