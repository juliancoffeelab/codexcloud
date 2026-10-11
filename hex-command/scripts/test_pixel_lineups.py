"""Race trees own selected artwork; reject broken or ambiguous assignments."""

import json
from pathlib import Path
import tempfile
import shutil
import unittest
from unittest.mock import patch

import importlib.util
spec = importlib.util.spec_from_file_location('build_pixel_atlas', Path(__file__).parent / 'build-pixel-atlas.py')
art = importlib.util.module_from_spec(spec)
spec.loader.exec_module(art)
load_lineups = art.load_lineups


class RaceTrees(unittest.TestCase):
    def test_road_exits_allow_different_interiors(self):
        rows = ["R" * 64 if 28 <= y < 36 else "G" * 64 for y in range(64)]
        art.require_road_exits(rows, 9)
        rows[20] = "G" * 24 + "R" * 16 + "G" * 24
        art.require_road_exits(rows, 9)

    def test_road_exits_reject_offsets_and_width_changes(self):
        for start, end in [(24, 32), (29, 35)]:
            rows = [("R" if start <= y < end else "G") * 8 +
                    ("R" if 28 <= y < 36 else "G") * 56 for y in range(64)]
            with self.subTest(start=start, end=end), self.assertRaisesRegex(ValueError, "misaligned exit"):
                art.require_road_exits(rows, 9)

    def test_settlement_exits_distinguish_masonry_from_paths(self):
        rows = ["C" * 32 + "H" * 32 if 28 <= y < 36 else "C" * 64 for y in range(64)]
        surface = frozenset("RrH")
        art.require_road_exits(rows, 1, surface, label="village")
        rows[32] = "C" * 64
        with self.assertRaisesRegex(ValueError, "village 1: misaligned exit"):
            art.require_road_exits(rows, 1, surface, label="village")

    def test_roadless_settlement_has_no_edge_paths(self):
        rows = ["C" * 64 for _ in range(64)]
        surface = frozenset("RrH")
        art.require_road_exits(rows, 0, surface, label="village")
        rows[32] = "R" * 8 + "C" * 56
        with self.assertRaisesRegex(ValueError, "village 0: misaligned exit"):
            art.require_road_exits(rows, 0, surface, label="village")

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


class TerrainFloors(unittest.TestCase):
    def fixture(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        self.enterContext(patch.object(art, 'ART', root))
        floors = {'palettes': {'wood': {'G': [90, 110, 30, 255], 'g': [60, 80, 20, 255]},
                               'shore': {'G': [20, 90, 140, 255], 'g': [90, 150, 170, 255]}},
                  'defaults': {'wood': 'wood', 'shore': 'shore'}}
        (root / 'terrain').mkdir()
        path = root / 'terrain/floors.json'
        path.write_text(json.dumps(floors))
        for kind in ['wood', 'shore']:
            directory = root / 'terrain' / kind
            directory.mkdir()
            (directory / 'palette.json').write_text(json.dumps({'.': [0, 0, 0, 0],
                                                                'T': [7, 40, 15, 255]}))
            rows = ['GgT' + '.' * 61] + ['.' * 64] * 63
            for name in ['tile.txt', '18.txt']:
                (directory / name).write_text('\n'.join(rows) + '\n')
        return root, path, floors

    def test_floor_tuning_changes_only_its_surface_across_variants(self):
        root, path, floors = self.fixture()
        shore = art.rgba_pixels(root / 'terrain/shore/tile.txt')[2]
        matrix = root / 'terrain/wood/tile.txt'
        before = art.rgba_pixels(matrix)[2]
        source = matrix.read_bytes()
        floors['palettes']['wood'] = {'G': [50, 70, 20, 255], 'g': [30, 50, 10, 255]}
        path.write_text(json.dumps(floors))
        after = art.rgba_pixels(matrix)[2]
        self.assertEqual(list(after[:8]), floors['palettes']['wood']['G'] + floors['palettes']['wood']['g'])
        self.assertEqual(after[8:], before[8:])  # Foliage and alpha stay unchanged.
        self.assertEqual(matrix.read_bytes(), source)
        self.assertEqual(art.rgba_pixels(root / 'terrain/wood/18.txt')[2], after)
        self.assertEqual(art.rgba_pixels(root / 'terrain/shore/tile.txt')[2], shore)
        self.assertEqual(art.rgba_pixels(root / 'terrain/shore/tile.txt', floor='wood')[2], after)
        with self.assertRaisesRegex(ValueError, 'unknown floor'):
            art.rgba_pixels(matrix, floor='missing')

    def test_floor_rejects_missing_transparent_and_invalid_shades(self):
        root, path, floors = self.fixture()
        for invalid in [None, {'G': [1, 2, 3, 255]},
                        {'G': [1, 2, 3, 0], 'g': [1, 2, 3, 255]},
                        {'G': [1, 2, 300, 255], 'g': [1, 2, 3, 255]}]:
            with self.subTest(invalid=invalid):
                floors['palettes']['wood'] = invalid
                path.write_text(json.dumps(floors))
                with self.assertRaisesRegex(ValueError, 'wood floor: exactly two opaque'):
                    art.read_palette(root / 'terrain/wood/tile.txt')

    def test_local_material_palette_cannot_override_floor(self):
        root, path, floors = self.fixture()
        (root / 'terrain/wood/palette.json').write_text(json.dumps({'.': [0, 0, 0, 0],
                                                                  'G': [1, 2, 3, 255]}))
        with self.assertRaisesRegex(ValueError, 'floor shades belong in terrain/floors.json'):
            art.read_palette(root / 'terrain/wood/tile.txt')

    def test_prop_layouts_reject_unknown_pieces_and_painted_pixels_outside_hex(self):
        root, path, floors = self.fixture()
        piece = root / 'terrain/props/wood/seed'
        piece.mkdir(parents=True)
        (piece / 'tile.txt').write_text('T' + '.' * 63 + '\n' + ('.' * 64 + '\n') * 63)
        (piece / 'palette.json').write_text(json.dumps({'.': [0, 0, 0, 0], 'T': [7, 40, 15, 255]}))
        layout = root / 'terrain/wood/layouts.json'
        layout.write_text(json.dumps([[{'prop': 'seed', 'x': 2, 'y': 0}]]))
        self.assertEqual(art.load_terrain_layouts()['wood'][0],
                         [{'name': 'prop_wood_seed', 'x': 2, 'y': 0}])
        self.assertEqual(art.read_palette(piece / 'tile.txt')['G'], floors['palettes']['wood']['G'])
        for placement, error in [({'prop': 'missing', 'x': 2, 'y': 0}, 'unknown prop'),
                                 ({'prop': '../seed', 'x': 2, 'y': 0}, 'invalid prop placement'),
                                 ({'prop': 'seed', 'x': 3, 'y': 0}, 'outside the hex')]:
            layout.write_text(json.dumps([[placement]]))
            with self.subTest(placement=placement), self.assertRaisesRegex(ValueError, error):
                art.load_terrain_layouts()

    def test_directional_props_use_the_named_frame_footprint(self):
        root, path, floors = self.fixture()
        piece = root / 'terrain/props/wood/seed'
        piece.mkdir(parents=True)
        (piece / 'tile.txt').write_text('T' + '.' * 63 + '\n' + ('.' * 64 + '\n') * 63)
        direction = root / 'terrain/wood/ne-sw.txt'
        direction.write_text('.G' + '.' * 62 + '\n' + ('.' * 64 + '\n') * 63)
        variants = {'tile': [[{'prop': 'seed', 'x': 2, 'y': 0}]],
                    'ne-sw': [[{'prop': 'seed', 'x': 1, 'y': 0}]]}
        layout = root / 'terrain/wood/layouts.json'
        layout.write_text(json.dumps(variants))
        self.assertEqual(art.load_terrain_layouts()['wood_ne-sw'][0],
                         [{'name': 'prop_wood_seed', 'x': 1, 'y': 0}])
        variants['ne-sw'][0][0]['x'] = 2
        layout.write_text(json.dumps(variants))
        with self.assertRaisesRegex(ValueError, 'outside the hex'):
            art.load_terrain_layouts()
        layout.write_text(json.dumps({'../tile': variants['tile']}))
        with self.assertRaisesRegex(ValueError, 'unknown terrain frame'):
            art.load_terrain_layouts()


if __name__ == "__main__":
    unittest.main()
