"""Summarize the selected measured fixtures and render the two confirmed courses."""
import hashlib
import math
from collections import Counter
from pathlib import Path

from build import OUT, ROOT, read, write


def panel(version, phase, mode):
    records = read(OUT / version / phase / 'summary.json')
    return {row['player']: dict(successes=row['wins'], defeats=row['games']-row['wins']-row['caps'],
                               caps=row['caps'], outcomes=row['outcomes'])
            for row in records if row['mode'] == mode}


def phase_routes(version, mode):
    traces = read(OUT / version / 'confirmation/traces/trace-extract.json')
    counts = {}
    for battle in traces:
        if not battle['config']['campaign'].endswith(f'-{mode}'):
            continue
        totals = counts.setdefault(battle['player'], Counter())
        totals['traces'] += 1
        actions = [action for phase in battle['phases'] if phase['side'] == 'player'
                   for action in phase['actions'] if action.get('unit') == 'u1' and action['kind'] == 'move']
        if any(step['row'] == 7 for action in actions for step in action['path']):
            totals['captainOnBypass'] += 1
        else:
            totals['captainStayedOffBypass'] += 1
        if battle['success']:
            totals['clearOnPlayerTurn'+str(battle['rounds'])] += 1
        if 'u2' in battle['phases'][-1]['after']:
            totals['rangerSurvived'] += 1
    return counts


def variants(version, role):
    folder = OUT / version
    protocol = read(folder / 'confirmation/protocol.json')
    out = []
    for mode in ('switch', 'stay'):
        cid = f'rl-scree-relay-{version}-{mode}'
        campaign = folder / 'catalog/game/campaigns' / cid
        out.append(dict(
            id=cid, role=role if mode == 'switch' else 'matched counterexample',
            scope=dict(size=[18, 11], playerSight='clear', enemySight='clear', opponent='Classic',
                       playerCampaigner='Mission', playerRecruiter='Situational', recruitsPerPhase=0,
                       completedByTurn=7, deadlineDefeatOnTurn=8, simulationRoundCap=100,
                       diplomacy='unrestricted', objective='living commander at (16,4)'),
            discovery=dict(seeds=[56180101, 56180108], players=panel(version, 'discovery', mode),
                           raw=str(folder / 'discovery/results.jsonl')),
            confirmation=dict(seeds=[protocol['seed_start'], protocol['seed_start']+protocol['seeds']-1],
                              players=panel(version, 'confirmation', mode),
                              raw=str(folder / 'confirmation/results.jsonl'),
                              protocol=str(folder / 'confirmation/protocol.json')),
            replay=dict(routes=phase_routes(version, mode),
                        extract=str(folder / 'confirmation/traces/trace-extract.json'),
                        rawDirectory=str(folder / 'confirmation/traces')),
            files=dict(catalog=str(folder / 'catalog'),
                       **{name:str(campaign / f'{name}.json') for name in ('map','roster','scripts','campaign','stage','story')}),
            render=str(OUT / 'scree-relay.png') if version == 'v8' else None,
        ))
    return out


def draw(version='v8'):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from matplotlib.patches import RegularPolygon

    folder = OUT / version
    traces = read(folder / 'confirmation/traces/trace-extract.json')
    fig, axes = plt.subplots(2, 1, figsize=(11, 8))
    fig.patch.set_facecolor('#f8f5ee')
    fig.suptitle('Scree Relay', fontweight='bold', fontsize=23, x=0.105, ha='left')
    fig.text(0.105, 0.917, 'Same terrain and objective. The dreadguard starts on a different lane.', fontsize=11)
    colors = {'mountain':'#727a78', 'plains':'#e9deb9', 'signpost':'#ebdc91'}
    def point(col, row):
        return math.sqrt(3)*(col+(row % 2)*0.5), -1.5*row
    for ax, mode, title in zip(axes, ('switch','stay'), ('A   Dreadguard on the direct road','B   Dreadguard on the bypass')):
        campaign = folder / 'catalog/game/campaigns' / f'rl-scree-relay-{version}-{mode}'
        terrain = read(campaign / 'map.json')
        roster = read(campaign / 'roster.json')
        for tile in terrain['tiles']:
            ax.add_patch(RegularPolygon(point(tile['col'],tile['row']),6,radius=0.99,
                                        facecolor=colors[tile['terrain']], edgecolor='#f8f5ee', linewidth=0.55))
        for unit in (*roster['factions'][0]['units'], *roster['factions'][1]['units']):
            x,y = point(**unit['tile'])
            label, color = {'u1':('C','#203c68'), 'u2':('R','#203c68'), 'u21':('D','#9b3829')}[unit['id']]
            ax.scatter([x],[y],s=154,c=color,zorder=4,edgecolor='#f8f5ee',linewidth=1.2)
            ax.text(x,y,label,color='white',fontsize=9,weight='bold',ha='center',va='center',zorder=5)
        sample = next(t for t in traces if t['player']=='full' and t['config']['seed']==56180201
                      and t['config']['campaign'].endswith('-'+mode))
        actions = [action for phase in sample['phases'] if phase['side']=='player'
                   for action in phase['actions'] if action.get('unit')=='u1' and action['kind']=='move']
        path = [point(**step) for action in actions for step in action['path']]
        xs, ys = zip(*path)
        ax.plot(xs,ys,linewidth=4.5,color='#f8f5ee',alpha=.8,zorder=2)
        ax.plot(xs,ys,linewidth=2.3,color='#17667f',zorder=3)
        x,y=point(16,4)
        ax.scatter([x],[y],s=190,c='#31473b',marker='D',zorder=4,edgecolor='white',linewidth=.9)
        ax.text(x,y,'X',color='white',fontsize=9,fontweight='bold',ha='center',va='center',zorder=5)
        ax.set_xlim(-1.15,31.25)
        ax.set_ylim(-14, -2.3)
        ax.set_aspect('equal')
        ax.axis('off')
        ax.set_title(title, loc='left', fontsize=12, fontweight='bold')
    fig.text(0.105,0.069,'C  Captain     R  Ranger     D  Dreadguard     X  Exit',fontsize=10)
    fig.text(0.105,0.044,'Teal line shows the Full planner’s captain movement on matched seed 56180201.',fontsize=10,color='#17667f')
    plt.subplots_adjust(top=.863,hspace=.14,bottom=.1)
    fig.savefig(OUT / 'scree-relay.png',dpi=165,facecolor=fig.get_facecolor())


if __name__ == '__main__':
    draw()
    write(OUT / 'final-summary.json', dict(
        selected=variants('v8', 'route-choice challenge'),
        optionalStressPair=variants('v6', 'advance under pursuit'),
        validation=dict(content='498080d951d65ae6f9f0c3ffe2ae299ca33510e1fc4321bcf069e6cd404f2928',
                        engine='8cf0581157741df24b8e5e5c57ab6b8d0b1caae00e3def9b5d4866ee71eb1859',
                        evaluationNative=str(ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123/native/hex_command_training.so'),
                        phaseTools=[str(ROOT/'rust/target/debug/hex-command-sim'), str(ROOT/'rust/target/release/companion_trace')],
                        finalBuilder=str(OUT/'build.py'), runScript=str(OUT/'evaluate.py'),
                        noTraining=True, noIntegration=True),
    ))
