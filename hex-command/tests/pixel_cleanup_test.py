"""Small regression checks for reviewed material recipe replay."""

import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
from pixel_cleanup import apply_cleanup


class MaterialReplayTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.units = root / "units"
        self.recipes = root / "recipes"
        self.unit = self.units / "example"
        self.unit.mkdir(parents=True)
        self.recipes.mkdir()
        (self.unit / "player.txt").write_text("AB\n")
        (self.unit / "enemy.txt").write_text("AC\n")
        (self.unit / "palette.json").write_text(json.dumps({
            "A": [120, 120, 120, 255],
            "B": [0, 100, 110, 255],
            "C": [185, 24, 37, 255],
        }))
        edit = {"x": 0, "y": 0, "from": "A", "to": "S"}
        recipe = {"units": {"example": {
            "canvas": [2, 1],
            "matrixChanges": [edit],
            "enemyMatrixChanges": [edit],
            "paletteChanges": {"S": {"from": None, "to": [160, 180, 190, 255]}},
        }}}
        (self.recipes / "race.json").write_text(json.dumps(recipe))
        self.matrices = [self.unit / "player.txt", self.unit / "enemy.txt"]

    def apply(self, check=False):
        apply_cleanup(self.matrices, check=check, units_root=self.units, recipe_root=self.recipes)

    def test_both_factions_keep_identity_and_replay_is_idempotent(self):
        self.apply()
        self.assertEqual(self.matrices[0].read_text(), "SB\n")
        self.assertEqual(self.matrices[1].read_text(), "SC\n")
        self.assertEqual(json.loads((self.unit / "palette.json").read_text())["S"], [160, 180, 190, 255])
        before = [path.read_bytes() for path in self.unit.iterdir()]
        self.apply()
        self.apply(check=True)
        self.assertEqual(before, [path.read_bytes() for path in self.unit.iterdir()])

    def test_stale_pose_is_rejected(self):
        self.matrices[0].write_text(".B\n")
        with self.assertRaisesRegex(ValueError, "stale cleanup source"):
            self.apply()
        self.assertEqual(self.matrices[0].read_text(), ".B\n")

    def test_check_reports_missing_cleanup_without_writing(self):
        with self.assertRaisesRegex(ValueError, "stale cleanup source"):
            self.apply(check=True)
        self.assertEqual(self.matrices[0].read_text(), "AB\n")

    def test_recipe_cannot_change_transparency_or_outline(self):
        file = self.recipes / "race.json"
        recipe = json.loads(file.read_text())
        for symbol in (".", "O"):
            with self.subTest(symbol=symbol):
                recipe["units"]["example"]["matrixChanges"][0]["to"] = symbol
                file.write_text(json.dumps(recipe))
                with self.assertRaisesRegex(ValueError, "must preserve"):
                    self.apply()
                self.assertEqual(self.matrices[0].read_text(), "AB\n")

    def test_reviewed_interior_outline_removal_preserves_outer_contour(self):
        file = self.recipes / "race.json"
        recipe = json.loads(file.read_text())
        unit = recipe["units"]["example"]
        unit["canvas"] = [3, 3]
        edit = {"x": 1, "y": 1, "from": "O", "to": "A", "removeInteriorOutline": True}
        unit["matrixChanges"] = [edit]
        unit["enemyMatrixChanges"] = [edit]
        for matrix in self.matrices:
            matrix.write_text("AAA\nAOA\nAAA\n")
        file.write_text(json.dumps(recipe))
        self.apply()
        self.assertEqual(self.matrices[0].read_text(), "AAA\nAAA\nAAA\n")
        self.assertEqual(self.matrices[1].read_text(), "AAA\nAAA\nAAA\n")
        edit["x"] = 0
        file.write_text(json.dumps(recipe))
        self.matrices[0].write_text("AAA\nOAA\nAAA\n")
        with self.assertRaisesRegex(ValueError, "must preserve outlines"):
            self.apply()
        self.assertEqual(self.matrices[0].read_text(), "AAA\nOAA\nAAA\n")


if __name__ == "__main__":
    unittest.main()
