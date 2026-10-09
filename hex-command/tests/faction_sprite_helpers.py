"""Check rendered faction colours against the shared editable sprite."""

import importlib.util
from pathlib import Path
import sys

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))
spec = importlib.util.spec_from_file_location("render_pixel", SCRIPTS / "render-pixel.py")
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)


def assert_faction_materials(test, directory):
    matrix = directory / "sprite.txt"
    rows = matrix.read_text().splitlines()
    symbols = "".join(rows)
    test.assertTrue(set(symbols) & set("^*"), f"{directory.name}: missing faction cloth")
    palette = renderer.read_palette(matrix)
    test.assertNotIn("^", palette)
    test.assertNotIn("*", palette)
    width, height, player = renderer.rgba_pixels(matrix, "player")
    enemy_width, enemy_height, enemy = renderer.rgba_pixels(matrix, "enemy")
    test.assertEqual((width, height), (enemy_width, enemy_height))
    for index, symbol in enumerate(symbols):
        with test.subTest(unit=directory.name, pixel=index, symbol=symbol):
            start = index * 4
            left, right = player[start:start + 4], enemy[start:start + 4]
            test.assertEqual(left[3], right[3])
            if symbol in "^*":
                test.assertEqual(list(left), renderer.faction_palette("player")[symbol])
                test.assertEqual(list(right), renderer.faction_palette("enemy")[symbol])
                test.assertNotEqual(left, right)
            else:
                test.assertEqual(list(left), palette[symbol])
                test.assertEqual(left, right)
    return rows
