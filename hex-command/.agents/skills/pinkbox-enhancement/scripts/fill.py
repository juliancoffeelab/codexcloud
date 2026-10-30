"""Fill explicitly selected connected material regions without changing anatomy."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import shutil
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / '.agents/skills/pixel-cleaning/scripts'))
from review import render, previews


def fill(matrix, specification, output):
    config = json.loads(specification.read_text())
    palette_path = matrix.parent / 'palette.json'
    for key, path in [('matrix_sha256', matrix), ('palette_sha256', palette_path)]:
        if config[key] != hashlib.sha256(path.read_bytes()).hexdigest():
            raise ValueError('Source changed since region selection')
    output = output.resolve()
    if not output.is_relative_to(ROOT / 'tmp') or output.exists():
        raise ValueError('Use a new output directory under project tmp/')
    original = render.require_canvas(render.read_matrix(matrix), matrix)
    rows = [list(row) for row in original]
    palette = json.loads(palette_path.read_text())
    palette.update(render.read_palette(matrix))
    for symbol, color in config.get('palette_additions', {}).items():
        if symbol in palette or symbol in '.O^*' or len(symbol) != 1 or len(color) != 4:
            raise ValueError('Additional material must have a new symbol and RGBA color')
        palette[symbol] = color
    changes = []
    selected = set()
    for region in config['regions']:
        mapping = region['mapping']
        if not mapping or not region.get('material') or not region.get('reason'):
            raise ValueError('Each region needs material, reason and color mapping')
        if any(s in '.O^*' or t in '.O^*' or t not in palette or s == t
               for s, t in mapping.items()):
            raise ValueError('Fill may recolor materials only; preserve outlines, alpha and faction cloth')
        x, y = region['seed']
        if not (0 <= y < len(rows) and 0 <= x < len(rows[0])) or original[y][x] not in mapping:
            raise ValueError('Seed must be inside a selected source material')
        seen = {(x, y)}
        queue = deque(seen)
        while queue:
            x, y = queue.popleft()
            for nx, ny in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                if (0 <= ny < len(rows) and 0 <= nx < len(rows[0])
                        and original[ny][nx] in mapping and (nx, ny) not in seen):
                    seen.add((nx, ny))
                    queue.append((nx, ny))
        if len(seen) != region['expected_pixels']:
            raise ValueError(f'{region["material"]}: region size changed; inspect it again')
        if selected & seen:
            raise ValueError('Selected regions overlap')
        selected.update(seen)
        for x, y in sorted(seen, key=lambda point: (point[1], point[0])):
            before = original[y][x]
            rows[y][x] = mapping[before]
            changes.append({'x': x, 'y': y, 'from': before, 'to': rows[y][x],
                            'material': region['material'], 'reason': region['reason']})
    output.mkdir(parents=True)
    before = output / 'before'
    before.mkdir()
    shutil.copyfile(matrix, before / matrix.name)
    shutil.copyfile(palette_path, before / 'palette.json')
    (output / matrix.name).write_text('\n'.join(''.join(row) for row in rows) + '\n')
    (output / 'palette.json').write_text(json.dumps(palette, indent=2) + '\n')
    (output / 'fill.json').write_text(json.dumps(config, indent=2) + '\n')
    (output / 'changes.json').write_text(json.dumps(changes, indent=2) + '\n')
    previews([matrix, output / matrix.name], output / 'comparison.png')
    print(json.dumps({'candidate': str(output / matrix.name), 'changedPixels': len(changes)}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('matrix', type=Path)
    parser.add_argument('specification', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    fill(args.matrix, args.specification, args.output)
