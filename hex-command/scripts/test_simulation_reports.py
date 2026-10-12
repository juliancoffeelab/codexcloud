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


if __name__ == "__main__":
    unittest.main()
