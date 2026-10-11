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
entries, load_lineups, rows = art.entries, art.load_lineups, art.rows
unit_directories = art.unit_directories


class RaceTrees(unittest.TestCase):
    def test_replacements_use_current_models_without_old_sources(self):
        models = art.gameplay_models(load_lineups())
        self.assertEqual(models, {
            "swordsman": "leather-swordsman", "soldier": "swordsman",
            "spearman": "leather-spearman", "warden": "spearman", "mage": "storm-mage",
            "necromancer": "lich-commander-l1", "lich": "lich-commander-l2",
        })
        self.assertTrue({"soldier", "warden", "mage", "lich", "necromancer"}.isdisjoint(unit_directories()))

    def test_selected_rosters_have_one_owner_and_no_legacy_human_tail(self):
        lineups = load_lineups()
        self.assertEqual(set(lineups), {
            "humans", "elves", "dwarves", "merfolk", "skeletons", "zombies", "woodland", "beasts", "undead-animals",
        })
        humans = {unit["id"] for unit in entries(lineups["humans"])}
        self.assertTrue({"leather-swordsman", "leather-spearman", "adept", "rider"} <= humans)
        self.assertFalse({"soldier", "warden", "mage"} & humans)
        for lineup in lineups.values():
            self.assertTrue(all([unit["level"] for unit in row] == [1, 2, 3] for row in rows(lineup)))
        self.assertEqual(lineups["skeletons"]["commander"]["units"][1]["id"], "lich-commander-l2")

    def test_undead_animals_have_their_own_lineup_and_sources(self):
        lineups = load_lineups()
        families = {"skeletal-hound", "corpse-vulture", "skeletal-horse", "bone-dragon", "zombie-bear"}
        expected = {id for family in families for id in (family, family + "-l2", family + "-l3")}
        self.assertEqual({unit["id"] for unit in entries(lineups["undead-animals"])}, expected)
        for lineup in ("skeletons", "zombies"):
            self.assertTrue(expected.isdisjoint(unit["id"] for unit in entries(lineups[lineup])))
        sources = unit_directories()
        self.assertTrue(all(sources[id].parent.name == "undead-animals" for id in expected))
        self.assertIn("skeleton-rider", {unit["id"] for unit in entries(lineups["skeletons"])})

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
