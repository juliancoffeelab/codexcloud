"""Validate faction isolation and preservation of the extracted silhouettes."""
import importlib.util
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'scripts'))
spec = importlib.util.spec_from_file_location('render_pixel', ROOT / 'scripts/render-pixel.py')
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)
cleanup = json.loads((HERE / 'cleanup.json').read_text())
report = []
for level in range(1, 4):
    directory = ROOT / f'art/pixel/units/lich-commander-l{level}'
    matrix = directory / 'sprite.txt'
    rows = matrix.read_text().splitlines()
    assert len(rows) == 64 and all(len(row) == 64 for row in rows)
    original = json.loads((HERE / f'extracted-l{level}.json').read_text())['rows']
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            assert (symbol == '.') == (original[y][x] == '.'), (level, x, y, 'silhouette changed')
    palette = renderer.read_palette(matrix)
    assert '^' not in palette and '*' not in palette
    symbols = ''.join(rows)
    assert set(symbols) <= set(palette) | {'^', '*'}
    assert '^' in symbols and '*' in symbols
    assert not (directory / 'player.txt').exists() and not (directory / 'enemy.txt').exists()
    _, _, player = renderer.rgba_pixels(matrix, 'player')
    _, _, enemy = renderer.rgba_pixels(matrix, 'enemy')
    changed = 0
    for index, symbol in enumerate(symbols):
        left, right = player[index*4:index*4+4], enemy[index*4:index*4+4]
        assert left[3] == right[3]
        if symbol in '^*':
            assert list(left) == renderer.faction_palette('player')[symbol]
            assert list(right) == renderer.faction_palette('enemy')[symbol]
            assert left != right
            changed += 1
        else:
            assert left == right and list(left) == palette[symbol]
    # Green effects are confined to the staff head, never bone or cloth.
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            if symbol in 'Ee':
                assert (level == 2 and x >= 43 and y <= 14) or (level == 3 and 50 <= x <= 53 and 11 <= y <= 14)
    corrections = {(c['x'], c['y']): c for c in cleanup[str(level)]}
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            if symbol != original[y][x]:
                correction = corrections[x, y]
                assert correction['from'] == original[y][x] and correction['to'] == symbol
    report.append({'level': level, 'canvas': [64, 64], 'factionPixels': changed,
                   'fixedMaterialPixels': sum(c != '.' and c not in '^*' for c in symbols),
                   'corrections': len(corrections), 'silhouettePreserved': True,
                   'factionIsolation': True})
print(json.dumps(report, indent=2))
