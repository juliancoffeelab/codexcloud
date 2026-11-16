"""Small synthetic regressions for strict saved-report comparison."""

import importlib.util
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    "compare_reports", Path(__file__).with_name("compare-simulation-reports.py")
)
from journal_test_fixture import arrow_detail

comparator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(comparator)


@unittest.skipUnless(
    importlib.util.find_spec("pyarrow"), "Arrow boundary tests need pyarrow"
)
class SavedReportComparison(unittest.TestCase):
    def test_key_order_is_ignored_but_arrays_nulls_hashes_and_checksums_are_exact(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def save(name, detail, reverse=False):
                folder = root / name
                folder.mkdir(exist_ok=True)
                payload = dict(
                    kind="arena",
                    config=dict(seed=1),
                    initial=dict(units=[], tiles=[]),
                    final=dict(units=[]),
                    phases=[],
                    **detail,
                )
                if reverse:
                    payload = dict(reversed(list(payload.items())))
                data = arrow_detail(payload)
                (folder / "report.details.arrow").write_bytes(data)
                report = {
                    "detailFormat": "arrow-ipc-snapshot-dictionaries-v5",
                    "unitProfiles": {},
                    "unitFamilies": {},
                    "status": "complete",
                    "elapsedSeconds": 1,
                    "fingerprint": name,
                    "outcomes": [
                        {
                            "kind": "arena",
                            "config": {"seed": 1},
                            "stateHash": "same",
                            "detail": {
                                "file": "report.details.arrow",
                                "offset": 0,
                                "bytes": len(data),
                                "sha256": hashlib.sha256(data).hexdigest(),
                            },
                        }
                    ],
                }
                path = folder / "report.json"
                path.write_text(json.dumps(report))
                return path

            detail = {"commands": ["move", "attack"], "state": {"hp": 2, "role": None}}
            a = save("a", detail)
            b = save("b", detail, reverse=True)
            self.assertTrue(comparator.compare(a, b)["equal"])
            for path in (a, b):
                report = json.loads(path.read_text())
                second = {**report["outcomes"][0], "config": {"seed": 2}}
                report["outcomes"].append(second)
                path.write_text(json.dumps(report))
            self.assertTrue(comparator.compare(a, b)["outcomeOrderEqual"])
            report = json.loads(b.read_text())
            report["outcomes"].reverse()
            b.write_text(json.dumps(report))
            with self.assertRaisesRegex(ValueError, "Outcome array order"):
                comparator.compare(a, b)
            a = save("a", detail)
            b = save("b", detail)
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
            (b.parent / "report.details.arrow").write_bytes(b"corrupt")
            with self.assertRaisesRegex(ValueError, "checksum"):
                comparator.compare(a, b)


if __name__ == "__main__":
    unittest.main()
