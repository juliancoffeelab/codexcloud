"""Prepare exactly three matrix references and one empty pink box; never edit art."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import string
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location('render_pixel', ROOT / 'scripts/build-pixel-atlas.py')
render = importlib.util.module_from_spec(spec)
spec.loader.exec_module(render)
unit_directory = render.unit_directory


def matrix_path(value):
    path = Path(value)
    if path.is_file():
        return path.resolve()
    path = unit_directory(value) / 'sprite.txt'
    if not path.is_file():
        raise ValueError(f'Unknown matrix or unit: {value}')
    return path


def prepare(output, anchors, labels, size=64, output_palette=None):
    if size != 64:
        raise ValueError('Character outputs must be exactly 64×64')
    output = Path(output).resolve()
    if not output.is_relative_to(ROOT / 'tmp'):
        raise ValueError('Templates belong in the project tmp/ directory')
    if len(labels) != 1:
        raise ValueError('Each pinkbox has exactly one output label; prepare separate templates for variants')
    if len(anchors) != 3:
        raise ValueError('Pinkbox requires exactly three references and one fill box')
    if output.exists() or output.with_suffix('.json').exists():
        raise ValueError('Use a new template filename; existing evidence is preserved')
    # Every grid cell is occupied: references first, one empty output last.
    columns, grid_rows = 2, 2
    cell, header, footer = 512, 32, 80
    width, height = columns * cell, grid_rows * (cell + header) + footer

    def cell_origin(index):
        return (index % columns * cell, index // columns * (cell + header))
    sheet = Image.new('RGBA', (width, height), (255, 0, 255, 255))
    draw = ImageDraw.Draw(sheet)
    font = ImageFont.truetype('DejaVuSans.ttf', 17)
    faction = render.faction_palette('player')
    palette = {'.': [0, 0, 0, 0], **faction}
    candidates = iter(string.ascii_letters + string.digits + '!#$%&()+,-/:;<=>?@[]{}_~')
    records = []
    for i, anchor in enumerate(anchors):
        path = matrix_path(anchor)
        rows = render.read_matrix(path)
        materials = render.read_palette(path)
        if path.stem != 'sprite':
            raise ValueError('Anchors must be shared sprite.txt matrices')
        if len(rows) != 64 or any(len(row) != 64 for row in rows):
            raise ValueError('Anchors must be exactly 64×64')
        for symbol in sorted(set(''.join(rows)) - {'.', '^', '*'}):
            color = materials[symbol]
            if not color[3]:
                continue
            r, g, b, _ = color
            if r > 110 and b > 90 and r - g > 65 and b - g > 65:
                raise ValueError(f'{path}: material color conflicts with removable magenta; choose another generation palette')
            if color in faction.values():
                raise ValueError(f'{path}: fixed material equals a faction color; resolve ownership first')
            if color in palette.values():
                continue
            target = symbol if symbol not in palette else next(c for c in candidates if c not in palette)
            palette[target] = color
        w, h, rgba = render.rgba_pixels(path, 'player')
        tile = Image.frombytes('RGBA', (w, h), bytes(rgba)).resize((w * 8, h * 8), Image.Resampling.NEAREST)
        x, y = cell_origin(i)
        sheet.alpha_composite(tile, (x + (cell - w * 8) // 2, y + header + (cell - h * 8) // 2))
        draw.text((x + 8, y + 6), f'ANCHOR {i + 1}: {anchors[i]}', font=font, fill='white')
        records.append({'matrix': str(path.relative_to(ROOT)), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    outputs = []
    for label in labels:
        x, y = cell_origin(len(anchors))
        draw.text((x + 8, y + 6), f'FILL: {label}', font=font, fill='white')
        outputs.append({'label': label, 'rect': [x, y + header, cell, cell],
                        'native': 64, 'origin': [0, 0]})
    if output_palette is not None:
        materials = json.loads(Path(output_palette).read_text())
        if materials.get('.') != [0, 0, 0, 0] or set(materials) & {'^', '*'}:
            raise ValueError('Output materials require transparency and separate faction shades')
        reference_colors = [c for anchor in anchors for c in render.read_palette(matrix_path(anchor)).values()]
        if any(c not in reference_colors for c in materials.values()):
            raise ValueError('Output materials must come from the supplied matrix references')
        palette = {**materials, **faction}
    colors = [(s, c) for s, c in palette.items() if c[3]]
    columns = width // 60
    if len(colors) > columns * 2:
        raise ValueError('Too many colors; choose simpler anchors')
    for i, (symbol, color) in enumerate(colors):
        x, y = 8 + i % columns * 60, height - footer + 8 + i // columns * 32
        draw.rectangle((x, y, x + 18, y + 18), fill=tuple(color))
        draw.text((x + 23, y), symbol, font=font, fill='white')
    output.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(output)
    manifest = {'version': 1, 'template': str(output.relative_to(ROOT)), 'canvas': [width, height],
                'anchors': records, 'palette': palette, 'slots': outputs,
                'background': [255, 0, 255, 255], 'logicalSlot': 64}
    if output_palette is not None:
        material_path = Path(output_palette).resolve()
        manifest['outputPalette'] = {'path': str(material_path.relative_to(ROOT)),
                                    'sha256': hashlib.sha256(material_path.read_bytes()).hexdigest()}
    output.with_suffix('.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(output)
    return manifest


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--anchors', nargs=3, required=True)
    parser.add_argument('--label', '--labels', dest='labels', nargs=1, required=True)
    parser.add_argument('--size', type=int, choices=[64], default=64)
    parser.add_argument('--output-palette', type=Path, help='Restrict output swatches and extraction to these original material colors')
    args = parser.parse_args()
    prepare(args.output, args.anchors, args.labels, args.size, args.output_palette)
