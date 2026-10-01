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


def soft_step(rng, duration, scout=False):
    """One padded sole contact; deliberately no pitched wooden impact."""
    length = round(duration * RATE)
    contact = noise(rng, duration, 35, 290 if scout else 480)
    contact *= envelope(length, .006 if scout else .003, .028 if scout else .038)
    scuff = noise(rng, duration, 450, 2100 if scout else 2800)
    scuff *= envelope(length, .014, .028)
    return contact + scuff * (.10 if scout else .13)


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
                 'terrain-hill': .42, 'unit-commander': .24, 'attack-bow': .55,
                 'unit-soldier': .16, 'unit-archer': .20, 'unit-scout': .18,
                 'terrain-village': .27, 'recruit': .32,
                 'terrain-water': .28, 'terrain-forest': .25,
                 'terrain-plains': .23, 'terrain-keep': .26}
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
    elif family == 'unit-soldier':
        # A single short armor-link tick. High, fixed inharmonic modes decay
        # immediately instead of the longer sheet-metal bending sound.
        time = np.arange(len(result)) / RATE
        for index, ratio in enumerate([1, 1.63, 2.34, 3.07]):
            freq = [1850, 2230, 1630][variant] * ratio * rng.uniform(.985, 1.015)
            result += np.sin(2 * np.pi * freq * time + rng.uniform(0, 6.28)) * (
                envelope(len(time), .0005, .009 / (1 + index * .35))
                / (index + 1))
        result += noise(rng, duration, 1300, 6200) * envelope(len(time), .001, .013) * .18
    elif family in ('unit-archer', 'unit-scout'):
        add(result, soft_step(rng, duration, family == 'unit-scout'), .004)
    elif family == 'unit-commander':
        # One fabric/leather brush: no low impact or abrupt contact transient.
        leather = noise(rng, duration, 500, [1500, 1700, 1300][variant])
        cloth = noise(rng, duration, 900, [3000, 3300, 2700][variant])
        brush = envelope(len(result), .028, .060)
        result += (leather * .60 + cloth * .35) * brush
    elif family == 'terrain-village':
        # One small gold coin contacting another, rather than a repeated jingle.
        add(result, resonator(rng, duration, [1790, 2030, 1880][variant], decay=.048), .006)
        tap = noise(rng, .025, 2600, 8000)
        add(result, tap * envelope(len(tap), .0006, .004), .006, .10)
    elif family == 'recruit':
        for index in range(2):
            offset = .012 + index * (.067 + variant * .018)
            add(result, resonator(rng, .25, rng.uniform(1400, 2200), decay=.075), offset, 1 / (index + 1) ** .4)
    elif family == 'terrain-water':
        # One liquid plop: rising bubble resonance, followed by a small wet wash.
        # No broadband attack or low booming body that suggests an explosion.
        time = np.arange(len(result)) / RATE
        base = [430, 530, 380][variant]
        phase = 2 * np.pi * (base * time + (1000 + variant * 260) * time ** 2)
        bubble = np.sin(phase) + .14 * np.sin(phase * 2.13)
        result += bubble * envelope(len(time), .004, .030)
        wash = noise(rng, duration, 220, 1900)
        result += wash * envelope(len(time), .014, .055) * .23
    elif family in ('terrain-forest', 'terrain-plains'):
        forest = family == 'terrain-forest'
        # One smooth leafy/grass brush, with no rhythmic repeated bursts.
        rustle = noise(rng, duration, [900, 1150, 740][variant] if forest else 350,
                       [5700, 6700, 4700][variant] if forest else 2900 + variant * 300)
        result += rustle * envelope(len(rustle), .025 if forest else .018,
                                    .040 + variant * .006)
    elif family == 'terrain-hill':
        time = np.arange(len(result)) / RATE
        # Gentle wind arrives promptly, then fades without a long howl.
        swell = envelope(len(time), .012, .14) * np.clip((duration - time) / .065, 0, 1)
        base = [385, 460, 335][variant]
        wander = 16 * np.sin(2 * np.pi * time / duration * .73 + variant * .5)
        phase = 2 * np.pi * np.cumsum(base + wander) / RATE
        howl = np.sin(phase) + .16 * np.sin(phase * 1.98 + .6)
        airy = noise(rng, duration, 160, 1150)
        resonant_air = noise(rng, duration, base - 45, base + 45)
        result += (howl * .12 + airy * .35 + resonant_air * .3) * swell
    elif family == 'terrain-keep':
        # One muted latch closing against stone, without a dragged-then-banged
        # sequence or ringing sheet metal.
        contact = noise(rng, duration, 70, 850 + variant * 170)
        result += contact * envelope(len(contact), .003, .032)
        add(result, resonator(rng, .12, 1350 + variant * 130, decay=.012), .007, .18)
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
    levels = {'unit-soldier': .36, 'unit-archer': .32, 'unit-scout': .25,
              'terrain-village': .34, 'terrain-water': .30,
              'terrain-forest': .27, 'terrain-plains': .21,
              'terrain-hill': .16, 'unit-commander': .20, 'terrain-keep': .30}
    result *= levels.get(family, .65) / peak
    # Darken bright transients without renormalizing them back to full volume.
    softened = sosfilt(butter(2, 2200, btype='lowpass', fs=RATE, output='sos'), result)
    result = .25 * result + .75 * softened
    edge = round(.010 * RATE)
    result[:edge] *= np.linspace(0, 1, edge)
    result[-edge:] *= np.linspace(1, 0, edge)
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
            assert rms > .005, (family, variant, rms)
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
          f'RMS {min(rms_values):.3f}–{max(rms_values):.3f}; '
          f'rounded attacks and softened high frequencies, no clipping.')


if __name__ == '__main__':
    main()
