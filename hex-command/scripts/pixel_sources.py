"""Shared unit matrices with two symbolic faction shades."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / 'art/pixel'
FACTIONS = ART / 'factions.json'


def faction_palette(side):
    palette = json.loads(FACTIONS.read_text())[side]
    if set(palette) != {'^', '*'} or any(
        not isinstance(rgba, list) or len(rgba) != 4 or rgba[3] != 255
        or any(type(channel) is not int or not 0 <= channel <= 255 for channel in rgba)
        for rgba in palette.values()
    ):
        raise ValueError(f'{side}: exactly two opaque faction shades are required')
    return palette


def legacy_text(path):
    path = Path(path)
    if path.name == 'palette.json' and path.with_name('sprite.txt').exists():
        palette = json.loads(path.read_text())
        defaults = {'T': faction_palette('player')['^'], 't': faction_palette('player')['*'],
                        'R': faction_palette('enemy')['^'], 'r': faction_palette('enemy')['*']}
        for symbol, colour in defaults.items():
            palette.setdefault(symbol, colour)
        return json.dumps(palette)
    if path.exists():
        return path.read_text()
    if path.name in ('player.txt', 'enemy.txt') and path.with_name('sprite.txt').exists():
        base, shadow = ('T', 't') if path.stem == 'player' else ('R', 'r')
        return path.with_name('sprite.txt').read_text().replace('^', base).replace('*', shadow)
    return path.read_text()


def normalize_pair(directory):
    """Convert recipe output into one shared matrix; fixed material pixels never change."""
    directory = Path(directory)
    player, enemy = directory / 'player.txt', directory / 'enemy.txt'
    if not (player.exists() and enemy.exists()):
        return False
    rows, other = player.read_text().splitlines(), enemy.read_text().splitlines()
    if not rows or not rows[0] or any(len(row) != len(rows[0]) for row in rows) or [len(row) for row in rows] != [len(row) for row in other]:
        raise ValueError(f'{directory}: faction matrices must share geometry')
    palette_path = directory / 'palette.json'
    palette = json.loads((palette_path if palette_path.exists() else ART / 'palette.json').read_text())
    if any(symbol in '^*' for row in rows + other for symbol in row):
        raise ValueError(f'{directory}: ^/* are reserved for shared faction channels')
    pairs = {(a, b) for left, right in zip(rows, other) for a, b in zip(left, right)
             if palette[a] != palette[b]}
    brightness = lambda colour: sum(colour[:3])
    levels = [brightness(palette[a]) for a, _ in pairs]
    midpoint = (min(levels) + max(levels)) / 2 if levels else 0
    shades = {}
    for a, b in pairs:
        if palette[a][3] != palette[b][3]:
            raise ValueError(f'{directory}: faction variants change transparency')
        shades[a, b] = '*' if a in 'tfcqojz' or brightness(palette[a]) < midpoint else '^'
    shared = [''.join(shades.get((a, b), a) for a, b in zip(left, right))
              for left, right in zip(rows, other)]
    used = set(''.join(shared))
    for raw in directory.glob('*.txt'):
        if raw.name not in ('player.txt', 'enemy.txt', 'sprite.txt'):
            used.update(raw.read_text())
    for symbol in ('T', 't', 'R', 'r'):
        if symbol not in used:
            palette.pop(symbol, None)
    (directory / 'sprite.txt').write_text('\n'.join(shared) + '\n')
    palette_path.write_text(json.dumps(palette, indent=2) + '\n')
    player.unlink()
    enemy.unlink()
    return True


def recipe_matches(path, expected):
    """Compare saved recipes without treating their obsolete faction RGBA as materials."""
    path = Path(path)
    current = legacy_text(path)
    if path.suffix != '.json':
        return current == expected
    actual, wanted = json.loads(current), json.loads(expected)
    if path.name == 'palette.json' and path.with_name('sprite.txt').exists():
        used = set(path.with_name('sprite.txt').read_text())
        for symbol in 'TtRr':
            if symbol not in used:
                actual.pop(symbol, None)
                wanted.pop(symbol, None)
    return actual == wanted
