"""Highlight every occurrence of exact RGB colors, one group per diagnostic image."""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[4]
PRESETS = Path(__file__).resolve().parents[1] / 'references/highlight-colors.json'
spec = importlib.util.spec_from_file_location('render_pixel', ROOT / 'scripts/build-pixel-art.py')
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)


def rgb(value):
    if not re.fullmatch(r'#[0-9a-fA-F]{6}', value):
        raise ValueError(f'Expected an exact #RRGGBB color, got {value!r}')
    return tuple(int(value[i:i + 2], 16) for i in (1, 3, 5))


def hex_rgb(color):
    return '#' + ''.join(f'{v:02x}' for v in color[:3])


def matching_pixels(width, pixels, colors):
    """No spatial, material, neighbor, ownership or palette-symbol exclusions."""
    return [
        {'x': (i // 4) % width, 'y': (i // 4) // width,
         'rgba': list(pixels[i:i + 4])}
        for i in range(0, len(pixels), 4)
        if pixels[i + 3] > 0 and tuple(pixels[i:i + 3]) in colors
    ]


def diagnostic_mapping(colors, occupied):
    """Keep source shades distinct and use colors absent from the original sprite."""
    if len(colors) > 2:
        raise ValueError('A check accepts at most two source colors')
    choices = [(0, 255, 255), (255, 0, 255), (176, 255, 0), (255, 225, 0),
               (255, 65, 65), (95, 80, 255), (255, 135, 0), (0, 255, 145)]
    used = set(occupied)
    mapping = {}
    # Dark-to-light order makes the legend easy to compare with source ramps.
    for source in sorted(colors, key=lambda c: (sum(c), c)):
        while choices:
            target = choices.pop(0)
            if target not in used:
                break
        else:
            raise ValueError('No unused high-contrast diagnostic color available')
        mapping[source] = target
        used.add(target)
    return mapping


def pair_groups(presets, names, present):
    """Split preset selections into checks of one or two exact RGB colors."""
    groups = {}
    for name in names:
        selected = list(dict.fromkeys(v.lower() for v in presets[name]['colors']
                                      if rgb(v) in present))
        if not selected:
            selected = presets[name]['colors'][:2]
        for i in range(0, len(selected), 2):
            key = name if len(selected) <= 2 else f'{name}-{i // 2 + 1:02d}'
            groups[key] = selected[i:i + 2]
    return groups


def highlight(matrix, groups, output, scale=8, side='enemy'):
    output = output.resolve()
    if not output.is_relative_to(ROOT / 'tmp') or output.exists():
        raise ValueError('Use a fresh output directory under project tmp/')
    if not 2 <= scale <= 16:
        raise ValueError('Use an integer preview scale between 2 and 16')
    for values in groups.values():
        if len({rgb(value) for value in values}) > 2:
            raise ValueError('A check accepts at most two source colors; use separate checks')
    rows = renderer.read_matrix(matrix)
    renderer.require_canvas(rows, matrix)
    width, height, pixels = renderer.rgba_pixels(matrix, side)
    original = Image.frombytes('RGBA', (width, height), bytes(pixels))
    output.mkdir(parents=True)
    counts = Counter(tuple(pixels[i:i + 3]) for i in range(0, len(pixels), 4)
                     if pixels[i + 3] > 0)
    report = {
        'matrix': str(matrix.resolve()),
        'matrix_sha256': hashlib.sha256(matrix.read_bytes()).hexdigest(),
        'palette_sha256': hashlib.sha256((matrix.parent / 'palette.json').read_bytes()).hexdigest(),
        'side': side,
        'palette_rgb_counts': {hex_rgb(c): n for c, n in sorted(counts.items())},
        'groups': {},
        'warning': 'Visual color isolation only. Counts are selected pixels, not defects. '
                   'Presets are explicit RGB lists, not material classifications.',
    }
    font = ImageFont.truetype('DejaVuSans.ttf', 16)
    cell, margin, header = width * scale, 16, 56
    for name, values in groups.items():
        colors = {rgb(v) for v in values}
        points = matching_pixels(width, pixels, colors)
        mapping = diagnostic_mapping({tuple(p['rgba'][:3]) for p in points}, counts)
        marked = original.copy()
        for point in points:
            # Preserve partial transparency; only the diagnostic RGB changes.
            target = mapping[tuple(point['rgba'][:3])]
            marked.putpixel((point['x'], point['y']), (*target, point['rgba'][3]))
        board_width = cell * 2 + margin * 3
        legend_columns = max(1, (board_width - margin * 2) // 230)
        legend_rows = (len(mapping) + legend_columns - 1) // legend_columns
        legend_top = height * scale + header + margin
        board = Image.new('RGB', (board_width, legend_top + legend_rows * 28 + margin), (27, 35, 29))
        draw = ImageDraw.Draw(board)
        draw.text((margin, 10), f'{matrix.parent.name}: original', font=font, fill='white')
        draw.text((cell + margin * 2, 10), f'{name}: {len(points)} pixels', font=font, fill='white')
        draw.text((margin, 33), 'Exact RGB selection; every occurrence, including stray pixels', font=font, fill='white')
        for col, picture in enumerate((original, marked)):
            background = Image.new('RGBA', (cell, height * scale), (48, 56, 51, 255))
            background.alpha_composite(picture.resize(background.size, Image.Resampling.NEAREST))
            board.paste(background, (margin + col * (cell + margin), header))
        for index, (source, target) in enumerate(mapping.items()):
            x = margin + index % legend_columns * 230
            y = legend_top + index // legend_columns * 28
            draw.rectangle((x, y, x + 14, y + 14), fill=source)
            draw.text((x + 20, y), hex_rgb(source), font=font, fill='white')
            draw.rectangle((x + 110, y, x + 124, y + 14), fill=target)
            draw.text((x + 130, y), hex_rgb(target), font=font, fill='white')
        board.save(output / f'{name}.png')
        matched = Counter(hex_rgb(p['rgba']) for p in points)
        report['groups'][name] = {
            'selected_colors': sorted(hex_rgb(c) for c in colors),
            'matched_color_counts': dict(sorted(matched.items())),
            'diagnostic_mapping': {hex_rgb(s): hex_rgb(t) for s, t in mapping.items()},
            'count': len(points), 'pixels': points, 'image': f'{name}.png',
        }
    (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


def main():
    presets = json.loads(PRESETS.read_text())
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('matrix', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--preset', action='append', choices=list(presets),
                        help='Separate pairwise checks for a preset; default: all presets')
    parser.add_argument('--color', action='append', default=[], metavar='#RRGGBB',
                        help='At most two distinct colors per custom check; all RGB aliases match')
    parser.add_argument('--scale', type=int, default=8)
    parser.add_argument('--side', choices=['player', 'enemy'], default='enemy')
    args = parser.parse_args()
    names = args.preset if args.preset is not None else ([] if args.color else list(presets))
    try:
        if len({rgb(v) for v in args.color}) > 2:
            raise ValueError('A check accepts at most two source colors')
        _, _, pixels = renderer.rgba_pixels(args.matrix, args.side)
        present = {tuple(pixels[i:i + 3]) for i in range(0, len(pixels), 4)
                   if pixels[i + 3] > 0}
        groups = pair_groups(presets, names, present)
        if args.color:
            groups['custom'] = args.color
        report = highlight(args.matrix, groups, args.output, args.scale, args.side)
    except ValueError as error:
        parser.error(str(error))
    for name, group in report['groups'].items():
        print(f'{name}: {group["count"]} pixels — {args.output / group["image"]}')


if __name__ == '__main__':
    main()
