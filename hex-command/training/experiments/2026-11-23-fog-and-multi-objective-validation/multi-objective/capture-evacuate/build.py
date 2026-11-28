"""Build a separate frozen-data catalog with authored Hollowbank Extraction fixtures."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import shutil


OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
SOURCE = RUN / 'catalog'
CAMPAIGNS = Path('game/campaigns')
TEMPLATE = SOURCE / CAMPAIGNS / 'rl-escort-lantern-maze'
CAPTIVE_TEMPLATE = SOURCE / CAMPAIGNS / 'lantern-road/stages/lantern-bell/roster.json'

POST = (7, 4)
CELL = (13, 2)
WEST = (2, 9)
EAST = (16, 9)
TITLE = 'Hollowbank Extraction'

VARIANTS = {
    'switchback': dict(deadline=15),
    'crossfire': dict(deadline=15, reverse=True),
    'pressure': dict(deadline=16, pressure=True),
    'cross-pressure': dict(deadline=16, pressure=True, reverse=True),
    'switchback-healer': dict(deadline=15, healer=True),
    'crossfire-healer': dict(deadline=15, reverse=True, healer=True),
    'crossfire-tight': dict(deadline=12, reverse=True, healer=True),
    'crossfire-nearside': dict(deadline=14, reverse=True, healer=True, reserve='near'),
    'crossfire-farside': dict(deadline=14, reverse=True, healer=True, reserve='far'),
    'crossfire-banks': dict(deadline=15, reverse=True, healer=True, reserve='both'),
    'crossfire-separation': dict(deadline=14, reverse=True, healer=True, distant=True),
    'crossfire-separation-tight': dict(deadline=12, reverse=True, healer=True, distant=True),
    'crossfire-late-guard': dict(deadline=14, reverse=True, healer=True, far_guard=True),
    'crossfire-fog': dict(deadline=14, reverse=True, healer=True, fog=True),
}


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def point(col, row):
    return dict(col=col, row=row)


def tile_ref(coords):
    return {'hex': point(*coords)}


def unit_match(character=None, faction=None, role=None, side=None):
    return dict(character=character, faction=faction, role=role, side=side,
                unitType=None, unitTypes=[], aiGroup=None, retreating=None)


def unit(template, number, kind, tile, hp='full', character=None, role=None):
    return dict(deepcopy(template), id=f'u{number}', type=kind, tile=point(*tile),
                hp=hp, character=character, role=role)


def objective(label, intent, gate=None, preserve=None, marker=None, satisfied=None):
    return dict(faction='player', id=None, label=label, intent=intent, optional=False,
                visibleWhen=gate or {'always': True}, availableWhen=gate,
                satisfiedWhen=satisfied, preserveLives=preserve,
                playerMarker=dict(tiles=[tile_ref(marker)], units=None, visibleInFog=True))


def event(event_id, when, actions, goal=None, priority=0):
    return dict(id=event_id, observer=None, when=when, actions=actions, objective=goal,
                priority=priority, remembers=True)


def arrived(character, coords, role):
    return {'stand': dict(tile=tile_ref(coords), through=False,
                          **unit_match(character=character, faction='player', role=role))}


def make_script(commander_exit, captive_exit, deadline, reserve):
    post_owned = {'owner': dict(tile=tile_ref(POST), faction='player')}
    post_secured = {'flag': 'crossing-secured'}
    captive_freed = {'flag': 'mara-freed'}
    mara_held = unit_match(character='mara', faction='prisoners', role='captive')
    mara_player = dict(mara_held, faction='player')
    reach_cell = dict(tile=None, target=mara_held, radius=1, through=False,
                      **unit_match(side='player'))
    mara_arrived = arrived('mara', captive_exit, 'captive')
    commander_arrived = arrived('tavin', commander_exit, 'leader')
    reserve_units = {'near': [('skeleton', (3, 7))], 'far': [('skeleton', (17, 8))],
                     'both': [('skeleton', (3, 7)), ('skeleton', (17, 8))]}
    reinforcements = [
        {'spawn': dict(faction='enemy', tile=tile_ref(tile), area=[], radius=1,
                       aiGroup=None, units=[kind])}
        for kind, tile in reserve_units.get(reserve, [])
    ]
    return dict(requiredCompanions=[], enemyGroups=[], enemyLeaderPost=None,
                defendObjectives=False, events=[
        event('commander-lost', {'noUnits': unit_match(character='tavin', faction='player')},
              [{'defeat': 'Commander Tavin fell.'}], priority=100),
        event('mara-lost', {'noUnits': unit_match(character='mara')},
              [{'defeat': 'Mara did not survive.'}], priority=100),
        event('secure-crossing', post_owned,
              [{'setFlag': 'crossing-secured'}, {'log': 'The crossing post is secure. Reach Mara at the northeast stockade.'}],
              objective('Secure the crossing post', {'capture': [tile_ref(POST)]}, marker=POST)),
        event('free-mara', {'all': [post_secured, {'proximity': reach_cell}]},
              [{'changeAllegiance': dict(units=mara_held, faction='player')},
               {'setFlag': 'mara-freed'}, {'log': 'Mara is free. Get her and Tavin to their separate exits.'},
               *reinforcements],
              objective('Free Mara at the stockade', {'approach': reach_cell},
                        gate=post_secured, marker=CELL)),
        event('evacuate-mara', {'all': [captive_freed, mara_arrived]},
              [{'log': 'Mara reached her exit.'}],
              objective('Get Mara to her exit', {'escort': mara_arrived['stand']},
                        gate=captive_freed, preserve=mara_player, marker=captive_exit,
                        satisfied={'all': [captive_freed, mara_arrived]})),
        event('evacuate-tavin', {'all': [captive_freed, commander_arrived]},
              [{'log': 'Tavin reached his exit.'}],
              objective('Get Tavin to his exit', {'escape': commander_arrived['stand']},
                        gate=captive_freed, marker=commander_exit,
                        satisfied={'all': [captive_freed, commander_arrived]})),
        event('evacuation-complete', {'all': [post_secured, captive_freed, mara_arrived, commander_arrived]},
              [{'victory': 'Mara and Tavin both reached safety.'}], priority=-1),
        event('deadline', {'turnAtLeast': deadline},
              [{'defeat': 'The evacuation window closed.'}], priority=-100),
    ])


def make_map(map_data, cid, briefing, commander_exit, captive_exit, fog):
    area = {(x, y): 'water' for x in (7, 8) for y in range(11)}
    area.update({(x, y): 'ford' for x in (7, 8) for y in (4, 8)})
    area.update({p: 'mountain' for p in [(12, 0), (12, 1), (13, 1), (14, 0),
                                        (15, 3), (14, 4), (15, 4), (15, 5)]})
    area.update({p: 'forest' for p in [(3, 1), (4, 1), (3, 2), (4, 2), (5, 3),
                                      (5, 6), (6, 6), (4, 7), (2, 8), (3, 9),
                                      (9, 0), (9, 1), (11, 2), (11, 3), (12, 3),
                                      (10, 6), (11, 6), (10, 7), (12, 7), (13, 8),
                                      (13, 9), (14, 9), (16, 7)]})
    area[POST] = 'village'
    floors = dict(water='dark-water', ford='shallow-water', mountain='mountain-rock',
                  forest='green-forest', village='green-plain', plains='green-plain')
    landmarks = {POST: 'crossing-post', CELL: 'stockade',
                 commander_exit: 'tavin-exit', captive_exit: 'mara-exit'}
    tiles = []
    for y in range(12):
        for x in range(18):
            coords = (x, y)
            terrain = area.get(coords, 'plains')
            entry = dict(point(*coords), terrain=terrain, floor=floors[terrain],
                         owner='enemy' if coords == POST else None,
                         environmentEffects=['fog-reveal-2'] if fog else [])
            if terrain in ('ford', 'village'):
                entry['connections'] = ['e', 'w'] if terrain == 'ford' else []
            if coords in landmarks:
                entry['id'] = landmarks[coords]
            tiles.append(entry)
    return dict(map_data, mapId=cid, mapRevision=1, name=TITLE, description=briefing,
                rows=12, cols=18, tiles=tiles)


def make_roster(roster, pressure, healer, fog, far_guard):
    player, enemy = roster['factions']
    template = player['units'][1]
    prisoner_data = read(CAPTIVE_TEMPLATE)
    prisoners = next(f for f in prisoner_data['factions'] if f['id'] == 'prisoners')
    prisoner_vision = next(g for g in prisoner_data['visionGroups'] if g['id'] == 'prisoners')
    roster['visionGroups'].append(prisoner_vision)
    roster['visionGroups'][0]['sight']['penetrates'] = [] if fog else ['fog']
    player.update(gold=0, recruits=[], units=[
        unit(template, 1, 'swordsman', (2, 4), character='tavin', role='leader'),
        unit(template, 2, 'swordsman', (4, 4)),
        unit(template, 3, 'scout', (5, 5)),
        unit(template, 4, 'adept', (4, 5)),
        unit(template, 5, 'healer' if healer else 'spearman', (3, 5)),
    ])
    enemy.update(gold=0, recruits=[], units=[
        unit(template, 21, 'skeleton-dreadguard' if pressure else 'skeleton', POST),
        unit(template, 22, 'skeleton', (11, 4)),
        unit(template, 23, 'bonearcher', (16, 5) if far_guard else (12, 3)),
        unit(template, 24, 'revenant' if pressure else 'skeleton', (5, 0)),
    ])
    player['relations']['prisoners'] = 'neutral'
    enemy['relations']['prisoners'] = 'allied'
    prisoners.update(relations=dict(player='neutral', enemy='allied', prisoners='allied'),
                     units=[unit(template, 41, 'captives', CELL, character='mara', role='captive')])
    roster['factions'].append(prisoners)
    return dict(roster, turn='player', round=1, log=[])


def build(catalog, variant, deadline, reverse=False, pressure=False, healer=False, fog=False,
          reserve=None, distant=False, far_guard=False):
    cid = f'rl-hollowbank-extraction-{variant}'
    dest = catalog / CAMPAIGNS / cid
    map_data, roster, campaign, stage, story = (read(TEMPLATE / f'{name}.json')
                                               for name in ('map', 'roster', 'campaign', 'stage', 'story'))
    west, east = ((0, 10), (17, 10)) if distant else (WEST, EAST)
    commander_exit, captive_exit = (east, west) if reverse else (west, east)
    exits = 'Mara leaves through the southwest exit and Tavin through the southeast exit.' if reverse else (
        'Mara leaves through the southeast exit and Tavin through the southwest exit.')
    briefing = f'Secure the crossing post, then reach Mara at the northeast stockade. {exits} Get both there before turn {deadline}.'
    map_data = make_map(map_data, cid, briefing, commander_exit, captive_exit, fog)
    roster = make_roster(roster, pressure, healer, fog, far_guard)
    scripts = make_script(commander_exit, captive_exit, deadline, reserve)
    campaign.update(id=cid, name=TITLE, tagline=TITLE, description=briefing, shortDescription=briefing,
                    leader='swordsman', characters=[dict(id='tavin', name='Tavin'), dict(id='mara', name='Mara')])
    stage['id'] = cid
    story.update(subtitle=TITLE, briefing=briefing, hints=[], intro=[], victory=[])
    for name, value in dict(map=map_data, roster=roster, scripts=scripts, campaign=campaign,
                            stage=stage, story=story).items():
        write(dest / f'{name}.json', value)
    return f'{CAMPAIGNS}/{cid}/campaign.json', cid


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('catalog')
    parser.add_argument('variants', nargs='+')
    args = parser.parse_args()
    catalog = OUT / args.catalog
    if catalog.exists():
        raise SystemExit(f'Refusing to overwrite {catalog}')
    shutil.copytree(SOURCE, catalog)
    settings = read(OUT / 'variants.json') if (OUT / 'variants.json').exists() else VARIANTS
    manifest = read(catalog / 'manifest.json')
    simulation = read(catalog / 'simulation/campaign.json')
    for variant in args.variants:
        rel, cid = build(catalog, variant, **settings[variant])
        manifest['campaigns'].append(rel)
        simulation['campaigns'][cid] = dict(diplomacy=['unrestricted'], routes={})
    write(catalog / 'manifest.json', manifest)
    write(catalog / 'simulation/campaign.json', simulation)


if __name__ == '__main__':
    main()
