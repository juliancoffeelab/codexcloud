"""Anatomy references must not silently widen a preserved output palette."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location('pinkbox_template', ROOT / '.agents/skills/pinkbox-generation/scripts/template.py')
template = importlib.util.module_from_spec(spec)
spec.loader.exec_module(template)


spec = importlib.util.spec_from_file_location('extract', ROOT / '.agents/skills/pinkbox-generation/scripts/extract.py')
extract = importlib.util.module_from_spec(spec)
spec.loader.exec_module(extract)

spec = importlib.util.spec_from_file_location('enhancement', ROOT / '.agents/skills/pinkbox-enhancement/scripts/prepare.py')
enhancement = importlib.util.module_from_spec(spec)
spec.loader.exec_module(enhancement)


class OutputPalette(unittest.TestCase):
    def test_enhancement_preserves_pose_metadata_independently_of_color_lock(self):
        (ROOT / 'tmp').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ROOT / 'tmp') as directory:
            directory = Path(directory)
            source = directory / 'sprite.txt'
            source.write_text(('B' + '.' * 63 + '\n') * 64)
            materials = {'.': [0, 0, 0, 0], 'B': [99, 63, 37, 255]}
            palette = directory / 'palette.json'
            palette.write_text(json.dumps({**materials, 'flying': True}))
            styles = [str(source), str(source)]
            manifest = enhancement.prepare(str(source), directory / 'inherited.png', ['Revised'], styles)
            self.assertIs(manifest['slots'][0]['flying'], True)
            # A palette supplied just to lock colors must not replace pose metadata.
            locked = directory / 'colors.json'
            locked.write_text(json.dumps(materials))
            manifest = enhancement.prepare(str(source), directory / 'locked.png', ['Revised'], styles,
                                           output_palette=locked)
            self.assertIs(manifest['slots'][0]['flying'], True)
            manifest = enhancement.prepare(str(source), directory / 'grounded.png', ['Revised'], styles,
                                           flying=False)
            self.assertIs(manifest['slots'][0]['flying'], False)

    def test_extraction_requires_canvas_bottom_contact_unless_flying(self):
        (ROOT / 'tmp').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ROOT / 'tmp') as directory:
            directory = Path(directory)
            source = directory / 'generated.png'
            image = Image.new('RGBA', (128, 146), (255, 0, 255, 255))
            image.putpixel((90, 130), (99, 63, 37, 255))
            image.save(source)
            manifest = {'version': 1, 'logicalSlot': 64, 'anchors': [{}, {}, {}],
                        'canvas': [1024, 1168], 'palette': {'.': [0, 0, 0, 0],
                        'B': [99, 63, 37, 255], **extract.faction_palette('player')},
                        'slots': [{'label': 'Synthetic unit', 'rect': [512, 576, 512, 512],
                                   'native': 64, 'origin': [0, 0]}]}
            file = directory / 'manifest.json'
            file.write_text(json.dumps(manifest))
            rejected = directory / 'rejected'
            with self.assertRaisesRegex(ValueError, 'canvas bottom'):
                extract.extract(source, file, rejected)
            self.assertTrue((rejected / 'raw.png').exists())
            self.assertFalse((rejected / 'unit-1/sprite.txt').exists())
            manifest['slots'][0]['flying'] = True
            file.write_text(json.dumps(manifest))
            extract.extract(source, file, directory / 'flying')
            palette = json.loads((directory / 'flying/unit-1/palette.json').read_text())
            self.assertIs(palette['flying'], True)
            # Explicit metadata must work through the ordinary palette reader too.
            self.assertNotIn('flying', template.render.read_palette(directory / 'flying/unit-1/sprite.txt'))
            manifest['slots'][0]['flying'] = False
            file.write_text(json.dumps(manifest))
            image.putpixel((90, 135), (99, 63, 37, 255))
            image.save(source)
            extract.extract(source, file, directory / 'grounded')
            rows = (directory / 'grounded/unit-1/sprite.txt').read_text().splitlines()
            self.assertIn('B', rows[63])
            with self.assertRaisesRegex(ValueError, 'boolean'):
                template.render.require_grounded(rows, manifest['palette'], flying='yes')

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
