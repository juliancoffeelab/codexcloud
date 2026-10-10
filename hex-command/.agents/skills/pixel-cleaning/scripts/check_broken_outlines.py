"""Report border gaps, outline topology, enclosed holes and material ownership.

Flags are review candidates, not instructions to automatically paint black.
Diagonal pixel-art steps and intentional facial marks require visual review.
"""
import argparse
import json
from collections import deque
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "scripts"))


def inspect(rows, outline='O', transparent='.', ownership=None):
    if not rows or not rows[0] or any(len(r) != len(rows[0]) for r in rows):
        raise ValueError('Matrix must be nonempty and rectangular')
    height, width = len(rows), len(rows[0])

    def neighbors(x, y):
        return [(xx, yy) for xx, yy in ((x-1, y), (x+1, y), (x, y-1), (x, y+1))
                if 0 <= xx < width and 0 <= yy < height]

    flags = []

    def flag(kind, x, y, **extra):
        flags.append(dict(kind=kind, x=x, y=y, symbol=rows[y][x], **extra))

    # Flood exterior transparency: remaining transparent components are holes.
    exterior = set()
    queue = deque((x, y) for y in range(height) for x in range(width)
                  if (x in (0, width-1) or y in (0, height-1)) and rows[y][x] == transparent)
    while queue:
        x, y = queue.popleft()
        if (x, y) in exterior:
            continue
        exterior.add((x, y))
        queue.extend((xx, yy) for xx, yy in neighbors(x, y)
                     if rows[yy][xx] == transparent and (xx, yy) not in exterior)
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            ns = neighbors(x, y)
            colors = [rows[yy][xx] for xx, yy in ns]
            if symbol == transparent:
                if (x, y) not in exterior:
                    flag('enclosed-hole', x, y)
                continue
            if symbol != outline:
                if transparent in colors or x in (0, width-1) or y in (0, height-1):
                    flag('missing-border', x, y)
            else:
                degree = colors.count(outline)
                if degree == 0:
                    flag('isolated-outline', x, y)
                elif degree == 1:
                    flag('outline-endpoint', x, y)
                if all(rows[yy][xx] in (outline, transparent)
                       for yy in range(max(0, y-1), min(height, y+2))
                       for xx in range(max(0, x-1), min(width, x+2))):
                    flag('unsupported-outline', x, y)
                if degree >= 3:
                    flag('thick-outline', x, y)
            if ownership:
                part = ownership['mask'][y][x]
                allowed = ownership['allowed'].get(part)
                if allowed is not None and symbol not in allowed:
                    flag('material-leak', x, y, part=part)
    # Small cardinally connected outline groups that the single-pixel rule misses.
    seen = set()
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            if symbol != outline or (x, y) in seen:
                continue
            queue = [(x, y)]
            seen.add((x, y))
            component = []
            while queue:
                xx, yy = queue.pop()
                component.append((xx, yy))
                for nx, ny in neighbors(xx, yy):
                    if rows[ny][nx] == outline and (nx, ny) not in seen:
                        seen.add((nx, ny))
                        queue.append((nx, ny))
            if 2 <= len(component) <= 6:
                for xx, yy in component:
                    flag('outline-fragment', xx, yy, component_size=len(component))
    # A diagonal pair with neither cardinal bridge is a broken step candidate.
    for y in range(height-1):
        for x in range(width-1):
            for a, b, c, d in [((x,y),(x+1,y+1),(x+1,y),(x,y+1)),
                                ((x+1,y),(x,y+1),(x,y),(x+1,y+1))]:
                if (rows[a[1]][a[0]] == rows[b[1]][b[0]] == outline
                        and rows[c[1]][c[0]] != outline and rows[d[1]][d[0]] != outline):
                    flag('diagonal-gap', *a, other=list(b), bridges=[list(c), list(d)])
    return flags


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('matrices', nargs='+', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--images-dir', type=Path, help='Default: beside each matrix')
    parser.add_argument('--only', choices=['missing-border', 'isolated-outline', 'outline-fragment', 'enclosed-hole'], help='Render only the selected rule')
    parser.add_argument('--ownership', type=Path, help='JSON with mask rows and allowed symbols by part')
    parser.add_argument('--fail-on-findings', action='store_true')
    args = parser.parse_args()
    ownership = json.loads(args.ownership.read_text()) if args.ownership else None
    report = {}
    for path in args.matrices:
        rows = path.read_text().splitlines()
        sidecar = path.with_suffix('.ownership.json')
        active_ownership = ownership or (json.loads(sidecar.read_text()) if sidecar.exists() else None)
        if active_ownership and (len(active_ownership['mask']) != len(rows) or
                          any(len(a) != len(b) for a, b in zip(active_ownership['mask'], rows))):
            parser.error('Ownership mask dimensions must match the matrix')
        report[str(path)] = inspect(rows, ownership=active_ownership)
        if args.only:
            report[str(path)] = [f for f in report[str(path)] if f['kind'] == args.only]
        from pixel_review_image import write_review
        image_dir = args.images_dir or path.parent
        image_path = image_dir / (path.parent.name + '-' + path.stem + '-edges.png' if args.images_dir else path.stem + '-edges.png')
        write_review(path, report[str(path)], image_path)
        print(f'Image: {image_path} (left: original; right: ALL selected findings)')
        counts = {}
        for f in report[str(path)]:
            counts[f['kind']] = counts.get(f['kind'], 0) + 1
        print(f'{path}: {json.dumps(counts, sort_keys=True)}')
    if args.output:
        args.output.write_text(json.dumps(report, indent=2) + '\n')
    return int(args.fail_on_findings and any(report.values()))


if __name__ == '__main__':
    raise SystemExit(main())
