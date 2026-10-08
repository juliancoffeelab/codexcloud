"""Small checks for fills detected from visible artwork rather than assigned armour masks."""

import importlib.util
from pathlib import Path
import unittest

file = Path(__file__).resolve().parents[1] / "art/pixel/reference/dwarf-base-cleanup/build.py"
spec = importlib.util.spec_from_file_location("dwarf_bases", file)
bases = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bases)


class DwarfPartsTest(unittest.TestCase):
    def setUp(self):
        self.rows = ["BBOBB", "BbObB", "OOOOO", "HHhOO"]
        self.parts, self.mask = bases.describe(self.rows)

    def test_shades_fill_together_and_dark_outline_separates_same_colour_parts(self):
        self.assertEqual([part["pixels"] for part in self.parts], [4, 4, 3])
        self.assertEqual([part["material"] for part in self.parts], ["brown", "brown", "skin"])

    def test_colour_edge_stops_fill_even_without_a_black_line(self):
        parts, _ = bases.describe(["HHBB", "hhbb"])
        self.assertEqual([part["pixels"] for part in parts], [4, 4])

    def test_broken_contour_leaks_and_is_rejected(self):
        rows = self.rows[:]
        rows[1] = "BbBbB"
        with self.assertRaisesRegex(ValueError, "fill region leaked"):
            bases.verify_parts(rows, self.parts, self.mask)

    def test_skin_fleck_in_leather_creates_an_artifact_and_is_rejected(self):
        rows = self.rows[:]
        rows[1] = "BHObB"
        with self.assertRaisesRegex(ValueError, "material edge changed"):
            bases.verify_parts(rows, self.parts, self.mask)

    def test_all_native_bases_recover_their_saved_regions_from_the_actual_pixels(self):
        import json

        for directory in sorted((bases.HERE / "bases").iterdir()):
            with self.subTest(unit=directory.name):
                rows = (directory / "player.mask").read_text().splitlines()
                parts = json.loads((directory / "parts.json").read_text())
                bases.verify_parts(rows, parts, (directory / "parts.mask").read_text())
                bases.verify_anchors(rows, json.loads((directory / "anchors.json").read_text()))

    def test_named_parts_cannot_share_one_colour_lake(self):
        anchors = [{"name": "cuirass", "material": "brown", "seed": [0, 0], "region": "00"},
                   {"name": "apron", "material": "brown", "seed": [3, 0], "region": "01"}]
        bases.verify_anchors(self.rows, anchors)
        rows = self.rows[:]
        rows[1] = "BbBbB"
        with self.assertRaisesRegex(ValueError, "Fill leaks between cuirass and apron"):
            bases.verify_anchors(rows, anchors)


if __name__ == "__main__":
    unittest.main()
