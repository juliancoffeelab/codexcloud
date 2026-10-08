"""Reject leaked fills and incomplete rune armour/robe upgrades."""

import importlib.util
import json
from pathlib import Path
import unittest

file = Path(__file__).resolve().parents[1] / "art/pixel/reference/rune-line/build.py"
spec = importlib.util.spec_from_file_location("rune_line", file)
rune = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rune)


class RuneLineTest(unittest.TestCase):
    def setUp(self):
        self.source, self.palette, self.roles, self.pixels = rune.load()

    def test_part_fill_cannot_swallow_another_named_part(self):
        anchors = json.loads((rune.SOURCE / "anchors.json").read_text())
        self.assertEqual(len(anchors), 28)
        self.assertEqual(len({a["region"] for a in anchors}), 28)
        # Open the reviewed seam between apron and overskirt.
        rows = self.source[:]
        rows[45] = rows[45][:32] + "B" + rows[45][33:]
        with self.assertRaisesRegex(ValueError, "boundary|leaks"):
            rune.parts_module.verify_anchors(rows, anchors)

    def test_every_assigned_armour_and_robe_part_rejects_leftover_material(self):
        for branch in ("thundersmith", "stonespell"):
            for level in (2, 3):
                rows, palette, gold = rune.derive(self.source, self.palette, self.roles, self.pixels, branch, level)
                for role in self.roles:
                    if role["role"] not in ({"garment", "glove", "boot", "fastener"} if branch == "thundersmith" else {"garment"}):
                        continue
                    point = next((p for p in self.pixels[role["id"]] if p not in gold), None)
                    if point is None:
                        continue
                    x, y = point
                    for symbol in "HBIM":
                        changed = rows[:]
                        changed[y] = changed[y][:x] + symbol + changed[y][x + 1:]
                        with self.subTest(branch=branch, level=level, part=role["name"], symbol=symbol):
                            with self.assertRaisesRegex(ValueError, "Incomplete"):
                                rune.verify(self.source, self.palette, changed, palette, self.roles, self.pixels, branch, level, gold)

    def test_protected_face_hand_wood_and_crystal_cannot_be_recoloured(self):
        for branch in ("thundersmith", "stonespell"):
            rows, palette, gold = rune.derive(self.source, self.palette, self.roles, self.pixels, branch, 3)
            for role in self.roles:
                if role["role"] != "protected":
                    continue
                x, y = self.pixels[role["id"]][0]
                changed = rows[:]
                changed[y] = changed[y][:x] + "G" + changed[y][x + 1:]
                with self.subTest(branch=branch, part=role["name"]):
                    with self.assertRaisesRegex(ValueError, "protected"):
                        rune.verify(self.source, self.palette, changed, palette, self.roles, self.pixels, branch, 3, gold)

    def test_crystal_hammer_and_darker_robes_with_gold_and_enemy_cloth(self):
        hammer = next(r for r in self.roles if r["role"] == "hammer")
        for branch in ("thundersmith", "stonespell"):
            for level in (2, 3):
                rows, palette, gold = rune.derive(self.source, self.palette, self.roles, self.pixels, branch, level)
                if branch == "thundersmith":
                    self.assertTrue(all(rows[y][x] in "EeJ" for x, y in self.pixels[hammer["id"]]))
                else:
                    self.assertEqual(palette["U"], (rune.GREEN if level == 2 else rune.DARK_GREEN)["U"])
                self.assertEqual(bool(gold), level == 3)
                directory = rune.UNITS / f"dwarf-{branch}-l{level}"
                player = (directory / "player.txt").read_text()
                enemy = (directory / "enemy.txt").read_text()
                self.assertEqual(enemy, player.translate(str.maketrans("Tt", "Rr")))
                self.assertGreater(enemy.count("R") + enemy.count("r"), 0)
                self.assertEqual(palette["E"], rune.CRYSTAL["E"])
                self.assertEqual(palette["z"], [46, 112, 132, 255])

    def test_lineup_has_ten_branches_and_only_the_chosen_twenty_five_dwarves(self):
        lineup = json.loads((rune.PROJECT / "art/pixel/lineups/dwarves.json").read_text())
        self.assertEqual(len(lineup["trees"]), 5)
        self.assertEqual(sum(len(t["branches"]) for t in lineup["trees"]), 10)
        selected = {t["base"]["id"] for t in lineup["trees"]} | {u["id"] for t in lineup["trees"] for b in t["branches"] for u in b["units"]}
        catalog = json.loads((rune.UNITS / "catalog.json").read_text())
        self.assertEqual({u for u in catalog if u.startswith("dwarf-")}, selected)
        self.assertEqual(len(selected), 25)


if __name__ == "__main__":
    unittest.main()
