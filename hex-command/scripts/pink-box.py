"""Build a disposable generation reference from three approved unit matrices.

Usage: python3 scripts/pink-box.py tmp/lich-template.png skeleton revenant lich
"""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess

spec = importlib.util.spec_from_file_location('render_pixel', Path(__file__).with_name('render-pixel.py'))
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output', type=Path)
parser.add_argument('anchors', nargs=3)
args = parser.parse_args()
project = Path(__file__).resolve().parents[1]
output = args.output.resolve()
if not output.is_relative_to(project / 'tmp'):
    parser.error('Generation references belong under this project’s ignored tmp/ directory')
pixels = bytearray([255, 0, 255, 255] * (1024 * 1024))
colours = set()
for slot, unit in enumerate(args.anchors):
    directory = project / 'art/pixel/units' / unit
    if directory.parent != project / 'art/pixel/units':
        parser.error('Anchor names must be unit directory names')
    matrix = directory / 'sprite.txt'
    width, height, native = renderer.rgba_pixels(matrix, 'player')
    if width > 64 or height > 64:
        parser.error('Anchors must fit a 64×64 native frame')
    ox = (slot % 2) * 512 + (512 - width * 8) // 2
    oy = (slot // 2) * 512 + (512 - height * 8) // 2
    for y in range(height):
        for x in range(width):
            rgba = native[(y * width + x) * 4:(y * width + x + 1) * 4]
            if not rgba[3]:
                continue
            colours.add(tuple(rgba))
            for dy in range(8):
                for dx in range(8):
                    start = ((oy + y * 8 + dy) * 1024 + ox + x * 8 + dx) * 4
                    pixels[start:start + 4] = rgba
# Always supply both current faction shades, even if an anchor uses only one.
colours.update(tuple(c) for c in renderer.faction_palette('player').values())
colours = sorted(colours)
for index, rgba in enumerate(colours):
    x = 8 + (index % 32) * 15
    y = 468 + (index // 32) * 16
    if y + 12 > 510:
        parser.error('Too many anchor colours; choose a smaller material palette')
    for dy in range(12):
        for dx in range(12):
            start = ((y + dy) * 1024 + x + dx) * 4
            pixels[start:start + 4] = bytes(rgba)
output.parent.mkdir(parents=True, exist_ok=True)
output.write_bytes(renderer.png_bytes(1024, 1024, pixels))
subprocess.run(['magick', str(output), '-draw', "fill '#ffffff' font 'DejaVu-Sans' font-size 18 text 16,28 'STYLE ANCHOR 1' text 528,28 'STYLE ANCHOR 2' text 16,540 'STYLE ANCHOR 3' text 528,540 'NEW DESIGN — FILL HERE'", str(output)], check=True)
output.with_suffix('.json').write_text(json.dumps({'anchors': args.anchors, 'faction': renderer.faction_palette('player'), 'swatches': colours, 'nativeSlot': [64, 64], 'generationSlot': 'bottom-right'}, indent=2) + '\n')
print(output.relative_to(project))
