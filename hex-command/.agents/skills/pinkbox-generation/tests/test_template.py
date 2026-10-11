"""Anatomy references must not silently widen a preserved output palette."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location('pinkbox_template', ROOT / '.agents/skills/pinkbox-generation/scripts/template.py')
template = importlib.util.module_from_spec(spec)
spec.loader.exec_module(template)


spec = importlib.util.spec_from_file_location('extract', ROOT / '.agents/skills/pinkbox-generation/scripts/extract.py')
extract = importlib.util.module_from_spec(spec)
spec.loader.exec_module(extract)


class OutputPalette(unittest.TestCase):
    def test_generation_and_extraction_reject_old_size_before_writing(self):
        (ROOT / 'tmp').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ROOT / 'tmp') as directory:
            directory = Path(directory)
            with self.assertRaisesRegex(ValueError, '64×64'):
                template.prepare(directory / 'template.png', ['wolf', 'lynx', 'bear'], ['New'], 48)
            self.assertFalse((directory / 'template.png').exists())
            manifest = {'version': 1, 'logicalSlot': 64, 'anchors': [{}, {}, {}],
                        'canvas': [1024, 1168], 'slots': [{'rect': [512, 576, 512, 512], 'native': 48, 'origin': [8, 8]}]}
            file = directory / 'manifest.json'
            file.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, '64×64'):
                extract.extract(directory / 'unused.png', file, directory / 'output')
            self.assertFalse((directory / 'output').exists())


    def test_style_anchors_do_not_add_output_colors(self):
        (ROOT / 'tmp').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ROOT / 'tmp') as directory:
            directory = Path(directory)
            source = template.matrix_path('wolf')
            original = template.render.read_palette(source)
            palette = directory / 'palette.json'
            palette.write_text(json.dumps(original))
            manifest = template.prepare(directory / 'locked.png', ['wolf', 'lynx', 'bear'], ['Wolf'], 64, palette)
            self.assertEqual(manifest['palette'], {**original, **template.render.faction_palette('player')})
            self.assertEqual(len(manifest['anchors']), 3)
            self.assertEqual(manifest['slots'][0]['native'], 64)
            self.assertEqual(manifest['outputPalette']['path'], str(palette.relative_to(ROOT)))

    def test_rejects_new_colors_and_faction_materials(self):
        (ROOT / 'tmp').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ROOT / 'tmp') as directory:
            directory = Path(directory)
            for number, extra in enumerate(({'X': [1, 2, 3, 255]}, {'^': [0, 0, 0, 255]})):
                palette = directory / f'palette-{number}.json'
                palette.write_text(json.dumps({'.': [0, 0, 0, 0], **extra}))
                with self.assertRaises(ValueError):
                    template.prepare(directory / f'invalid-{number}.png', ['wolf', 'lynx', 'bear'], ['Wolf'], 64, palette)
                self.assertFalse((directory / f'invalid-{number}.png').exists())


if __name__ == '__main__':
    unittest.main()
