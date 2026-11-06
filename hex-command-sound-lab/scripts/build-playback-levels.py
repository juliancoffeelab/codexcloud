#!/usr/bin/env python3
"""Generate non-destructive per-take playback gains from check-loudness.py output."""
import hashlib
import json
import math
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CEILING = -1.5


def gain_for(stats, target):
    if stats.get("empty") or stats["peakDbFS"] <= -100:
        return 0.0
    wanted = target - stats["strongest50msRmsDbFS"]
    headroom = CEILING - stats["peakDbFS"]
    return round(max(-18.0, min(24.0, wanted, headroom)), 2)


def main():
    raw = (ROOT / "dist/catalog.json").read_bytes()
    catalog = json.loads(raw)
    report = json.loads((ROOT / "dist/loudness.json").read_text())
    digest = hashlib.sha256(raw).hexdigest()
    if report["catalogSha256"] != digest:
        raise ValueError("Loudness report is stale: rerun check-loudness.py")
    measured = {item["id"]: item for item in report["entries"]}
    reference_raw = (ROOT / "loudness-reference.json").read_bytes()
    (ROOT / "dist/loudness-reference.json").write_bytes(reference_raw)
    reference = json.loads(reference_raw)
    game_gain = 20 * math.log10(reference["gameMasterGain"])
    board_gain = 20 * math.log10(reference["boardReferenceVolumePercent"] / 100)
    reference_output = statistics.median(measured[id]["full"]["strongest50msRmsDbFS"] + game_gain for id in reference["referenceIds"])
    target = reference_output - board_gain
    reference["medianGameOutputStrongest50msRmsDbFS"] = round(reference_output, 2)
    reference["targetBeforeBoardMasterDbFS"] = round(target, 2)
    entries = {}
    for item in catalog:
        stats = measured[item["id"]]
        if hashlib.sha256((ROOT / "dist" / item["path"]).read_bytes()).hexdigest() != stats["listeningSha256"]:
            raise ValueError(f"Listening file changed: {item['id']}")
        preserve = bool(item.get("preserveLevel", item.get("derivedFrom")))
        if preserve:
            full = 0
            takes = [0 for take in stats["takes"]]
        elif item["pack"] == "current":
            family = item["name"].rsplit("-", 1)[0]
            full = round(game_gain - board_gain + 20 * math.log10(reference["familyGains"].get(family, 1)), 2)
            takes = [full for take in stats["takes"]]
        else:
            full = gain_for(stats["full"], target)
            takes = [gain_for(take, target) for take in stats["takes"]]
        entries[item["id"]] = {"full": full, "takes": takes, "preserved": preserve, "gameRelativeGain": item["pack"] == "current"}
    output = {"catalogSha256": digest, "targetStrongest50msRmsDbFS": round(target, 2), "reference": reference,
              "samplePeakCeilingDbFS": CEILING, "minGainDb": -18, "maxGainDb": 24,
              "method": "Constant gain per decoded take/full session. Target strongest 50ms RMS subject to sample-peak ceiling; no compressor, no true-peak or perceptual loudness guarantee. Authored quiet edits keep zero gain; archived game samples retain their relative game gain at the board reference volume.",
              "entries": entries}
    (ROOT / "dist/playback-levels.json").write_text(json.dumps(output, indent=2) + "\n")
    print(f"Generated per-take playback gains for {len(entries)} recordings")


if __name__ == "__main__":
    main()
