"""Read-only detector bundle, faction previews and labeled pixel neighborhoods."""
import argparse
from collections import Counter, deque
import hashlib
import importlib.util
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[4]
from check_isolated_pixels import isolated
from check_broken_outlines import inspect
from pixel_review_image import write_review
spec = importlib.util.spec_from_file_location('render_pixel', ROOT / 'scripts/build-pixel-art.py')
render = importlib.util.module_from_spec(spec)
spec.loader.exec_module(render)
faction_palette = render.faction_palette
require_canvas = render.require_canvas


def panel(matrix, side, scale=8):
    w, h, rgba = render.rgba_pixels(matrix, side)
    sprite = Image.frombytes('RGBA', (w, h), bytes(rgba)).resize((w * scale, h * scale), Image.Resampling.NEAREST)
    background = Image.new('RGBA', sprite.size)
    draw = ImageDraw.Draw(background)
    for y in range(0, h * scale, scale * 4):
        for x in range(0, w * scale, scale * 4):
            draw.rectangle((x, y, x + scale * 4 - 1, y + scale * 4 - 1),
                           fill=(57, 65, 59) if (x // (scale * 4) + y // (scale * 4)) % 2 else (48, 56, 51))
    background.alpha_composite(sprite)
    return background


def previews(matrices, output):
    sides = ('player', 'enemy', 'debug') if any(path.stem == 'sprite' for path in matrices) else ('player',)
    tiles = [(path, side, panel(path, side)) for side in sides for path in matrices]
    width = max(t.width for _, _, t in tiles)
    height = max(t.height for _, _, t in tiles)
    board = Image.new('RGB', (len(matrices) * (width + 16), len(sides) * (height + 36)), (22, 32, 28))
    draw = ImageDraw.Draw(board)
    for i, (path, side, tile) in enumerate(tiles):
        x, y = i % len(matrices) * (width + 16), i // len(matrices) * (height + 36)
        draw.text((x + 4, y + 8), f'{path.parent.name}: {side}', fill='white')
        board.paste(tile, (x, y + 32))
    board.save(output)


def neighborhood(rows, palette, x, y, groups):
    h, w = len(rows), len(rows[0]); symbol = rows[y][x]
    group = next((set(v) for v in groups.values() if symbol in v), {symbol})
    neighbors = [{'x': xx, 'y': yy, 'symbol': rows[yy][xx], 'rgba': palette[rows[yy][xx]],
                  'diagonal': xx != x and yy != y}
                 for yy in range(max(0, y - 1), min(h, y + 2))
                 for xx in range(max(0, x - 1), min(w, x + 2)) if (xx, yy) != (x, y)]
    visited = {(x, y)}; queue = deque(visited)
    while queue:
        xx, yy = queue.popleft()
        for ny in range(max(0, yy - 1), min(h, yy + 2)):
            for nx in range(max(0, xx - 1), min(w, xx + 2)):
                if rows[ny][nx] in group and (nx, ny) not in visited:
                    visited.add((nx, ny)); queue.append((nx, ny))
    return {'neighbors8': neighbors, 'sameMaterialNeighbors8': sum(n['symbol'] in group for n in neighbors),
            'materialComponent8': len(visited)}


def contexts(rows, palette, points, output, radius):
    font = ImageFont.truetype('DejaVuSans.ttf', 11)
    scale = 8; n = radius * 2 + 1; cw, ch = n * scale + 16, n * scale + 56
    paths = []
    for start in range(0, len(points), 12):
        board = Image.new('RGB', (cw * 3, ch * 4), (22, 32, 28)); draw = ImageDraw.Draw(board)
        for i, finding in enumerate(points[start:start + 12]):
            x, y = finding['x'], finding['y']; bx, by = i % 3 * cw, i // 3 * ch
            draw.text((bx + 4, by + 4), f'({x},{y}) {rows[y][x]}', font=font, fill='white')
            draw.text((bx + 4, by + 18), ', '.join(finding['kinds'])[:27], font=font, fill='white')
            for dy in range(-radius, radius + 1):
                for dx in range(-radius, radius + 1):
                    xx, yy = x + dx, y + dy
                    color = (22, 32, 28)
                    if 0 <= yy < len(rows) and 0 <= xx < len(rows[0]):
                        rgba = palette[rows[yy][xx]]
                        color = tuple(rgba[:3]) if rgba[3] else ((57, 65, 59) if (xx // 4 + yy // 4) % 2 else (48, 56, 51))
                    px, py = bx + 4 + (dx + radius) * scale, by + 40 + (dy + radius) * scale
                    draw.rectangle((px, py, px + scale - 1, py + scale - 1), fill=color)
            px, py = bx + 4 + radius * scale, by + 40 + radius * scale
            draw.rectangle((px, py, px + scale - 1, py + scale - 1), outline=(255, 0, 225))
        target = output / f'neighbors-{start // 12 + 1:03}.png'; board.save(target); paths.append(target.name)
    return paths


def review(matrix, output, policy=None, points=None, radius=10):
    if not 1 <= radius <= 20:
        raise ValueError('Neighborhood radius must be between 1 and 20')
    output = output.resolve()
    if not output.is_relative_to(ROOT / 'tmp') or output.exists():
        raise ValueError('Use a new review directory under project tmp/')
    rows = render.read_matrix(matrix); palette = render.read_palette(matrix)
    require_canvas(rows, matrix)
    unit = matrix.stem == 'sprite'
    if unit:
        palette.update(faction_palette('enemy'))
    # Validate both factions without calling the renderer CLI, which can replay recipes.
    rendered = {side: render.rgba_pixels(matrix, side)[2] for side in ('player', 'enemy', 'debug')}
    faction_count = 0
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            start = (y * len(row) + x) * 4
            colors = [pixels[start:start + 4] for pixels in rendered.values()]
            if len({color[3] for color in colors}) != 1:
                raise ValueError(f'Faction alpha differs at {x},{y}')
            if symbol in '^*':
                faction_count += 1
            elif any(color != colors[0] for color in colors[1:]):
                raise ValueError(f'Fixed material changes across factions at {x},{y}')
    ownership_path = matrix.with_suffix('.ownership.json')
    ownership = json.loads(ownership_path.read_text()) if ownership_path.exists() else None
    if ownership and (len(ownership['mask']) != len(rows) or any(len(a) != len(b) for a, b in zip(ownership['mask'], rows))):
        raise ValueError('Ownership mask dimensions must match')
    groups = (policy or {}).get('allowed_shades', {})
    unapproved = {s for v in (policy or {}).get('unshaded_materials', {}).values() for s in v.get('extra_shades', [])}
    strict = isolated(rows, palette)
    edges = inspect(rows, ownership=ownership)
    flags = [dict(f, kind=f.get('kind', 'stray')) for f in strict] + edges
    material = isolated(rows, palette, allowed_shades=groups, unapproved_shades=unapproved) if policy else []
    flags += [dict(f, kind=f.get('kind', 'material-stray')) for f in material]
    output.mkdir(parents=True)
    write_review(matrix, strict, output / 'strays.png')
    if policy:
        write_review(matrix, material, output / 'material-strays.png')
    images = ['strays.png'] + (['material-strays.png'] if policy else [])
    for kind in sorted({f['kind'] for f in edges}):
        name = f'{kind}.png'; write_review(matrix, [f for f in edges if f['kind'] == kind], output / name); images.append(name)
    preview_name = 'factions.png' if unit else 'terrain.png'
    previews([matrix], output / preview_name)
    unique = {}
    for f in flags:
        entry = unique.setdefault((f['x'], f['y']), {'x': f['x'], 'y': f['y'], 'symbol': rows[f['y']][f['x']], 'kinds': []})
        if f['kind'] not in entry['kinds']:
            entry['kinds'].append(f['kind'])
    selected = list(unique.values()) if points is None else [unique.get((x, y), {'x': x, 'y': y, 'kinds': ['manual-review']}) for x, y in points]
    for f in selected:
        x, y = f['x'], f['y']
        if not (0 <= x < len(rows[0]) and 0 <= y < len(rows)):
            raise ValueError(f'Point outside matrix: {x},{y}')
        f.update(neighborhood(rows, palette, x, y, groups))
    pages = contexts(rows, palette, selected, output, radius)
    report = {'matrix': str(matrix.resolve()), 'sha256': hashlib.sha256(matrix.read_bytes()).hexdigest(),
              'palette_sha256': hashlib.sha256((matrix.parent / 'palette.json').read_bytes()).hexdigest(),
              'counts': dict(Counter(f['kind'] for f in flags)), 'findings': flags, 'pixels': selected,
              'factionValidation': {'identicalAlpha': True, 'fixedMaterialsIdentical': True,
                                    'factionPixels': faction_count,
                                    'ownershipNeedsVisualReview': True},
              'images': images + [preview_name] + pages,
              'warning': 'Candidates only. Cardinal rules flag legitimate diagonals and miss connected wrong-material patches. Inspect the complete render too.'}
    (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'output': str(output), 'counts': report['counts'], 'neighborhoodPages': len(pages)}))
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('matrix', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--policy', type=Path)
    parser.add_argument('--points', nargs='+', help='Native x,y coordinates; omit for ALL flagged coordinates')
    parser.add_argument('--radius', type=int, default=10)
    args = parser.parse_args()
    points = [tuple(map(int, p.split(','))) for p in args.points] if args.points else None
    review(args.matrix, args.output, json.loads(args.policy.read_text()) if args.policy else None, points, args.radius)
