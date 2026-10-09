"""Fast checks for faction consolidation and atlas inputs."""
import importlib.util
import json
from pathlib import Path
import tempfile
import subprocess
import unittest

from pixel_sources import faction_palette, legacy_text, normalize_pair, recipe_matches

spec = importlib.util.spec_from_file_location('render_pixel', Path(__file__).with_name('render-pixel.py'))
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)


class SharedSources(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)

    def write_pair(self, player, enemy, palette):
        for side, rows in [('player', player), ('enemy', enemy)]:
            (self.directory / f'{side}.txt').write_text(rows)
        (self.directory / 'palette.json').write_text(json.dumps(palette))

    def test_five_shades_clamp_and_keep_fixed_materials(self):
        palette = {'.': [0, 0, 0, 0], 'W': [90, 70, 40, 255]}
        for index, (a, b) in enumerate(zip('ABCDE', 'abcde')):
            palette[a] = [0, 30 + index * 30, 50 + index * 30, 255]
            palette[b] = [30 + index * 30, 0, 0, 255]
        self.write_pair('.ABCDEW\n', '.abcdeW\n', palette)
        self.assertTrue(normalize_pair(self.directory))
        matrix = self.directory / 'sprite.txt'
        self.assertEqual(matrix.read_text(), '.**^^^W\n')
        self.assertFalse(normalize_pair(self.directory))
        for side in ('player', 'enemy', 'debug'):
            width, height, pixels = renderer.rgba_pixels(matrix, side)
            self.assertEqual((width, height), (7, 1))
            self.assertEqual(list(pixels[:4]), [0, 0, 0, 0])
            self.assertEqual(list(pixels[-4:]), palette['W'])
            self.assertEqual(list(pixels[4:8]), faction_palette(side)['*'])
            self.assertEqual(list(pixels[12:16]), faction_palette(side)['^'])
        self.assertFalse((self.directory / 'player.txt').exists())
        self.assertEqual(legacy_text(self.directory / 'enemy.txt'), '.rrRRRW\n')

    def test_fixed_team_like_symbol_and_raw_palette_survive(self):
        palette = {'.': [0, 0, 0, 0], 'T': [99, 100, 101, 255],
                   't': [0, 20, 30, 255], 'r': [30, 0, 0, 255]}
        self.write_pair('Tt\n', 'Tr\n', palette)
        (self.directory / 'extracted.txt').write_text('tT\n')
        normalize_pair(self.directory)
        self.assertEqual((self.directory / 'sprite.txt').read_text(), 'T*\n')
        self.assertEqual(renderer.rgba_pixels(self.directory / 'sprite.txt', 'enemy')[2][:4], bytes(palette['T']))
        self.assertEqual(json.loads((self.directory / 'palette.json').read_text())['t'], palette['t'])
        self.assertEqual(renderer.rgba_pixels(self.directory / 'extracted.txt')[2][:4], bytes(palette['t']))

    def test_saved_node_recipe_can_read_copy_and_render_virtual_sources(self):
        palette = {'.': [0, 0, 0, 0], 'T': [2, 70, 90, 255], 'R': [90, 2, 2, 255]}
        self.write_pair('.T\n', '.R\n', palette)
        normalize_pair(self.directory)
        bridge = Path(__file__).with_name('pixel-fs.cjs').resolve()
        script = """
const fs = require(process.argv[1]);
const path = require('node:path');
const assert = require('node:assert/strict');
const directory = process.argv[2];
assert.equal(fs.readFileSync(path.join(directory, 'enemy.txt'), 'utf8'), '.R\\n');
assert(fs.existsSync(path.join(directory, 'player.txt')));
fs.copyFileSync(path.join(directory, 'palette.json'), path.join(directory, 'copied.json'));
assert(JSON.parse(fs.readFileSync(path.join(directory, 'copied.json'))).T);
const png = fs.readFileSync(path.join(directory, 'enemy.png'));
assert.equal(png.subarray(1, 4).toString(), 'PNG');
"""
        subprocess.run(['node', '-e', script, str(bridge), str(self.directory)], check=True)
        self.assertFalse((self.directory / 'player.txt').exists())

    def test_reject_different_silhouettes(self):
        self.write_pair('T\n', '.\n', {'.': [0, 0, 0, 0], 'T': [0, 70, 90, 255]})
        with self.assertRaisesRegex(ValueError, 'transparency'):
            normalize_pair(self.directory)
        self.assertTrue((self.directory / 'player.txt').exists())

    def test_recipe_comparison_checks_materials_but_ignores_old_faction_ramp(self):
        palette = {'.': [0, 0, 0, 0], 'T': [2, 60, 90, 255], 'R': [90, 3, 2, 255], 'W': [60, 40, 20, 255]}
        self.write_pair('TW\n', 'RW\n', palette)
        normalize_pair(self.directory)
        path = self.directory / 'palette.json'
        self.assertTrue(recipe_matches(path, json.dumps(palette)))
        palette['W'] = [0, 0, 0, 255]
        self.assertFalse(recipe_matches(path, json.dumps(palette)))


if __name__ == '__main__':
    unittest.main()
