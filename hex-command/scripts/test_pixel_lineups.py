"""Race trees own selected artwork; reject broken or ambiguous assignments."""

import json
from pathlib import Path
import tempfile
import shutil
import unittest

import importlib.util
spec = importlib.util.spec_from_file_location('build_pixel_atlas', Path(__file__).parent / 'build-pixel-atlas.py')
art = importlib.util.module_from_spec(spec)
spec.loader.exec_module(art)
load_lineups = art.load_lineups


class RaceTrees(unittest.TestCase):
    def test_road_composition_keeps_transparency_and_joins_materials(self):
        parts = [["." * 64 for _ in range(64)] for _ in range(6)]
        parts[0][30] = "r" + "." * 63
        parts[1][30] = "Rr" + "." * 62
        palette = {'.': [0, 0, 0, 0], 'R': [200, 160, 90, 255],
                   'r': [130, 100, 60, 255], 'C': [230, 210, 170, 255]}
        frames = art.road_frames(parts, palette)
        _, _, pixels = frames['terrain_road_3']
        offset = 30 * 64 * 4
        self.assertEqual(pixels[offset:offset + 8], bytes(palette['R'] + palette['r']))
        self.assertEqual(pixels[offset + 8:offset + 12], bytes(palette['.']))
        self.assertEqual(frames['terrain_road_1'][2][offset:offset + 4], bytes(palette['r']))

    def fixture(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        trees = root / "units/lineups"
        trees.mkdir(parents=True)
        units = trees.parent / "race"
        units.mkdir(parents=True)
        members = [{"id": f"unit-{level}", "name": "Unit", "level": level, "source": f"unit-{level}"} for level in (1, 2, 3)]
        for member in members:
            source = units / member["source"]
            source.mkdir(parents=True)
            (source / "sprite.txt").write_text(("." * 64 + "\n") * 64)
        lineup = {"race": "race", "trees": [{"base": members[0], "branches": [{"role": "Branch", "units": members[1:]}]}]}
        return trees, units, lineup

    def test_rejects_wrong_tier_in_branch(self):
        trees, units, lineup = self.fixture()
        lineup["trees"][0]["branches"][0]["units"][1]["level"] = 2
        (trees / "race.json").write_text(json.dumps(lineup))
        with self.assertRaisesRegex(ValueError, "L2 followed by L3"):
            load_lineups(trees)

    def test_rejects_unit_owned_by_two_races(self):
        trees, units, lineup = self.fixture()
        (trees / "race.json").write_text(json.dumps(lineup))
        shutil.copytree(units, trees.parent / "other")
        lineup["race"] = "other"
        (trees / "other.json").write_text(json.dumps(lineup))
        with self.assertRaisesRegex(ValueError, "already assigned"):
            load_lineups(trees)

    def test_rejects_missing_artwork(self):
        trees, units, lineup = self.fixture()
        (units / "unit-3/sprite.txt").unlink()
        (trees / "race.json").write_text(json.dumps(lineup))
        with self.assertRaisesRegex(ValueError, "missing matrix"):
            load_lineups(trees)

    def test_rejects_48px_registered_unit(self):
        trees, units, lineup = self.fixture()
        (units / "unit-1/sprite.txt").write_text(("." * 48 + "\n") * 48)
        (trees / "race.json").write_text(json.dumps(lineup))
        with self.assertRaisesRegex(ValueError, "64×64"):
            load_lineups(trees)

    def test_rejects_source_outside_its_race(self):
        trees, units, lineup = self.fixture()
        lineup["trees"][0]["base"]["source"] = "../other/l1"
        (trees / "race.json").write_text(json.dumps(lineup))
        with self.assertRaisesRegex(ValueError, "inside its race folder"):
            load_lineups(trees)

    def test_rejects_unregistered_unit(self):
        trees, units, lineup = self.fixture()
        (trees / "race.json").write_text(json.dumps(lineup))
        extra = units / "unregistered"
        extra.mkdir(parents=True)
        (extra / "sprite.txt").write_text(".\n")
        with self.assertRaisesRegex(ValueError, "Unregistered unit"):
            load_lineups(trees)

    def test_rejects_nested_unit_source(self):
        trees, units, lineup = self.fixture()
        lineup["trees"][0]["base"]["source"] = "family/l1"
        (trees / "race.json").write_text(json.dumps(lineup))
        with self.assertRaisesRegex(ValueError, "own unit folder"):
            load_lineups(trees)

    def test_rejects_nested_tree_definition(self):
        trees, units, lineup = self.fixture()
        (trees / "race.json").write_text(json.dumps(lineup))
        (trees / "race").mkdir()
        with self.assertRaisesRegex(ValueError, "Race trees must be flat"):
            load_lineups(trees)


if __name__ == "__main__":
    unittest.main()
