from copy import deepcopy
import hashlib
import json
from pathlib import Path
import shutil


OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
FROZEN = RUN / 'catalog'
CATALOG = OUT / 'catalog'
TEMPLATE = FROZEN / 'game/campaigns/rl-defend-bracken-stand-010'
BASE = 'rl-ashbank-relay'
HOLD = (4, 3)
TARGET = (10, 6)
TURNS = 10
DEADLINE = 13

ALLIES = [
    ('warden', 3, 4), ('soldier', 4, 3), ('hammerman', 5, 4),
    ('rider', 4, 5), ('healer', 3, 5), ('adept', 4, 6),
]
FOES = [
    ('skeleton-axeman', 8, 2), ('skeleton', 8, 3), ('revenant', 9, 5),
    ('skeleton-impact-caster', 9, 6), ('deatharcher', 10, 6),
]
BASE_WAVES = [
    (4, (7, 1), ['skeletal-hound', 'skeleton']),
    (8, (7, 0), ['skeletal-hound', 'skeleton-axeman']),
]
VARIANTS = {
    'base': {},
    'archer': {'allies': [*ALLIES[:-1], ('archer', 4, 6)]},
    'reserve': {'allies': [*ALLIES, ('swordsman', 2, 5)]},
    'reinforced': {'allies': [*ALLIES, ('soldier', 2, 5)]},
    'lighter': {'waves': [(4, (7, 1), ['skeleton']), (8, (7, 0), ['skeletal-hound', 'skeleton'])]},
    'veteran': {'allies': [('sentinel', 3, 4), *ALLIES[1:2], ('warhammer', 5, 4), *ALLIES[3:]]},
    'thinner-post': {'foes': FOES[:-1], 'waves': BASE_WAVES},
    'crossroad': {'target': (9, 6), 'foes': [*FOES[:3], ('skeleton-impact-caster', 10, 5),
                                           ('deatharcher', 9, 6)]},
    'defender': {'allies': [('warden', 3, 4), ('sentinel', 4, 3), *ALLIES[2:]]},
    'early-raid': {'waves': [(3, (7, 1), ['skeletal-hound', 'skeleton']),
                             (7, (6, 0), ['skeletal-hound', 'skeleton-axeman'])]},
    'vigil': {'staffed': True},
    'vigil-reserve': {'allies': [*ALLIES, ('soldier', 2, 5)], 'staffed': True},
    'push-first': {'allies': [*ALLIES, ('swordsman', 2, 5)],
                   'waves': [(3, (12, 6), ['skeleton', 'skeletal-hound']),
                             (7, (12, 7), ['skeleton', 'skeletal-hound'])]},
}


LONG_TARGET = (14, 7)
LONG_FOES = [
    ('skeleton-axeman', 8, 2), ('skeleton', 7, 3), ('revenant', 12, 6),
    ('skeleton-impact-caster', 13, 7), ('deatharcher', 14, 7),
]
LONG_TERRAIN = {
    **{(10, row): 'water' for row in range(11)},
    (10, 3): 'ford', (10, 7): 'ford',
    **{(col, 7): 'road' for col in (6, 7, 8, 9, 11, 12, 13, 15, 16)},
    (11, 4): 'forest', (12, 4): 'forest', (13, 4): 'forest',
    (12, 8): 'hill', (14, 6): 'forest', (13, 9): 'forest',
    (9, 5): 'plains', (9, 6): 'plains', (10, 7): 'ford', (11, 7): 'road',
    (7, 8): 'plains', (8, 8): 'plains', (9, 8): 'plains',
}
LONG = dict(target=LONG_TARGET, foes=LONG_FOES, terrain=LONG_TERRAIN, size=(17, 11),
            capture_by=9)
for tag, changes in {
    'long': {},
    'long-reserve': {'allies': [*ALLIES, ('swordsman', 5, 5)]},
    'long-open': {'terrain': {p: t for p, t in LONG_TERRAIN.items() if t not in ('water', 'ford')}},
    'long-relay': {'allies': [('warden', 3, 4), ('soldier', 4, 3), ('hammerman', 5, 4),
                              ('rider', 7, 7), ('healer', 5, 5), ('adept', 7, 6)]},
    'long-veteran': {'allies': [('warden', 3, 4), ('sentinel', 4, 3), ('warhammer', 5, 4),
                                ('rider', 5, 6), ('healer', 4, 5), ('adept', 5, 7)]},
    'long-early': {'capture_by': 7, 'allies': [*ALLIES, ('scout', 5, 7)]},
    'long-pressure': {'allies': [*ALLIES, ('soldier', 5, 5)],
                      'waves': [(4, (5, 0), ['skeleton-axeman', 'skeleton']),
                                (8, (6, 0), ['skeletal-hound', 'skeleton-axeman'])]},
    'long-rush': {'capture_by': 6, 'allies': [*ALLIES, ('rider', 6, 7)],
                  'foes': [*LONG_FOES[:2], ('skeleton', 12, 6), *LONG_FOES[3:]]},
    'long-breach': {'capture_by': 7, 'allies': [*ALLIES, ('warhammer', 8, 7)],
                    'foes': [*LONG_FOES[:2], ('skeleton', 12, 6), *LONG_FOES[3:]]},
    'long-vigil': {'staffed': True, 'allies': [*ALLIES, ('soldier', 2, 5)]},
    'long-defended': {'defend': True, 'allies': [*ALLIES, ('soldier', 2, 5)]},
    'long-push-first': {'allies': [*ALLIES, ('swordsman', 5, 5)],
                        'waves': [(3, (13, 5), ['skeleton', 'skeletal-hound']),
                                  (7, (15, 6), ['skeleton', 'skeletal-hound'])]},
}.items():
    VARIANTS[tag] = LONG | changes


BREACH = VARIANTS['long-breach']
VETERAN = VARIANTS['long-veteran']
WEST_WAVES = [
    (4, (6, 1), ['skeleton-axeman', 'skeletal-hound']),
    (8, (5, 0), ['skeleton-axeman', 'skeletal-hound']),
]
TWO_FRONT_WAVES = [
    (4, (6, 1), ['skeleton-axeman', 'skeletal-hound']),
    (6, (15, 8), ['skeleton', 'skeletal-hound']),
    (8, (5, 0), ['skeleton-axeman', 'skeletal-hound']),
]
for suffix, changes in {
    'swift': {'capture_by': 5},
    'rapid': {'capture_by': 6},
    'west': {'capture_by': 7, 'waves': WEST_WAVES},
    'west-swift': {'capture_by': 5, 'waves': WEST_WAVES},
    'two-fronts': {'capture_by': 7, 'waves': TWO_FRONT_WAVES},
    'fort-relief': {'capture_by': 8, 'waves': TWO_FRONT_WAVES},
    'vigil': {'capture_by': 7, 'staffed': True},
    'riders': {'capture_by': 6, 'allies': [*ALLIES[:-1], ('rider', 6, 6), ('warhammer', 8, 7)]},
    'strong-west': {'waves': [(4, (7, 1), ['skeleton-axeman-l2', 'skeletal-hound']),
                              (8, (6, 0), ['skeleton-axeman-l2', 'skeletal-hound'])]},
}.items():
    VARIANTS[f'breach-{suffix}'] = BREACH | changes
for suffix, changes in {
    'swift': {'capture_by': 7},
    'west': {'capture_by': 8, 'waves': WEST_WAVES},
    'vigil': {'staffed': True, 'capture_by': 8},
    'two-fronts': {'waves': TWO_FRONT_WAVES},
}.items():
    VARIANTS[f'veteran-{suffix}'] = VETERAN | changes


FORK_PRESSURE = {
    'waves': [(4, (6, 1), ['skeleton-axeman-l2', 'skeleton-impact-caster']),
              (5, (15, 8), ['skeleton', 'skeletal-hound']),
              (8, (5, 0), ['skeleton-axeman-l2', 'skeletal-hound'])],
    'capture_by': 6,
}
for suffix, changes in {
    'base': {},
    'swift': {'capture_by': 5},
    'wounded': {'allies': [ALLIES[0], ('soldier', 4, 3, 19), *ALLIES[2:], ('warhammer', 8, 7)]},
    'reinforce': {'allies': [*ALLIES, ('warhammer', 8, 7), ('archer', 2, 2)]},
    'veteran': {'allies': [ALLIES[0], ('soldier', 4, 3, 21), *ALLIES[2:],
                         ('warhammer', 8, 7), ('soldier', 4, 2)]},
    'closer': {'waves': [(4, (6, 2), ['skeleton-axeman-l2', 'skeleton-impact-caster']),
                         (5, (15, 8), ['skeleton', 'skeletal-hound']),
                         (8, (4, 1), ['skeleton-axeman-l2', 'skeletal-hound'])]},
    'flanked': {'waves': [(4, (3, 1), ['skeleton-axeman-l2', 'skeleton-impact-caster']),
                          (5, (15, 8), ['skeleton', 'skeletal-hound']),
                          (8, (5, 0), ['skeleton-axeman-l2', 'skeletal-hound'])]},
    'trident': {'waves': [(4, (6, 1), ['skeleton-axeman-l2', 'skeleton-impact-caster-l2', 'skeletal-hound']),
                          (5, (15, 8), ['skeleton', 'skeletal-hound']),
                          (8, (5, 0), ['skeleton-axeman-l2', 'skeletal-hound'])]},
    'eastern': {'capture_by': 7,
                'waves': [(4, (6, 2), ['skeleton-axeman-l2', 'skeleton-impact-caster']),
                          (4, (15, 8), ['skeleton-axeman', 'skeletal-hound']),
                          (8, (4, 1), ['skeleton-axeman-l2', 'skeletal-hound'])]},
    'vigil': {'staffed': True, 'capture_by': 7,
              'allies': [ALLIES[0], ('soldier', 4, 3, 20), *ALLIES[2:], ('warhammer', 8, 7)]},
}.items():
    VARIANTS[f'fork-{suffix}'] = BREACH | FORK_PRESSURE | changes


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def hex_point(col, row):
    return {'col': col, 'row': row}


def hex_ref(place):
    return {'hex': hex_point(*place)}


def owns(place, turns=None, faction='player'):
    return {'tile': hex_ref(place), 'faction': faction, 'turns': turns, 'when': None}


def event(event_id, condition, actions=(), objective=None, remembers=True, priority=0):
    return dict(id=event_id, when=condition, actions=list(actions), objective=objective,
                observer=None, remembers=remembers, priority=priority)


def objective(label, intent, satisfied):
    return dict(faction='player', id=None, label=label, intent=intent, optional=False,
                visibleWhen={'always': True}, availableWhen={'always': True},
                satisfiedWhen=satisfied, preserveLives=None, playerMarker=None)


def tile(terrain, place, sites):
    floors = {'forest': 'green-forest', 'water': 'shallow-water', 'ford': 'shallow-water',
              'hill': 'green-plain', 'mountain': 'mountain-rock'}
    value = dict(**hex_point(*place), terrain=terrain, owner=None,
                 floor=floors.get(terrain, 'green-plain'), environmentEffects=[])
    if terrain in ('ford', 'village', 'road'):
        value['connections'] = ['e', 'w'] if terrain in ('ford', 'road') else []
    if place in sites:
        value['id'], value['owner'] = sites[place]
    return value


def unit(template, index, unit_type, col, row, hp='full', leader=False):
    return dict(deepcopy(template), id=f'u{index}', type=unit_type, hp=hp,
                role='leader' if leader else None, character=None, aiGroup=None,
                tile=hex_point(col, row))


def compile_case(tag, settings, catalog):
    campaign_id = f'{BASE}-{tag}'
    target = settings.get('target', TARGET)
    allies, foes = settings.get('allies', ALLIES), settings.get('foes', FOES)
    waves = settings.get('waves', BASE_WAVES)
    staffed = settings.get('staffed', False)
    capture_by = settings.get('capture_by')
    condition = {'own': owns(HOLD, TURNS)}
    intent = condition
    if staffed:
        requirement = dict(tile=hex_ref(HOLD), turns=TURNS, faction='player', unitTypes=[],
                           when={'own': owns(HOLD)})
        condition = {'held': requirement}
        intent = {'hold': requirement}
    hold_owned = {'own': owns(HOLD)}
    target_owned = {'own': owns(target)}
    board, roster, scripts, campaign, stage, story = (
        read(TEMPLATE / f'{stem}.json')
        for stem in ('map', 'roster', 'scripts', 'campaign', 'stage', 'story')
    )
    title = 'Ashbank Relay' if tag != 'push-first' else 'Ashbank Relay | Eastern Reserve'
    description = 'Hold Ashbank village for 10 consecutive turns and secure the enemy fort at the eastern crossing.'
    if capture_by is not None:
        description += f' The fort must be ours by the end of round {capture_by}.'
    cols, rows = settings.get('size', (13, 9))
    board.update(mapId=campaign_id, name=title, description=description, rows=rows, cols=cols)
    terrain = {
        (1, 1): 'forest', (2, 1): 'forest', (2, 2): 'forest', (3, 2): 'hill',
        (4, 1): 'forest', (5, 1): 'hill', (6, 1): 'forest', (6, 2): 'forest',
        (7, 2): 'hill', (9, 1): 'forest', (9, 2): 'forest', (10, 2): 'forest',
        (5, 3): 'road', (6, 3): 'road', (7, 3): 'road', (8, 3): 'road',
        (7, 4): 'hill', (8, 4): 'forest', (9, 3): 'forest', (10, 4): 'hill',
        (3, 7): 'forest', (4, 7): 'forest', (3, 8): 'forest', (2, 7): 'hill',
        (5, 6): 'road', (6, 6): 'road', (7, 6): 'road', (8, 6): 'road',
        (9, 6): 'road', (11, 6): 'road', (12, 6): 'road', (10, 7): 'hill',
        (6, 7): 'forest', (7, 8): 'forest', (8, 8): 'forest',
        HOLD: 'village', target: 'fort',
    }
    terrain.update(settings.get('terrain', {}))
    sites = {HOLD: ('ashbank-village', 'player'), target: ('east-crossing-fort', 'enemy')}
    board['tiles'] = [
        tile(terrain.get((col, row), 'plains'), (col, row), sites)
        for row in range(board['rows']) for col in range(board['cols'])
    ]
    for index, force in enumerate((allies, foes)):
        faction = roster['factions'][index]
        template = faction['units'][0]
        faction['units'] = [unit(template, i + 1 + index * 20, *entry, leader=index == 0 and i == 0)
                            for i, entry in enumerate(force)]
    roster['visionGroups'] = roster['visionGroups'][:2]
    events = [
        event('hold-village', condition, [{'setFlag': 'ten-turn-hold'}],
              objective('Hold Ashbank village for 10 consecutive turns', intent, None), remembers=False),
        event('capture-fort', target_owned, [],
              objective('Capture and control the enemy fort', {'capture': [hex_ref(target)]}, target_owned),
              remembers=False),
    ]
    for i, (round_no, place, units) in enumerate(waves, 1):
        spawn = dict(faction='enemy', tile=hex_ref(place), area=[], radius=1, aiGroup=None, units=units)
        events.append(event(f'enemy-reserve-{i}', {'turn': round_no}, [{'spawn': spawn}]))
    if capture_by is not None:
        events.append(event('crossing-deadline', {'all': [{'turn': capture_by + 1}, {'not': target_owned}]},
                            [{'defeat': 'The eastern crossing was still enemy-held at the deadline.'}], priority=-90))
    fall = deepcopy(next(x for x in scripts['events'] if x['id'] == 'fall'))
    events.extend([
        fall,
        event('win', {'all': [condition, hold_owned, target_owned]},
              [{'victory': 'Ashbank held and the eastern crossing secured.'}]),
        event('deadline', {'turn': settings.get('deadline', DEADLINE)},
              [{'defeat': 'The relief force arrived before both positions were secured.'}], priority=-100),
    ])
    scripts.update(requiredCompanions=[], enemyGroups=[], enemyLeaderPost=None,
                   defendObjectives=settings.get('defend', False), events=events)
    campaign.update(id=campaign_id, name=title, tagline=title, description=description,
                    shortDescription=description, race='human', leader=allies[0][0], characters=[])
    stage['id'] = campaign_id
    story.update(subtitle=title, briefing=description, hints=[], intro=[], victory=[])
    dest = catalog / 'game/campaigns' / campaign_id
    for stem, value in zip(('map', 'roster', 'scripts', 'campaign', 'stage', 'story'),
                           (board, roster, scripts, campaign, stage, story), strict=True):
        write(dest / f'{stem}.json', value)
    return dict(id=campaign_id, stage=str(dest), sites={'hold': HOLD, 'capture': target},
                holdTurns=TURNS, deadline=settings.get('deadline', DEADLINE),
                captureBy=capture_by, staffed=staffed,
                allies=allies, foes=foes, waves=waves)


def catalog_hash(catalog):
    digest = hashlib.sha256()
    for path in sorted(p for p in catalog.rglob('*') if p.is_file()):
        digest.update(path.relative_to(catalog).as_posix().encode() + b'\0')
        digest.update(hashlib.sha256(path.read_bytes()).digest())
    return digest.hexdigest()


def build(tags, catalog):
    shutil.copytree(FROZEN, catalog, dirs_exist_ok=True)
    manifest_path = catalog / 'manifest.json'
    simulation_path = catalog / 'simulation/campaign.json'
    manifest, simulation = read(manifest_path), read(simulation_path)
    cases = {}
    for tag in tags:
        case = compile_case(tag, VARIANTS[tag], catalog)
        cid = case['id']
        cases[tag] = case
        manifest['campaigns'].append(f'game/campaigns/{cid}/campaign.json')
        simulation['campaigns'][cid] = {'diplomacy': ['unrestricted'], 'routes': {}}
    write(manifest_path, manifest)
    write(simulation_path, simulation)
    return cases


if __name__ == '__main__':
    write(OUT / 'variants.json', build(VARIANTS, CATALOG))
