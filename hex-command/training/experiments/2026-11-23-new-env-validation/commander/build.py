from copy import deepcopy
import json
from pathlib import Path
import shutil


OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
CATALOG = OUT / 'catalog'
TEMPLATE = RUN / 'catalog/game/campaigns/rl-escort-lantern-maze'


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def point(col, row):
    return dict(col=col, row=row)


def hex_ref(col, row):
    return {'hex': point(col, row)}


def unit(template, id, kind, hp, pos, leader=False):
    return dict(deepcopy(template), id=f'u{id}', type=kind, hp=hp, tile=point(*pos),
                role='leader' if leader else None, character='nera' if leader else None)


def build(id, hp, deadline=13, fog=False, force='base'):
    cid = f'rl-commander-left-behind-{id}'
    dest = CATALOG / 'game/campaigns' / cid
    map_data, roster, scripts, campaign, stage, story = (
        read(TEMPLATE / f'{name}.json')
        for name in ('map', 'roster', 'scripts', 'campaign', 'stage', 'story')
    )
    title = 'Commander Left Behind' + (' — Wounded' if hp < 20 else '')
    briefing = 'Secure the eastern village and bring Nera to the eastern exit alive.'
    post = (13, 6)
    exit = (15, 6)
    refuge = (2, 7)
    area = {
        **{(x, y): 'water' for x in range(7, 11) for y in range(10)},
        **{(x, 6): 'ford' for x in range(7, 11)},
        **{(x, y): 'mountain' for x, y in [(0, 3), (1, 3), (1, 4), (1, 5)]},
        **{(x, y): 'forest' for x, y in [(4, 2), (5, 2), (3, 4), (4, 4), (5, 6),
                                       (6, 8), (5, 9), (6, 9), (11, 9), (12, 9),
                                       (11, 4), (12, 4), (12, 5), (14, 5)]},
        post: 'village', refuge: 'village',
    }
    floors = dict(water='dark-water', ford='shallow-water', mountain='mountain-rock',
                  forest='green-forest', village='green-plain', plains='green-plain')
    tiles = []
    for y in range(12):
        for x in range(17):
            coord = (x, y)
            terrain = area.get(coord, 'plains')
            owner = 'enemy' if coord == post else 'player' if coord == refuge else None
            tile = dict(point(*coord), terrain=terrain, floor=floors[terrain], owner=owner,
                        environmentEffects=['fog-reveal-2'] if fog else [])
            if terrain in ('ford', 'village'):
                tile['connections'] = ['e', 'w'] if terrain == 'ford' else []
            if coord in (post, refuge, exit):
                tile['id'] = {post: 'eastern-post', refuge: 'healing-village', exit: 'eastern-exit'}[coord]
            tiles.append(tile)
    map_data.update(mapId=cid, mapRevision=1, name=title, description=briefing, rows=12,
                    cols=17, tiles=tiles)
    unit_template = roster['factions'][0]['units'][1]
    allies = [('swordsman', hp, (2, 4)), ('scout', 11, (6, 7)),
              ('archer', 13, (5, 5)), ('adept', 17, (4, 5))]
    foes = [('skeleton', 16, (13, 6)), ('skeleton', 16, (14, 7)), ('revenant', 10, (0, 1))]
    if 'dread' in force:
        foes[0] = ('skeleton-dreadguard', 21, (13, 6))
    if force == 'screen':
        foes[1] = ('skeleton-impact-caster', 15, (14, 7))
    if 'healer' in force:
        allies.append(('healer', 15, (3, 7)))
    for index, profiles in enumerate((allies, foes)):
        faction = roster['factions'][index]
        faction.update(gold=0, recruits=[])
        faction['units'] = [
            unit(unit_template, i + 1 + index * 20, *profile, leader=(index == 0 and i == 0))
            for i, profile in enumerate(profiles)
        ]
    roster['turn'] = 'player'
    roster['round'] = 1
    roster['log'] = []
    roster['visionGroups'][0]['sight']['penetrates'] = [] if fog else ['fog']
    source_events = {event['id']: event for event in scripts['events']}
    leader_exit = source_events['leader-exit']
    stand = dict(leader_exit['when']['all'][0]['stand'], character='nera', tile=hex_ref(*exit))
    owns = {'owner': {'tile': hex_ref(*post), 'faction': 'player'}}
    leader_exit['when'] = {'all': [{'stand': stand}, owns]}
    leader_exit['objective'].update(label='Bring Nera to the eastern exit', intent={'escape': stand},
                                    visibleWhen={'always': True}, availableWhen=None)
    post_goal = dict(deepcopy(leader_exit), id='secure-post', when=owns, actions=[], remembers=False)
    post_goal['objective'] = dict(deepcopy(leader_exit['objective']), label='Secure the eastern village',
                                  intent={'capture': [hex_ref(*post)]}, satisfiedWhen=owns)
    source_events['deadline']['when']['turnAtLeast'] = deadline
    scripts.update(events=[source_events['commander-lost'], post_goal, leader_exit,
                           source_events['deadline']], requiredCompanions=[], defendObjectives=False)
    campaign.update(id=cid, name=title, tagline=title, description=briefing, shortDescription=briefing,
                    leader='swordsman', characters=[dict(id='nera', name='Nera')])
    stage['id'] = cid
    story.update(subtitle=title, briefing=briefing, hints=[], intro=[], victory=[])
    for stem, data in [('map', map_data), ('roster', roster), ('scripts', scripts),
                       ('campaign', campaign), ('stage', stage), ('story', story)]:
        write(dest / f'{stem}.json', data)
    return cid


def main():
    if not CATALOG.exists():
        shutil.copytree(RUN / 'catalog', CATALOG)
    manifest_path = CATALOG / 'manifest.json'
    simulation_path = CATALOG / 'simulation/campaign.json'
    manifest = read(manifest_path)
    simulation = read(simulation_path)
    variants = [
        ('healthy', 25, 13, False, 'base'), ('wounded', 7, 13, False, 'base'),
        ('healthy-fog', 25, 13, True, 'base'), ('wounded-fog', 7, 13, True, 'base'),
        ('screen', 25, 13, False, 'screen'), ('dread', 25, 13, False, 'dread'),
        ('wounded-healer', 7, 13, False, 'healer'),
        ('dread-wounded', 7, 13, False, 'dread'),
        ('dread-wounded-healer', 7, 13, False, 'dread-healer'),
        ('dread-long', 25, 22, False, 'dread'),
        ('healthy-long', 25, 22, False, 'base'),
        ('dread-wounded-long', 7, 22, False, 'dread'),
    ]
    for variant in variants:
        cid = build(*variant)
        rel = f'game/campaigns/{cid}/campaign.json'
        if rel not in manifest['campaigns']:
            manifest['campaigns'].append(rel)
        simulation['campaigns'][cid] = {'diplomacy': ['unrestricted'], 'routes': {}}
    write(manifest_path, manifest)
    write(simulation_path, simulation)


if __name__ == '__main__':
    main()
