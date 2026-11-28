"""Build arena fixtures against the frozen native rules in an ignored catalog."""
import argparse
import copy
import json
import shutil
from pathlib import Path

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
BASE = RUN / 'catalog'
TEMPLATE = BASE / 'game/scenario/combat-sluice'
POST = (4, 2)
MODES = ('interrupt', 'finish')


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def hex_at(position):
    return dict(zip(('col', 'row'), position))


def unit(template, ident, kind, position, hp='full', leader=False):
    return dict(template, id=f'u{ident}', type=kind, tile=hex_at(position), hp=hp,
                role='leader' if leader else None, character=None)


VARIANTS = {
    'ledge': dict(clock=6, striker='knight', lead='commander', support='ranger',
                  guard='bonearcher', sentry='skeleton', hp=14, near=(15, 8), far=(23, 12)),
    'causeway': dict(clock=6, striker='knight', lead='commander', support='archer',
                     guard='skeleton', sentry='revenant', hp=15, near=(16, 8), far=(23, 12)),
    'ridge': dict(clock=6, striker='knight', lead='commander', support='ranger',
                  guard='deatharcher', sentry='skeleton', hp=18, near=(14, 9), far=(23, 12)),
    'scar': dict(clock=6, striker='knight', lead='commander', support='soldier',
                 guard='bonearcher', sentry='revenant', hp=23, near=(15, 8), far=(23, 12)),
    'pass': dict(clock=6, striker='knight', lead='commander', support='archer',
                 guard='bonearcher', sentry='revenant', hp=18, near=(16, 8), far=(1, 17), switchback=True),
    'watch': dict(clock=6, striker='knight', lead='commander', support='ranger',
                  guard='deatharcher', sentry='skeleton-dreadguard', hp=19, near=(15, 8), far=(1, 17), switchback=True),
    'harrow': dict(clock=6, striker='knight', lead='commander', support='archer',
                   guard='necromancer', sentry='revenant', hp=15, near=(16, 8), far=(1, 17), switchback=True),
    'highroad': dict(clock=6, striker='knight', lead='commander', support='soldier',
                     guard='bonearcher', sentry='skeleton', hp=17, near=(15, 8), far=(1, 17), switchback=True),
}


def fixture(variant, mode, catalog):
    setup = VARIANTS[variant]
    ident = f'rl-marlow-cut-{variant}-{mode}'
    folder = catalog / 'game/scenario' / ident
    terrain_map = read(TEMPLATE / 'map.json')
    title = f'Marlow Cut, {variant.title()} {mode}'
    briefing = f'The enemy holds the watch post. Retake it or defeat its commander before it holds for {setup["clock"]} rounds.'
    terrain_map.update(name=title, mapId=ident, description=briefing, cols=25, rows=19 if setup.get('switchback') else 14, tiles=[])
    for row in range(terrain_map['rows']):
        for col in range(25):
            upper = 2 <= row <= 4 and 3 <= col <= 10
            connector = (row == 5 and 8 <= col <= 10) or (row == 6 and 8 <= col <= 11)
            lower = 7 <= row <= 12 and 8 <= col <= 23
            switchback = setup.get('switchback')
            if switchback:
                upper = upper and col >= 5 or row == 2 and 1 <= col <= 5
            far_bank = switchback and col in (1, 2) and 3 <= row <= 17
            if upper or connector or lower or far_bank:
                terrain = 'plains'
                if 17 <= col <= 20 and row in (9, 10):
                    terrain = 'forest'
                if col >= 19 and row in (7, 8):
                    terrain = 'hill'
                if variant in ('causeway', 'scar', 'pass', 'harrow', 'highroad') and row in (7, 8) and 11 <= col <= 13:
                    terrain = 'marsh'
            else:
                terrain = 'mountain'
                if far_bank and row >= 5:
                    terrain = 'hill'
            owner = None
            if (col, row) == POST:
                terrain, owner = 'keep', 'enemy'
            terrain_map['tiles'].append(dict(col=col, row=row, terrain=terrain, owner=owner,
                                             floor='mountain-rock' if terrain == 'mountain' else 'green-plain',
                                             environmentEffects=[]))
    roster = read(TEMPLATE / 'roster.json')
    player, enemy = roster['factions']
    player_units = [(setup['lead'], (10, 8), 'full', True),
                    (setup['striker'], (11, 7), 'full', False),
                    (setup['support'], (10, 7), 'full', False)]
    enemy_units = [('necromancer', setup['near' if mode == 'finish' else 'far'], setup['hp'], True),
                   (setup['guard'], (5, 3) if setup.get('switchback') else (4, 3), 'full', False),
                   (setup['sentry'], (6, 3), 'full', False)]
    for faction, units, first in [(player, player_units, 1), (enemy, enemy_units, 21)]:
        template = faction['units'][0]
        faction.update(gold=0, recruits=[], units=[unit(template, first + index, *entry)
                                                   for index, entry in enumerate(units)])
    roster['log'] = [briefing]
    scenario = dict(read(TEMPLATE / 'scenario.json'), characters=[])
    scenario_rule = {'id': ident, 'variants': [{'map': ident, 'forces': {'kind': 'scenario'}, 'mapRevision': 1}],
                     'territory': [hex_at(POST)], 'holdTurns': setup['clock']}
    for filename, obj in [('map', terrain_map), ('roster', roster), ('scenario', scenario)]:
        write(folder / f'{filename}.json', obj)
    write(folder / 'simulation-rule.json', scenario_rule)
    return ident, scenario_rule


def build(variants, dest):
    catalog = dest / 'catalog'
    shutil.copytree(BASE, catalog, dirs_exist_ok=True)
    manifest = read(catalog / 'manifest.json')
    arena = read(catalog / 'simulation/arena.json')
    for variant in variants:
        for mode in MODES:
            ident, scenario = fixture(variant, mode, catalog)
            path = f'game/scenario/{ident}/scenario.json'
            if path not in manifest['scenarios']:
                manifest['scenarios'].append(path)
            arena['scenarios'] = [entry for entry in arena['scenarios'] if entry['id'] != ident]
            arena['scenarios'].append(scenario)
    write(catalog / 'manifest.json', manifest)
    write(catalog / 'simulation/arena.json', arena)
    return catalog


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--variants', default=','.join(VARIANTS))
    parser.add_argument('--output', type=Path, default=OUT)
    args = parser.parse_args()
    print(build(args.variants.split(','), args.output.resolve()))
