"""Check complete branch fills, protected materials, inlays and faction variants."""

import importlib.util
import json
from pathlib import Path
import unittest

file = Path(__file__).resolve().parents[1] / "art/pixel/reference/dwarf-upgrades/build.py"
spec = importlib.util.spec_from_file_location("dwarf_upgrades", file)
upgrades = importlib.util.module_from_spec(spec)
spec.loader.exec_module(upgrades)


class DwarfUpgradesTest(unittest.TestCase):
    def setUp(self):
        self.source, self.palette, self.parts, self.pixels = upgrades.load("dwarf-crossbow")
        self.rows, self.result_palette, self.gold = upgrades.derive(
            self.source, self.palette, self.parts, self.pixels, 3)

    def changed(self, x, y, symbol):
        rows = self.rows[:]
        rows[y] = rows[y][:x] + symbol + rows[y][x + 1:]
        return rows

    def test_all_eight_branches_fill_both_levels_and_preserve_faces_grips_and_weapons(self):
        inlays = json.loads((upgrades.HERE / "gold-inlays.json").read_text())
        for unit in upgrades.branches():
            source, palette, parts, pixels = upgrades.load(unit)
            points = {(p["x"], p["y"]) for p in inlays.get(unit, [])}
            for level in (2, 3):
                with self.subTest(unit=unit, level=level):
                    rows, result_palette, ornaments = upgrades.derive(source, palette, parts, pixels, level, points)
                    upgrades.verify(source, palette, rows, result_palette, parts, pixels, level, ornaments, points)

    def test_leftover_skin_or_leather_is_rejected_in_every_silver_surface(self):
        for part in self.parts:
            if part["role"] not in upgrades.ARMOUR:
                continue
            x, y = self.pixels[part["id"]][0]
            for symbol in "HhBbIiLl":
                with self.subTest(part=part["name"], symbol=symbol):
                    with self.assertRaisesRegex(ValueError, "Incomplete"):
                        upgrades.verify(self.source, self.palette, self.changed(x, y, symbol), self.result_palette,
                                        self.parts, self.pixels, 3, self.gold)

    def test_protected_skin_wood_and_beard_cannot_be_painted_as_plate(self):
        for part in self.parts:
            if part["role"] != "protected":
                continue
            x, y = self.pixels[part["id"]][0]
            with self.subTest(part=part["name"]):
                with self.assertRaisesRegex(ValueError, "protected"):
                    upgrades.verify(self.source, self.palette, self.changed(x, y, "P"), self.result_palette,
                                    self.parts, self.pixels, 3, self.gold)

    def test_eyes_and_contours_cannot_be_erased(self):
        for y, row in enumerate(self.source):
            if "O" in row:
                with self.assertRaisesRegex(ValueError, "contour, eye or silhouette"):
                    upgrades.verify(self.source, self.palette, self.changed(row.index("O"), y, "P"),
                                    self.result_palette, self.parts, self.pixels, 3, self.gold)
                break

    def test_enemy_changes_only_faction_cloth_and_keeps_gold(self):
        for unit in upgrades.branches():
            for level in (2, 3):
                directory = upgrades.UNITS / f"{unit}-l{level}"
                player = (directory / "player.txt").read_text()
                enemy = (directory / "enemy.txt").read_text()
                self.assertEqual(enemy, player.translate(str.maketrans("Tt", "Rr")))
                self.assertNotIn("T", enemy)
                self.assertNotIn("t", enemy)
                self.assertGreater(enemy.count("R") + enemy.count("r"), 0)
                if level == 3:
                    self.assertGreater(enemy.count("G") + enemy.count("Y"), 0)

    def test_actual_lineup_has_five_starters_and_eight_active_branches(self):
        lineup = json.loads((upgrades.PROJECT / "art/pixel/lineups/dwarves.json").read_text())
        self.assertEqual(len(lineup["trees"]), 5)
        self.assertEqual(len(upgrades.branches()), 8)
        catalog = json.loads((upgrades.UNITS / "catalog.json").read_text())
        starters = {u["base"]["id"] for u in lineup["trees"]}
        self.assertEqual({u for u, entry in catalog.items() if u.startswith("dwarf-") and entry["level"] == 1}, starters)
        self.assertTrue(set(lineup["poseReferences"]).isdisjoint(catalog))
        self.assertNotIn("dwarf-pickaxe-l2", catalog)

    def test_repaired_skin_flecks_are_gold_on_l3(self):
        for unit, records in json.loads((upgrades.HERE / "gold-inlays.json").read_text()).items():
            rows = (upgrades.UNITS / f"{unit}-l3/player.txt").read_text().splitlines()
            self.assertTrue(all(rows[p["y"]][p["x"]] == "G" for p in records))

    def test_hammerer_and_goat_forearms_are_covered_at_every_level(self):
        for unit in ("dwarf-hammerer", "dwarf-goat-rider"):
            source = upgrades.BASES / "bases" / unit
            reviewed = json.loads((source / "reviewed-specks.json").read_text())
            points = [(p["x"], p["y"]) for p in reviewed if p.get("feature") == "arm-covering"]
            self.assertGreater(len(points), 0)
            rows = (source / "player.mask").read_text().splitlines()
            self.assertTrue(all(rows[y][x] in "IiBb" for x, y in points))
            for level in (2, 3):
                for side in ("player", "enemy"):
                    rows = (upgrades.UNITS / f"{unit}-l{level}/{side}.txt").read_text().splitlines()
                    self.assertTrue(all(rows[y][x] in "PpSYG" for x, y in points))

    def test_rune_apprentice_stone_has_no_faction_colour_on_its_border(self):
        palette = json.loads((upgrades.UNITS / "dwarf-runesmith/palette.json").read_text())
        self.assertEqual(palette["z"], [46, 112, 132, 255])
        for side in ("player", "enemy"):
            rows = (upgrades.UNITS / f"dwarf-runesmith/{side}.txt").read_text().splitlines()
            self.assertTrue(all(rows[y][x] not in "TtRr" for y in range(26, 36) for x in range(14, 21)))
            self.assertTrue(any(c in "Ee" for row in rows[26:36] for c in row[14:21]))
            self.assertEqual(sum(row[14:21].count("z") for row in rows[26:36]), 16)


if __name__ == "__main__":
    unittest.main()
