"""The requested neighbour check uses exact rendered colour and cardinal adjacency."""

import importlib.util
from pathlib import Path
import unittest

file = Path(__file__).resolve().parents[1] / ".agents/skills/pixel-cleaning/scripts/check_isolated_pixels.py"
spec = importlib.util.spec_from_file_location("isolated_pixels", file)
check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check)


class IsolatedPixelsTest(unittest.TestCase):
    def setUp(self):
        self.palette = {".": [0, 0, 0, 0], "B": [111, 72, 43, 255],
                        "b": [64, 40, 28, 255], "X": [111, 72, 43, 255], "O": [25, 22, 20, 255]}

    def test_approved_material_shades_connect_only_within_their_material(self):
        self.assertEqual(check.isolated(["Bb"], self.palette, allowed_shades={"leather": "Bb"}), [])
        self.assertEqual(len(check.isolated(["bO"], self.palette, allowed_shades={"leather": "Bb"})), 2)

    def test_unapproved_shade_is_flagged_even_in_a_solid_patch(self):
        flags = check.isolated(["bbb"], self.palette, allowed_shades={"leather": "Bb"},
                               unapproved_shades={"b"})
        self.assertEqual(len(flags), 3)
        self.assertTrue(all(f["kind"] == "unapproved-shade" for f in flags))

    def test_diagonal_match_does_not_count(self):
        self.assertEqual([(p["x"], p["y"]) for p in check.isolated(["B..", ".B.", "..."], self.palette)],
                         [(0, 0), (1, 1)])

    def test_edge_pixels_can_match_horizontally_or_vertically(self):
        self.assertEqual(check.isolated(["BB.", ".B.", "..."], self.palette), [])

    def test_same_material_but_different_shade_is_still_flagged(self):
        self.assertEqual(len(check.isolated(["Bb"], self.palette)), 2)

    def test_palette_alias_with_the_same_rgba_counts_as_same_colour(self):
        self.assertEqual(check.isolated(["BX"], self.palette), [])

    def test_exterior_black_is_filtered_but_interior_black_remains(self):
        self.assertEqual(check.isolated([".O."], self.palette), [])
        self.assertEqual(len(check.isolated([".O."], self.palette, ignore_exterior_black=False)), 1)
        flags = check.isolated(["BBB", "BOB", "BBB"], self.palette)
        self.assertEqual([(p["x"], p["y"]) for p in flags], [(1, 1)])

    def test_diagonal_empty_does_not_hide_an_interior_black_pixel(self):
        flags = check.isolated([".BB", "BOB", "BBB"], self.palette)
        self.assertIn((1, 1), [(p["x"], p["y"]) for p in flags])


if __name__ == "__main__":
    unittest.main()
