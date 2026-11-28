"""Build each Scree Relay iteration in its own copy of the frozen evaluation catalog."""
import argparse
import copy
import json
import shutil
from pathlib import Path

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
BASE = RUN / 'catalog'
TEMPLATE = BASE / 'game/campaigns/rl-route-ambush'


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def unit(template, ident, unit_type, col, row, hp='full', character=None, role=None):
    return dict(template, id=ident, type=unit_type, tile=dict(col=col, row=row), hp=hp,
                character=character, role=role)


def named_goal(template, actor, col, row, title):
    event = copy.deepcopy(template)
    selector = event['when']['stand']
    selector.update(tile={'hex': {'col': col, 'row': row}}, character=actor, through=False)
    event['objective'].update(label=title, intent={'goTo': copy.deepcopy(selector)},
                              playerMarker={'tiles': [selector['tile']], 'units': None,
                                            'visibleInFog': True})
    return event


def fixture(version, mode, catalog):
    cid = f'rl-scree-relay-{version}-{mode}'
    folder = catalog / 'game/campaigns' / cid
    title = 'Scree Relay — ' + ('Switch routes' if mode == 'switch' else 'Stay on the road')
    round_limit = 8 if version in ('v1', 'v2', 'v3', 'v5') else 7
    use_relay = version not in ('v6', 'v7', 'v8')
    goal_text = 'Bring the ranger to the relay, then the captain to the exit' if use_relay else 'Bring the captain to the exit'
    briefing = f'{goal_text} by the end of turn {round_limit}.'
    terrain_map = read(TEMPLATE / 'map.json')
    terrain_map.update(name=title, mapId=cid, description=briefing, cols=18, rows=11, tiles=[])
    relay_tile, exit_tile = (14, 4), (16, 4)
    for row in range(11):
        for col in range(18):
            north = row == 4 and 2 <= col <= 16
            south = row == 7 and 6 <= col <= 14
            connector = row in (5, 6) and col == 6
            north_banks = (row, col) in ((3, 5), (3, 6), (3, 7), (5, 5), (3, 8))
            east = row in (3, 5) and 13 <= col <= 16 or row == 6 and 13 <= col <= 14
            terrain = 'plains' if north or south or connector or north_banks or east else 'mountain'
            if (col, row) in (relay_tile, exit_tile):
                terrain = 'signpost'
            if version in ('v2', 'v3', 'v4') and (col, row) in ((8, 3), (9, 3), (10, 3), (11, 3), (7, 5)):
                terrain = 'plains'
            if version in ('v4', 'v5', 'v6', 'v7', 'v8') and col in (14, 15) and row in (5, 6, 7):
                terrain = 'plains'
            floor = 'mountain-rock' if terrain == 'mountain' else 'green-plain'
            terrain_map['tiles'].append(dict(col=col, row=row, floor=floor, terrain=terrain, owner=None,
                                             environmentEffects=[]))
    roster = read(TEMPLATE / 'roster.json')
    player, enemy = roster['factions']
    player.update(units=[unit(player['units'][0], 'u1', 'commander', 4, 4, character='garrison', role='leader'),
                         unit(player['units'][1], 'u2', 'ranger', 8, 4, character='runner'),
                         unit(player['units'][2], 'u3', 'swordsman', 6, 4)])
    if version in ('v3', 'v4'):
        player['units'].append(unit(player['units'][2], 'u4', 'adept', 5, 4))
    guard = 'skeleton-dreadguard' if version in ('v1', 'v2') else 'skeleton'
    caster = 'skeleton-impact-caster-l2' if version == 'v1' else 'skeleton-impact-caster'
    enemies = [(guard, 9, 4), (caster, 10, 4), (caster, 11, 4), ('bonearcher', 15, 3)]
    if mode == 'stay':
        enemies = [(guard, 10, 7), (caster, 10, 4), (caster, 11, 7), ('bonearcher', 15, 3)]
    if version in ('v5', 'v6', 'v7', 'v8'):
        enemies = [('skeleton-dreadguard', 10, 4), ('skeleton-impact-caster', 11, 4)]
        if mode == 'stay':
            enemies = [('skeleton-dreadguard', 10, 7), ('skeleton-impact-caster', 11, 7)]
    if version in ('v7', 'v8'):
        enemies = enemies[:1]
    if version == 'v8':
        player['units'] = player['units'][:2]
    enemy.update(units=[unit(enemy['units'][0], f'u{21 + i}', *entry) for i, entry in enumerate(enemies)])
    roster['log'] = [briefing]

    scripts = read(TEMPLATE / 'scripts.json')
    win, deadline, lost, _ = scripts['events']
    relay = named_goal(win, 'runner', *relay_tile, 'Bring the ranger to the relay')
    relay.update(id='light-relay', actions=[{'setFlag': 'relay-lit'}, {'log': 'The exit is open.'}])
    relay['objective']['satisfiedWhen'] = {'flag': 'relay-lit'}
    exit_event = named_goal(win, 'garrison', *exit_tile, 'Bring the captain to the exit')
    exit_event.update(id='captain-exit', when={'all': [exit_event['when'], {'flag': 'relay-lit'}]})
    exit_event['objective']['availableWhen'] = {'flag': 'relay-lit'}
    deadline['when']['turn'] = round_limit + 1
    if use_relay:
        scripts['events'] = [relay, exit_event, deadline, lost]
    else:
        exit_event['when'] = exit_event['when']['all'][0]
        exit_event['objective']['availableWhen'] = {'always': True}
        scripts['events'] = [exit_event, deadline, lost]

    campaign = read(TEMPLATE / 'campaign.json')
    campaign.update(id=cid, name=title, tagline=title, description=briefing)
    campaign['characters'].append({'id': 'runner', 'name': 'Relay ranger'})
    stage = dict(read(TEMPLATE / 'stage.json'), id=cid)
    story = dict(read(TEMPLATE / 'story.json'), subtitle=title, briefing=briefing)
    for name, value in [('map', terrain_map), ('roster', roster), ('scripts', scripts),
                        ('campaign', campaign), ('stage', stage), ('story', story)]:
        write(folder / f'{name}.json', value)
    return cid


def build(version):
    catalog = OUT / version / 'catalog'
    shutil.copytree(BASE, catalog, dirs_exist_ok=True)
    manifest = read(catalog / 'manifest.json')
    simulation = read(catalog / 'simulation/campaign.json')
    cids = [fixture(version, mode, catalog) for mode in ('switch', 'stay')]
    for cid in cids:
        path = f'game/campaigns/{cid}/campaign.json'
        if path not in manifest['campaigns']:
            manifest['campaigns'].append(path)
        simulation['campaigns'][cid] = {'diplomacy': ['unrestricted'], 'routes': {}}
    write(catalog / 'manifest.json', manifest)
    write(catalog / 'simulation/campaign.json', simulation)
    return cids


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('version', choices=tuple(f'v{i}' for i in range(1, 9)))
    print(build(parser.parse_args().version))
