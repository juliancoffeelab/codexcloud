"""Read-only verification of promoted commander matrices and tier geometry."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
records = json.loads((HERE / 'sources.json').read_text())['units']
catalog = json.loads((ROOT / 'art/pixel/units/catalog.json').read_text())
for id, record in records.items():
    directory = ROOT / 'art/pixel/units' / id
    for file, key in [('sprite.txt', 'matrix_sha256'), ('palette.json', 'palette_sha256')]:
        assert hashlib.sha256((directory / file).read_bytes()).hexdigest() == record[key], id
    rows = (directory / 'sprite.txt').read_text().splitlines()
    assert [len(rows[0]), len(rows)] == record['canvas'] == catalog[id]['canvas'], id
    assert catalog[id]['role'] == 'commander'
    assert not {'^', '*'} & json.loads((directory / 'palette.json').read_text()).keys()
    base = 'dwarf-forge-lord' if record['race'] == 'dwarf' else 'elf-banner-lord'
    baseline = (ROOT / 'art/pixel/units' / base / 'sprite.txt').read_text().splitlines()
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            old = baseline[y][x]
            crown = record['race'] == 'elf' and record['level'] == 3 and 22 <= x <= 32 and 6 <= y <= 10
            piping = record['race'] == 'elf' and record['level'] == 3 and x == 45 and 13 <= y <= 17
            if not crown:
                assert (symbol == '.') == (old == '.'), (id, x, y, 'alpha changed')
                assert (symbol == 'O') == (old == 'O'), (id, x, y, 'outline changed')
            if not piping:
                assert (symbol in '^*') == (old in '^*'), (id, x, y, 'faction ownership changed')
for race, filename in [('dwarf', 'dwarves'), ('elf', 'elves')]:
    lineup = json.loads((ROOT / f'art/pixel/lineups/{filename}.json').read_text())
    assert len(lineup['trees']) == 5
    assert [unit['level'] for unit in lineup['commander']['units']] == [1, 2, 3]
    assert all(records[unit['id']]['race'] == race for unit in lineup['commander']['units'])
print('Verified six commander sources, protected tier geometry and five ordinary roots per race')
