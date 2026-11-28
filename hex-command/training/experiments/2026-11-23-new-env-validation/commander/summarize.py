import hashlib
import json
from pathlib import Path
import shutil

from build import CATALOG, OUT, ROOT, RUN, read, write


def render_files(variant, campaign_dir):
    dest = OUT / 'render' / variant
    dest.mkdir(parents=True, exist_ok=True)
    for filename in ('map.json', 'roster.json'):
        shutil.copyfile(campaign_dir / filename, dest / filename)
    title = read(dest / 'map.json')['name']
    write(dest / 'scenario.json', dict(map='map.json', roster='roster.json', fixedArmies=True,
                                       characters=[dict(id='nera', name='Nera')], tags=['simulation', 'human']))
    write(dest / 'overlay.json', dict(observer='player', title=title,
                                      notes=['Secure the east post and bring Nera to the exit alive before turn 22.',
                                             'Classic enemies, clear visibility, no recruitment.'],
                                      annotations=[
                                          dict(col=x, row=y, text=text, color='#ffe08a', dy=-21, ring=True)
                                          for x, y, text in [(2, 4, 'Nera'), (2, 7, 'Refuge'), (8, 6, 'Ford'),
                                                             (13, 6, 'East post'), (15, 6, 'Exit')]]))
    return dict(id=f'commander-{variant}', **{key: str((dest / filename).relative_to(ROOT))
                                             for key, filename in [('map', 'map.json'), ('roster', 'roster.json'),
                                                                   ('scenario', 'scenario.json'), ('overlay', 'overlay.json'),
                                                                   ('output', 'preview.png')]})


def main():
    confirmation = read(OUT / 'confirmation-summary.json')
    variants = []
    selected = {'dread-long': 'full', 'dread-wounded-long': 'classic'}
    for variant, teacher in selected.items():
        campaign_dir = CATALOG / 'game/campaigns' / f'rl-commander-left-behind-{variant}'
        files = {key: campaign_dir / f'{key}.json' for key in ('map', 'roster', 'scripts', 'campaign', 'stage', 'story')}
        variants.append(dict(variant=variant, enemy='classic', visibility='clear both sides',
                             deadline='lose at start of turn 22', recruits=0,
                             teacher=teacher, teacherType='ordinary native rule-based planner',
                             comparisons=[c for c in confirmation['comparisons'] if c['variant'] == variant],
                             seeds=confirmation['seeds'],
                             files={key: dict(path=str(path), sha256=hashlib.sha256(path.read_bytes()).hexdigest())
                                    for key, path in files.items()},
                             render=render_files(variant, campaign_dir)))
    healthy, wounded = [read(Path(v['files']['roster']['path'])) for v in variants]
    assert healthy['factions'][0]['units'][0]['hp'] == 25
    assert wounded['factions'][0]['units'][0]['hp'] == 7
    healthy['factions'][0]['units'][0]['hp'] = 7
    assert healthy == wounded
    binary = RUN / 'native/hex_command_training.so'
    sources = dict(
        learningModels=confirmation['models'], nativeBinding=dict(path=str(binary), sha256=hashlib.sha256(binary.read_bytes()).hexdigest()),
        sourceCheckpointCatalog=str(RUN / 'catalog'), exactFixtureCatalog=str(CATALOG),
        builder=str(OUT / 'build.py'), runner=str(OUT / 'run.py'),
        confirmationRaw=str(OUT / 'confirmation.jsonl'),
        discovery=[str(OUT / (name + '-summary.json')) for name in ('discovery', 'discovery-followup', 'discovery-counterpart')],
    )
    trace_evidence = OUT / 'action-evidence.json'
    if trace_evidence.exists():
        sources.update(actionEvidence=str(trace_evidence), traceSource=str(OUT / 'probe/src/main.rs'),
                       rawTraces=[str(path) for path in sorted(OUT.glob('trace-*.jsonl.gz'))])
    healer = 'dread-wounded-healer'
    healer_path = CATALOG / 'game/campaigns' / f'rl-commander-left-behind-{healer}'
    alternate = dict(variant=healer, enemy='classic', visibility='clear both sides',
                     deadline='lose at start of turn 13', recruits=0,
                     changesComparedWithHealthy='7 HP Nera, one additional 15 HP allied healer, turn 13 deadline',
                     teacher='classic', teacherType='ordinary native rule-based planner',
                     comparisons=[c for c in confirmation['comparisons'] if c['variant'] == healer],
                     seeds=confirmation['seeds'], campaignDirectory=str(healer_path))
    write(OUT / 'final-summary.json', dict(accepted=variants, alternate=alternate, sources=sources))
    write(OUT / 'render-manifest.json', [v['render'] for v in variants])


if __name__ == '__main__':
    main()
