"""Verify promoted elf sources without rebuilding or altering their artwork."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
records = json.loads(Path(__file__).with_name('sources.json').read_text())['units']
catalog = json.loads((ROOT / 'art/pixel/units/catalog.json').read_text())
for unit, record in records.items():
    directory = ROOT / 'art/pixel/units' / unit
    for name, key in [('sprite.txt', 'matrix_sha256'), ('palette.json', 'palette_sha256')]:
        if hashlib.sha256((directory / name).read_bytes()).hexdigest() != record[key]:
            raise ValueError(f'{unit}/{name}: changed since promotion; review and update sources.json')
    rows = (directory / 'sprite.txt').read_text().splitlines()
    canvas = [len(rows[0]), len(rows)]
    if canvas != record['canvas'] or canvas != catalog[unit]['canvas']:
        raise ValueError(f'{unit}: native canvas mismatch')
print(f'Verified {len(records)} reviewed elf matrices and palettes')
