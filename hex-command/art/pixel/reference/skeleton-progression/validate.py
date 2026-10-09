"""Check servant anatomy, restrained progression and faction ownership."""
import hashlib
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
changes = json.loads((HERE / 'changes.json').read_text())
lineup = json.loads((ROOT / 'art/pixel/lineups/skeletons.json').read_text())
assert [len(tree['branches']) for tree in lineup['trees']] == [2, 2, 1, 1, 1]
report = []
for id, change in changes.items():
    directory = ROOT / 'art/pixel/units' / id
    matrix = directory / 'sprite.txt'
    rows = matrix.read_text().splitlines()
    normalized = []
    for y, row in enumerate(rows):
        normalized.append(''.join('.' if id.startswith('skeleton-impact-caster') and y < 18 and x < 23 else 'F' if c in '^*' else change.get('darkSkull', {}).get('boneSymbols', 'Ii')[0] if c == 'D' else change.get('darkSkull', {}).get('boneSymbols', 'Ii')[1] if c == 'd' else c for x, c in enumerate(row)))
    assert hashlib.sha256('\n'.join(normalized).encode()).hexdigest() == change['protectedAnatomyHash'], id
    if 'darkSkull' in change:
        left, top, right, bottom = change['darkSkull']['bounds']
        assert not any(c in change['darkSkull']['boneSymbols'] for row in rows[top:bottom] for c in row[left:right]), id
    palette = renderer.read_palette(matrix)
    assert '^' not in palette and '*' not in palette
    _, _, player = renderer.rgba_pixels(matrix, 'player')
    _, _, enemy = renderer.rgba_pixels(matrix, 'enemy')
    symbols = ''.join(rows)
    for index, symbol in enumerate(symbols):
        p, e = player[index*4:index*4+4], enemy[index*4:index*4+4]
        assert p[3] == e[3]
        if symbol in '^*':
            assert list(p) == renderer.faction_palette('player')[symbol]
            assert list(e) == renderer.faction_palette('enemy')[symbol]
        else:
            assert p == e
    if id.startswith('skeleton-impact-caster'):
        assert all(c in '.OWw' for row in rows[8:18] for c in row[:23]), id
    report.append({'id': id, 'level': change['level'], 'anatomyPreserved': True,
                   'factionPixels': sum(c in '^*' for c in symbols),
                   'staffHeadSimplified': bool(change['staffException'])})
print(json.dumps(report, indent=2))
