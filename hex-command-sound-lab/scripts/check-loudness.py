#!/usr/bin/env python3
"""Measure decoded listening files and every take; no source audio is changed.

Requires Python 3.11+ and FFmpeg. Uses peak/RMS and strongest 50 ms window:
short Foley often cannot produce a meaningful integrated LUFS measurement.
"""
import argparse
import array
import csv
import hashlib
import json
import math
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RATE = 44100
WINDOW = round(RATE * 0.05)


def db(amplitude):
    return round(max(-120, 20 * math.log10(amplitude)), 2) if amplitude else -120.0


def measure(samples, gain_db):
    size = len(samples)
    if not size:
        return {"empty": True}
    peak = max(abs(value) for value in samples)
    energy = sum(value * value for value in samples)
    strongest = 0
    # Half-overlapping windows avoid splitting a transient at a frame boundary.
    for start in range(0, max(1, size - WINDOW // 2), WINDOW // 2):
        chunk = samples[start:min(size, start + WINDOW)]
        strongest = max(strongest, sum(value * value for value in chunk) / len(chunk))
    first = next((i for i, value in enumerate(samples) if abs(value) >= 10 ** (-55 / 20)), size)
    peak_db, rms_db, window_db = db(peak), db(math.sqrt(energy / size)), db(math.sqrt(strongest))
    flags = []
    if peak_db <= -100:
        flags.append("silent")
    if window_db + gain_db < -38:
        flags.append("quiet-at-playback-setting")
    if first / RATE > 0.1:
        flags.append("leading-quiet-over-100ms")
    if peak_db + gain_db > 0:
        flags.append("projected-clipping")
    return {
        "duration": round(size / RATE, 4), "peakDbFS": peak_db, "rmsDbFS": rms_db,
        "strongest50msRmsDbFS": window_db, "leadingBelow55Ms": round(first / RATE * 1000, 1),
        "playbackPeakDbFS": round(peak_db + gain_db, 2),
        "playbackRmsDbFS": round(rms_db + gain_db, 2),
        "playbackStrongest50msRmsDbFS": round(window_db + gain_db, 2),
        "decodedSamplesAtOrAboveFullScale": sum(abs(value) >= 1 for value in samples),
        "flags": flags,
    }


def analyze(item, volume_db, normalize, profiles):
    path = ROOT / "dist" / item["path"]
    data = subprocess.check_output([
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-nostdin", "-i", str(path),
        "-vn", "-ac", "1", "-ar", str(RATE), "-f", "f32le", "-",
    ])
    samples = array.array("f")
    samples.frombytes(data)
    if sys.byteorder != "little":
        samples.byteswap()
    profile = profiles.get(item["id"], {}) if normalize else {}
    applied = (profile.get("full", item.get("gainDb", 0)) if normalize else 0) + volume_db
    takes = []
    for index, take in enumerate(item.get("takes") or [{"start": 0, "duration": len(samples) / RATE}]):
        start = max(0, int(take["start"] * RATE))
        end = min(len(samples), int((take["start"] + take["duration"]) * RATE))
        take_gain = profile.get("takes", [])[index] if index < len(profile.get("takes", [])) else (item.get("gainDb", 0) if normalize else 0)
        takes.append({"take": index + 1, "start": take["start"], "combinedPlaybackGainDb": round(take_gain + volume_db, 2), **measure(samples[start:end], take_gain + volume_db)})
    return {
        "id": item["id"], "name": item["name"], "pack": item["pack"], "path": item["path"],
        "listeningSha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "intentionallyQuietEdit": bool(item.get("derivedFrom")),
        "catalogGainDb": item.get("gainDb", 0),
        "combinedPlaybackGainDb": round(applied, 2),
        "full": measure(samples, applied), "takes": takes,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--volume", type=float, default=55, help="App volume percentage (default: 55)")
    parser.add_argument("--legacy-gains", action="store_true", help="Ignore playback profiles when rebuilding measurements/gains")
    parser.add_argument("--no-normalize", action="store_true", help="Model Match levels unchecked")
    parser.add_argument("--only", help="Filter catalog IDs/names by substring; requires --output")
    parser.add_argument("--output", type=Path, help="Report JSON path (CSV beside it)")
    parser.add_argument("--workers", type=int, default=4)
    args = parser.parse_args()
    if not 0 < args.volume <= 100 or args.workers < 1:
        parser.error("volume must be above 0 and at most 100; workers must be positive")
    if args.only and not args.output:
        parser.error("filtered runs require --output to preserve the complete published report")
    raw = (ROOT / "dist/catalog.json").read_bytes()
    catalog = json.loads(raw)
    if args.only:
        catalog = [x for x in catalog if args.only.lower() in (x["id"] + x["name"]).lower()]
    profiles = {}
    levels_path = ROOT / "dist/playback-levels.json"
    if levels_path.exists() and not args.legacy_gains:
        levels = json.loads(levels_path.read_text())
        if levels["catalogSha256"] != hashlib.sha256(raw).hexdigest():
            raise ValueError("playback-levels.json is stale; remove it and regenerate the report and gains")
        profiles = levels["entries"]
    volume_db = 20 * math.log10(args.volume / 100)
    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        entries = list(pool.map(lambda item: analyze(item, volume_db, not args.no_normalize, profiles), catalog))
    report = {
        "catalogSha256": hashlib.sha256(raw).hexdigest(), "volumePercent": args.volume,
        "matchLevels": not args.no_normalize, "sampleRate": RATE,
        "playbackMethod": "raw" if args.no_normalize else "per-take-playback-levels" if profiles else "legacy-whole-recording-peak",
        "playbackReference": levels.get("reference") if profiles else None,
        "method": "Decoded mono listening copies; sample peak, whole-range RMS, strongest half-overlapping 50ms RMS. dBFS is not acoustic SPL or perceived loudness. Playback levels include the selected per-range profile (or legacy catalog gain) and app volume; no device/system gain.",
        "quietFlag": "Playback strongest 50ms RMS below -38 dBFS; diagnostic heuristic, not an automatic mastering target. Authored quiet edits remain intentional.",
        "entries": entries,
    }
    dest = args.output or ROOT / "dist/loudness.json"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(report, indent=2) + "\n")
    with dest.with_suffix(".csv").open("w", newline="") as handle:
        fields = ["id", "name", "pack", "range", "start", "duration", "peakDbFS", "rmsDbFS", "strongest50msRmsDbFS", "combinedPlaybackGainDb", "playbackPeakDbFS", "playbackRmsDbFS", "playbackStrongest50msRmsDbFS", "leadingBelow55Ms", "flags", "intentionallyQuietEdit"]
        writer = csv.DictWriter(handle, fieldnames=fields, extrasaction="ignore")
        writer.writeheader()
        for item in entries:
            for name, measurement in [("full", item["full"]), *[(f"take-{x['take']}", x) for x in item["takes"]]]:
                row = {**item, **measurement, "range": name, "flags": ";".join(measurement.get("flags", []))}
                writer.writerow(row)
    flagged = [x for x in entries if "quiet-at-playback-setting" in x["takes"][0].get("flags", [])]
    print(json.dumps({"recordings": len(entries), "takes": sum(len(x["takes"]) for x in entries), "quietFirstTakes": len(flagged), "silentFirstTakes": sum("silent" in x["takes"][0].get("flags", []) for x in entries), "json": str(dest), "csv": str(dest.with_suffix('.csv'))}, indent=2))


if __name__ == "__main__":
    main()
