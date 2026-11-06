#!/usr/bin/env python3
"""Numerical regression checks for level measurements and published playback gains."""
import importlib.util
import json
import math
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def module(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / file)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


meter = module("loudness", "check-loudness.py")
levels = module("playback_levels", "build-playback-levels.py")


class PlaybackLevels(unittest.TestCase):
    def test_actual_selected_take_balances_quiet_and_loud_passes(self):
        quiet = meter.measure([0.02] * 4410, 0)
        loud = meter.measure([0.2] * 4410, 0)
        quiet_gain, loud_gain = levels.gain_for(quiet, -20), levels.gain_for(loud, -20)
        self.assertGreater(quiet_gain - loud_gain, 19)
        self.assertAlmostEqual(quiet["strongest50msRmsDbFS"] + quiet_gain,
                               loud["strongest50msRmsDbFS"] + loud_gain, places=1)

    def test_transient_cannot_be_boosted_past_peak_ceiling(self):
        transient = meter.measure([1.0] + [0.0] * 4409, 0)
        gain = levels.gain_for(transient, -20)
        self.assertLessEqual(transient["peakDbFS"] + gain, levels.CEILING + 0.01)

    def test_silence_is_not_amplified_and_leading_quiet_is_measured(self):
        self.assertEqual(levels.gain_for(meter.measure([0.0] * 4410, 0), -20), 0)
        self.assertEqual(meter.measure([0.0] * 4410 + [0.1] * 4410, 0)["leadingBelow55Ms"], 100)

    def test_all_published_ranges_preserve_edits_and_have_headroom(self):
        catalog = json.loads((ROOT / "dist/catalog.json").read_text())
        report = {x["id"]: x for x in json.loads((ROOT / "dist/loudness.json").read_text())["entries"]}
        profiles = json.loads((ROOT / "dist/playback-levels.json").read_text())["entries"]
        self.assertEqual(set(profiles), {x["id"] for x in catalog})
        for item in catalog:
            profile, stats = profiles[item["id"]], report[item["id"]]
            self.assertEqual(len(profile["takes"]), len(item.get("takes") or [None]))
            for gain, measurement in [(profile["full"], stats["full"]), *zip(profile["takes"], stats["takes"])]:
                self.assertTrue(math.isfinite(gain))
                if item.get("preserveLevel", item.get("derivedFrom")):
                    self.assertEqual(gain, 0, item["id"])
                elif item["pack"] == "current":
                    reference = json.loads((ROOT / "loudness-reference.json").read_text())
                    family = item["name"].rsplit("-", 1)[0]
                    expected = 20 * math.log10(reference["gameMasterGain"] / (reference["boardReferenceVolumePercent"] / 100) * reference["familyGains"].get(family, 1))
                    self.assertAlmostEqual(gain, expected, places=2, msg=item["id"])
                else:
                    self.assertGreaterEqual(gain, -18)
                    self.assertLessEqual(gain, 24)
                    self.assertLessEqual(measurement["peakDbFS"] + gain, levels.CEILING + 0.01, item["id"])


if __name__ == "__main__":
    unittest.main()
