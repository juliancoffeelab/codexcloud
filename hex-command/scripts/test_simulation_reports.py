"""Small synthetic regressions for strict saved-report comparison."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("compare_reports", Path(__file__).with_name("compare-simulation-reports.py"))
comparator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(comparator)


class SavedReportComparison(unittest.TestCase):
    def test_key_order_is_ignored_but_arrays_nulls_hashes_and_checksums_are_exact(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def save(name, detail, reverse=False):
                folder = root / name
                folder.mkdir(exist_ok=True)
                data = (json.dumps(detail, sort_keys=reverse) + "\n").encode()
                (folder / "report.details.jsonl").write_bytes(data)
                report = {"status": "complete", "elapsedSeconds": 1, "fingerprint": name,
                          "outcomes": [{"kind": "arena", "config": {"seed": 1}, "stateHash": "same",
                                        "detail": {"file": "report.details.jsonl", "offset": 0,
                                                   "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}}]}
                path = folder / "report.json"
                path.write_text(json.dumps(report))
                return path

            detail = {"commands": ["move", "attack"], "state": {"hp": 2, "role": None}}
            a = save("a", detail)
            b = save("b", detail, reverse=True)
            self.assertTrue(comparator.compare(a, b)["equal"])
            for path in (a,b):
                report = json.loads(path.read_text())
                second = {**report["outcomes"][0],"config":{"seed":2}}
                report["outcomes"].append(second)
                path.write_text(json.dumps(report))
            self.assertTrue(comparator.compare(a,b)["outcomeOrderEqual"])
            report = json.loads(b.read_text())
            report["outcomes"].reverse()
            b.write_text(json.dumps(report))
            with self.assertRaisesRegex(ValueError,"Outcome array order"):
                comparator.compare(a,b)
            a = save("a",detail)
            b = save("b",detail)
            for changed in [
                {"commands": ["attack", "move"], "state": detail["state"]},
                {"commands": detail["commands"], "state": {"hp": 2}},
                {"commands": detail["commands"], "state": {"hp": 2.0, "role": None}},
            ]:
                with self.assertRaisesRegex(ValueError, "Detailed gameplay"):
                    comparator.compare(a, save("b", changed))
            b = save("b", detail)
            report = json.loads(b.read_text())
            report["outcomes"][0]["stateHash"] = "changed"
            b.write_text(json.dumps(report))
            with self.assertRaisesRegex(ValueError, "Compact gameplay"):
                comparator.compare(a, b)
            b = save("b", detail)
            (b.parent / "report.details.jsonl").write_bytes(b"corrupt")
            with self.assertRaisesRegex(ValueError, "checksum"):
                comparator.compare(a, b)




class SupportMeasurement(unittest.TestCase):
    def test_actual_recipients_exclude_self_and_do_not_double_count_overlapping_healers(self):
        spec = importlib.util.spec_from_file_location('support_measure', Path(__file__).with_name('measure-support.py'))
        audit = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(audit)
        catalog = {'medic': {'hp': 10, 'healing': 3}, 'strong': {'hp': 10, 'healing': 5}, 'troop': {'hp': 10}}
        units = [dict(id='a', type='medic', hp=10, side='player', tile='0,0'),
                 dict(id='b', type='strong', hp=10, side='player', tile='2,0'),
                 dict(id='c', type='troop', hp=3, side='player', tile='1,0')]
        tiles = {u['tile']: {'terrain': 'plains'} for u in units}
        counts = audit.observe(units, 'player', 'healing', catalog, tiles, {})
        self.assertEqual(counts['providerTurns'], 2)
        self.assertEqual(counts['contacts'], 2)
        self.assertEqual(counts['coveredRecipients'], 1)
        self.assertEqual(counts['overlappingRecipients'], 1)
        self.assertEqual(counts['healedRecipients'], 1)
        self.assertEqual(counts['healedHP'], 5)
        tiles['1,0']['terrain'] = 'grove'
        counts = audit.observe(units, 'player', 'healing', catalog, tiles, {'1,0': 'player'})
        self.assertEqual(counts['healedHP'], 3)
        units[-1]['hp'] = 6
        counts = audit.observe(units, 'player', 'healing', catalog, tiles, {'1,0': 'player'})
        self.assertEqual(counts['healedRecipients'], 0)
        self.assertEqual(counts['contacts'], 2)


if __name__ == "__main__":
    unittest.main()
