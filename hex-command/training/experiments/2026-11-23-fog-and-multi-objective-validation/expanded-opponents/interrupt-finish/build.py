"""Apply returning fog to exact validated Marlow Cut arena fixtures."""
from copy import deepcopy
import json
from pathlib import Path
import shutil

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
ORIGINAL = ROOT / 'tmp/rl/multi-objective-validation/interrupt-finish'
BASE = ORIGINAL / 'validated/catalog'
CATALOG = OUT / 'catalog'
POST = {'col': 4, 'row': 2}
STARTS = ('interrupt', 'finish')
VISIBILITY = ('clear', 'player-fog', 'both-fog')
FOG_EFFECT = 'fog-reveal-3'


def read(path):
    return json.loads(path.read_text())


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + '\n')


def arena(start, visibility):
    original = f'rl-marlow-cut-pass-{start}'
    return original if visibility == 'clear' else f'{original}-{visibility}'


def location(root, ident):
    return root / 'game/scenario' / ident


def build():
    shutil.copytree(BASE, CATALOG, dirs_exist_ok=True)
    manifest = read(CATALOG / 'manifest.json')
    simulation = read(CATALOG / 'simulation/arena.json')
    for start in STARTS:
        source_ident = arena(start, 'clear')
        source = location(BASE, source_ident)
        source_map = read(source / 'map.json')
        source_roster = read(source / 'roster.json')
        source_rule = read(source / 'simulation-rule.json')
        assert source_rule['holdTurns'] == 6 and source_rule['territory'] == [POST]
        for visibility in VISIBILITY[1:]:
            ident = arena(start, visibility)
            folder = location(CATALOG, ident)
            field = deepcopy(source_map)
            field['mapId'] = ident
            for tile in field['tiles']:
                tile['environmentEffects'].append(FOG_EFFECT)
            roster = deepcopy(source_roster)
            for group in roster['visionGroups']:
                blinded = group['id'] == 'player' or visibility == 'both-fog'
                if blinded:
                    group['sight'].update(range=3, unlimited=False, penetrates=[])
            assert all(not group['explored'] for group in roster['visionGroups'])
            definition = deepcopy(source_rule)
            definition['id'] = ident
            definition['variants'][0]['map'] = ident
            write(folder / 'map.json', field)
            write(folder / 'roster.json', roster)
            write(folder / 'simulation-rule.json', definition)
            shutil.copy2(source / 'scenario.json', folder / 'scenario.json')
            manifest['scenarios'].append(str((folder / 'scenario.json').relative_to(CATALOG)))
            simulation['scenarios'].append(definition)
    write(CATALOG / 'manifest.json', manifest)
    write(CATALOG / 'simulation/arena.json', simulation)
    print(CATALOG)


if __name__ == '__main__':
    build()
