from copy import deepcopy
import json
from pathlib import Path
import shutil

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
FROZEN = RUN / 'catalog'
TEMPLATE = FROZEN / 'game/campaigns/rl-lair-outworks'
CATALOG = HERE / 'catalog'
BASE = 'rl-kiln-crossing'


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


PLAYER = [
    ('commander', 3, 4, 'full'), ('soldier', 4, 3, 'full'),
    ('hammerman', 3, 3, 'full'), ('swordsman', 3, 2, 'full'),
    ('archer', 3, 5, 'full'), ('scout', 4, 1, 'full'),
]
ENEMY = [
    ('skeleton-dreadguard', 6, 3, 'full'), ('skeleton-axeman-l2', 6, 2, 'full'),
    ('revenant', 7, 4, 'full'), ('skeleton', 7, 5, 'full'),
    ('deatharcher', 7, 1, 'full'), ('skeleton-impact-caster-l2', 8, 3, 'full'),
]


def change(force, changes):
    result = []
    for kind, x, y, hp in force:
        result.append(changes.get(kind, (kind, x, y, hp)))
    return result


# Every variant keeps four dedicated melee fighters per side plus stronger enemy ranged damage.
VARIANTS = {
    'original': dict(),
    'longer': dict(deadline=12),
    'wounded': dict(enemy={
        'deatharcher': ('deatharcher', 7, 1, 17),
        'skeleton-impact-caster-l2': ('skeleton-impact-caster-l2', 8, 3, 14),
        'skeleton-dreadguard': ('skeleton-dreadguard', 6, 3, 20),
    }),
    'pierce': dict(enemy={
        'skeleton-impact-caster-l2': ('bonearcher', 8, 3, 'full'),
    }),
    'north': dict(player={
        'swordsman': ('swordsman', 3, 1, 'full'),
        'archer': ('archer', 4, 2, 'full'),
    }, deadline=10),
    'cohesion': dict(player={
        'swordsman': ('soldier', 4, 2, 'full'),
        'archer': ('healer', 3, 5, 'full'),
    }, deadline=10),
}


# Focused follow-up. A screened battery in a smaller engagement, with a rearmost leader.
SMALL_PLAYER = [
    ('warden', 2, 4, 'full'), ('soldier', 4, 3, 'full'), ('hammerman', 4, 2, 'full'),
    ('healer', 3, 4, 'full'), ('scout', 4, 1, 'full'),
]
SMALL_ENEMY = [
    ('revenant', 6, 3, 'full'), ('skeleton-axeman-l2', 7, 3, 'full'),
    ('skeleton', 7, 4, 'full'), ('deatharcher', 7, 1, 17),
    ('skeleton-impact-caster', 8, 3, 'full'),
]
for name, rules in {
    'screen': {},
    'screen-ridge': {'terrain': {(6, 3): 'ruins', (5, 4): 'ruins', (5, 2): 'forest',
                                 (6, 1): 'plains', (7, 2): 'forest'}},
    'screen-troop': {'player': {'scout': ('rider', 4, 1, 'full')}},
    'screen-infantry': {'player': {'warden': ('sentinel', 2, 4, 'full'),
                                   'scout': ('rider', 4, 1, 'full')}},
    'screen-open': {'terrain': {(5, 2): 'ruins', (5, 5): 'forest', (6, 1): 'plains'}},
    'screen-soft': {'enemy': {'skeleton-axeman-l2': ('skeleton-axeman', 7, 3, 'full'),
                              'deatharcher': ('deatharcher', 7, 1, 12)}},
    'screen-deeper': {'enemy': {'revenant': ('revenant', 7, 2, 'full'),
                                'skeleton': ('skeleton', 7, 5, 'full')}},
    'screen-rank': {'player': {'warden': ('sentinel', 2, 4, 'full'),
                               'hammerman': ('warhammer', 4, 2, 'full')}},
}.items():
    VARIANTS[name] = dict(player_force=SMALL_PLAYER, enemy_force=SMALL_ENEMY,
                          deadline=11, **rules)


for name, changes in {
    'screen-close-fog': {'fog_range': 2},
    'screen-wide-fog': {'fog_range': 4},
    'screen-shelter': {'terrain': {(4, 2): 'forest', (3, 4): 'forest', (5, 4): 'ruins'}},
    'screen-approach': {'player': {'warden': ('warden', 1, 4, 'full'),
                                   'soldier': ('soldier', 3, 3, 'full'),
                                   'scout': ('scout', 3, 1, 'full')}},
    'screen-overlap': {'enemy': {'skeleton-impact-caster': ('skeleton-impact-caster', 7, 2, 'full')}},
    'open-close-fog': {'fog_range': 2, 'terrain': VARIANTS['screen-open']['terrain']},
    'open-wide-fog': {'fog_range': 4, 'terrain': VARIANTS['screen-open']['terrain']},
}.items():
    VARIANTS[name] = dict(player_force=SMALL_PLAYER, enemy_force=SMALL_ENEMY,
                          deadline=11, **changes)


def compile_variant(tag, variant, manifest, simulation):
    case_id = f'{BASE}-{tag}'
    destination = CATALOG / 'game/campaigns' / case_id
    destination.mkdir(parents=True, exist_ok=True)
    board = read(TEMPLATE / 'map.json')
    board.update(mapId=case_id, name='Kiln Crossing', cols=11, rows=8)
    special_terrain = {(5, row): 'mountain' for row in (0, 2, 5, 6, 7)}
    special_terrain.update({(5, 1): 'hill', (4, 4): 'ruins'})
    special_terrain.update({(4, 1): 'forest', (6, 1): 'forest', (7, 0): 'forest'})
    special_terrain.update({(col, row): 'road' for col in (4, 5, 6) for row in (3, 4) if (col, row) != (4, 4)})
    special_terrain.update(variant.get('terrain', {}))
    board['tiles'] = [
        dict(col=col, row=row, terrain=special_terrain.get((col, row), 'plains'),
             owner=None, floor='green-plain', environmentEffects=[])
        for row in range(board['rows']) for col in range(board['cols'])
    ]
    roster = read(TEMPLATE / 'roster.json')
    player = change(variant.get('player_force', PLAYER), variant.get('player', {}))
    enemy = change(variant.get('enemy_force', ENEMY), variant.get('enemy', {}))
    for faction, force in zip(roster['factions'], (player, enemy), strict=True):
        unit = faction['units'][0]
        faction['units'] = [
            dict(deepcopy(unit), type=kind, tile=dict(col=x, row=y), hp=hp,
                 id=f'u{index + (1 if faction["id"] == "player" else 11)}', character=None,
                 role='leader' if faction['id'] == 'player' and index == 0 else None)
            for index, (kind, x, y, hp) in enumerate(force)
        ]
    scripts = read(TEMPLATE / 'scripts.json')
    targets = [entry[0] for entry in enemy[-2:]]
    scripts['events'][0]['when']['noUnits']['unitTypes'] = targets
    scripts['events'][0]['objective']['intent']['defeat']['unitTypes'] = targets
    scripts['events'][0]['objective']['label'] = 'Defeat both ranged crews'
    scripts['events'][0]['actions'][0]['victory'] = 'Both ranged crews were defeated.'
    scripts['events'][-1]['when']['turn'] = variant.get('deadline', 9)
    scripts['events'][-1]['actions'][0]['defeat'] = 'The battery held through the deadline.'
    campaign = read(TEMPLATE / 'campaign.json')
    campaign.update(id=case_id, name='Kiln Crossing', characters=[], leader=player[0][0],
                    description='A screened ranged battery watches the open road and the northern ridge crossing.',
                    shortDescription='Defeat both ranged crews and keep the commander alive.')
    stage = read(TEMPLATE / 'stage.json')
    stage['id'] = case_id
    story = read(TEMPLATE / 'story.json')
    story.update(subtitle='Kiln Crossing', briefing=campaign['shortDescription'])
    for name, value in (('map', board), ('roster', roster), ('scripts', scripts),
                        ('campaign', campaign), ('stage', stage), ('story', story)):
        write(destination / f'{name}.json', value)
    manifest['campaigns'].append(f'game/campaigns/{case_id}/campaign.json')
    routes = deepcopy(simulation['campaigns']['rl-lair-outworks'])
    for route in routes['routes'].values():
        stages = route['stages']
        stages[case_id] = stages.pop('rl-lair-outworks')
        for action in stages[case_id]['scriptEvents'][0]['actions']:
            if 'addConcealment' in action:
                action['addConcealment']['revealWithin'] = variant.get('fog_range', 3)
    simulation['campaigns'][case_id] = routes
    return dict(player=player, enemy=enemy, deadline=variant.get('deadline', 9),
                cols=board['cols'], rows=board['rows'], fog_range=variant.get('fog_range', 3))


def main():
    shutil.copytree(FROZEN, CATALOG, dirs_exist_ok=True)
    manifest = read(CATALOG / 'manifest.json')
    simulation = read(CATALOG / 'simulation/campaign.json')
    descriptions = {
        tag: compile_variant(tag, value, manifest, simulation)
        for tag, value in VARIANTS.items()
    }
    write(CATALOG / 'manifest.json', manifest)
    write(CATALOG / 'simulation/campaign.json', simulation)
    write(HERE / 'variants.json', descriptions)


if __name__ == '__main__':
    main()
