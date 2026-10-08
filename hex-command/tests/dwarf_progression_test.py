"""Regression checks for dwarf piece coverage and protected native materials."""

import importlib.util
from pathlib import Path
import unittest

file = Path(__file__).resolve().parents[1] / "art/pixel/reference/dwarf-progression/build.py"
spec = importlib.util.spec_from_file_location("dwarf_progression", file)
progression = importlib.util.module_from_spec(spec)
spec.loader.exec_module(progression)


class DwarfCoverageTest(unittest.TestCase):
    def setUp(self):
        self.source = ["HBBH", "ABBH", "BB.O"]
        self.mask = ["KbhP", "CwbA", "LF.o"]
        self.palette = {
            "H": [247, 190, 140, 255],
            "B": [111, 72, 43, 255],
            "A": [199, 214, 221, 255],
            ".": [0, 0, 0, 0],
            "O": [25, 22, 20, 255],
        }

    def test_plate_covers_skin_coloured_extraction_pixels_inside_the_piece(self):
        rows, palette = progression.derive(self.source, self.mask, self.palette, 2)
        self.assertIn(rows[0][3], "PpS")
        self.assertIn(rows[1][3], "PpS")
        self.assertIn(rows[2][0], "Llk")
        self.assertEqual(rows[0][2], "B")
        self.assertEqual(rows[1][1], "B")
        self.assertEqual(palette["B"], self.palette["B"])

    def test_incomplete_silver_piece_is_rejected_at_helmet_arm_and_boot(self):
        rows, palette = progression.derive(self.source, self.mask, self.palette, 3)
        for x, y in ((0, 0), (3, 1), (1, 2)):
            for stray in ("H", "B", "L"):
                with self.subTest(x=x, y=y, stray=stray):
                    corrupted = rows[:]
                    corrupted[y] = corrupted[y][:x] + stray + corrupted[y][x + 1:]
                    with self.assertRaisesRegex(ValueError, "Incomplete"):
                        progression.verify(self.source, self.mask, self.palette, corrupted, palette, 3)

    def test_skin_hair_and_wood_cannot_be_recoloured_by_armour(self):
        rows, palette = progression.derive(self.source, self.mask, self.palette, 3)
        for x, y in ((1, 0), (2, 0), (1, 1)):
            corrupted = rows[:]
            corrupted[y] = corrupted[y][:x] + "P" + corrupted[y][x + 1:]
            with self.assertRaisesRegex(ValueError, "protected"):
                progression.verify(self.source, self.mask, self.palette, corrupted, palette, 3)

    def test_armour_cannot_fill_transparent_gaps_or_remove_contours(self):
        rows, palette = progression.derive(self.source, self.mask, self.palette, 3)
        for x in (2, 3):
            corrupted = rows[:]
            corrupted[2] = corrupted[2][:x] + "P" + corrupted[2][x + 1:]
            with self.assertRaisesRegex(ValueError, "silhouette/outline"):
                progression.verify(self.source, self.mask, self.palette, corrupted, palette, 3)


if __name__ == "__main__":
    unittest.main()
