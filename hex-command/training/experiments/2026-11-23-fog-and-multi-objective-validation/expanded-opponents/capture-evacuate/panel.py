"""Build and evaluate the Hollowbank same-map visibility/opponent matrix."""
import argparse
from collections import Counter
from copy import deepcopy
import hashlib
from itertools import product
import json
import os
from pathlib import Path
import shutil
import sys
import time


OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
PRIOR = ROOT / 'tmp/rl/multi-objective-validation/capture-evacuate'
RUN = ROOT / 'tmp/rl/ppo-from-pretrain-5m-9h-20261123'
CATALOG = OUT / 'catalog'
CAMPAIGNS = Path('game/campaigns')
VARIANTS = ('crossfire-late-guard', 'crossfire-nearside')
VISIBILITY = {'clear': ('authored', []),
              'player-fog': ('player-fog', ['player']),
              'both-fog': ('both-fog', ['player', 'enemy'])}
ENEMIES = {
    'classic': ('classic', dict(recruiter='classic', campaign='mission', planner='classic')),
    'queued': ('balanced', dict(recruiter='balanced', campaign='mission', planner='queued')),
    'full': ('predictor', dict(recruiter='predictor', campaign='mission', planner='full')),
    'predictor': ('predictor', dict(recruiter='predictor', campaign='mission', planner='queued')),
}
PLAYERS = ('hour8', 'hour10', 'full', 'queued', 'classic')
DISCOVERY = list(range(493022001, 493022009))
CONFIRMATION = list(range(493023001, 493023021))


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def sha256(path):
    with open(path, 'rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def cid(variant):
    return f'rl-hollowbank-extraction-{variant}'


def build():
    shutil.copytree(PRIOR / 'catalog-final', CATALOG)
    sim_file = CATALOG / 'simulation/campaign.json'
    manifest = read(sim_file)
    stage_audit = {}
    for variant in VARIANTS:
        campaign = cid(variant)
        stage = CATALOG / CAMPAIGNS / campaign
        original = PRIOR / 'catalog-final' / CAMPAIGNS / campaign
        roster = read(stage / 'roster.json')
        factions = {f['id']: f for f in roster['factions']}
        sight = {vg['id']: vg for vg in roster['visionGroups']}
        assert factions['enemy']['controller'] == {'kind': 'ai', 'policy': None}
        assert all(factions[f]['recruits'] == [] and factions[f]['gold'] == 0 for f in ('player', 'enemy'))
        assert all(sight[f]['sight']['penetrates'] == ['fog'] for f in ('player', 'enemy'))
        for faction in ('player', 'enemy'):
            group = deepcopy(sight[faction])
            group['id'] = f'fog-{faction}'
            group['sight']['penetrates'] = []
            roster['visionGroups'].append(group)
        write(stage / 'roster.json', roster)
        trimmed = deepcopy(roster)
        trimmed['visionGroups'] = [g for g in trimmed['visionGroups'] if g['id'] not in {'fog-player', 'fog-enemy'}]
        assert trimmed == read(original / 'roster.json')
        for name, (route, affected) in VISIBILITY.items():
            if not affected:
                continue
            choices = [{'setVisionGroup': dict(faction=faction, group=f'fog-{faction}')} for faction in affected]
            event = dict(id='returning-fog', when={'chapterStarted': True}, observer=None, remembers=True,
                         priority=100, objective=None, actions=choices + [
                             {'addConcealment': dict(area='map', kind='fog', revealWithin=3,
                                                    requires={'terrains': None, 'illumination': None})}])
            manifest['campaigns'][campaign]['routes'][route] = dict(stages={campaign: dict(
                scriptEvents=[event], scriptConditions={}, ignoreOptionalWhen=None)})
        exact = ('map.json', 'scripts.json', 'campaign.json', 'stage.json', 'story.json')
        assert all(sha256(stage / doc) == sha256(original / doc) for doc in exact)
        stage_audit[campaign] = dict(unchanged_hashes={doc: sha256(stage / doc) for doc in exact},
                                     exact_roster_except_added_vision_groups=True,
                                     enemy_controller=factions['enemy']['controller'],
                                     deadline=next(e['when']['turnAtLeast'] for e in read(stage / 'scripts.json')['events']
                                                   if e['id'] == 'deadline'))
    write(sim_file, manifest)
    write(OUT / 'authored-audit.json', dict(original_catalog=str(PRIOR / 'catalog-final'),
                                            modified_catalog=str(CATALOG), variants=stage_audit,
                                            visibility=[dict(label=mode, route=route, affected_sides=affected,
                                                             radius=3 if affected else None)
                                                        for mode, (route, affected) in VISIBILITY.items()]))


def config(variant, visibility, opponent, player, seed):
    enemy, blocks = ENEMIES[opponent]
    return dict(campaign=cid(variant), objective=VISIBILITY[visibility][0], seed=seed,
                diplomacy='unrestricted', enemy=enemy, enemyBlocks=blocks, player='balanced',
                playerBlocks=dict(recruiter='situational', campaign='mission',
                                  planner='queued' if player in ('hour8', 'hour10') else player),
                maxRounds=100, recruitLimit=0)


def discovery_schedule():
    return [dict(variant=variant, visibility=visibility, opponent=opponent, players=PLAYERS)
            for variant, visibility, opponent in product(VARIANTS, VISIBILITY, ENEMIES)]


def run(name, schedule, seeds):
    panel = OUT / name
    panel.mkdir(exist_ok=False)
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(CATALOG)
    os.environ['RAYON_NUM_THREADS'] = '1'
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    models = {f"hour{entry['hour']}": entry['model'] for entry in read(RUN / 'evaluation/hourly-summary.json')
              if entry['hour'] in [8, 10]}
    model_text = {player: Path(path).read_text() for player, path in models.items()}
    write(panel / 'protocol.json', dict(name=name, catalog=str(CATALOG), seeds=seeds, schedule=schedule,
                                        models={p: dict(path=path, sha256=sha256(path)) for p, path in models.items()},
                                        native=dict(path=native.__file__, sha256=sha256(native.__file__)), workers=1,
                                        enemy_presets=ENEMIES, visibility=VISIBILITY,
                                        maxRounds=100, recruitLimit=0, diplomacy='unrestricted',
                                        qualification_threshold={'discovery_reference_min': 6, 'discovery_Hour10_clear_gap_min': 3,
                                                                 'confirmation_reference_min': 15,
                                                                 'confirmation_Hour10_clear_gap_min': 6}))
    summaries = []
    start = time.monotonic()
    with (panel / 'results.jsonl').open('w') as raw:
        for cell in schedule:
            label = {k: cell[k] for k in ('variant', 'visibility', 'opponent')}
            tally = {}
            for player in cell['players']:
                cases = [config(**label, player=player, seed=seed) for seed in seeds]
                games = [json.loads(line) for line in native.evaluate([json.dumps(c) for c in cases], model_text.get(player))]
                assert len(games) == len(cases)
                for case, result in zip(cases, games):
                    raw.write(json.dumps(dict(**label, player=player, config=case, result=result)) + '\n')
                raw.flush()
                tally[player] = dict(wins=sum(g['success'] for g in games), games=len(games),
                                      caps=sum(g['truncated'] for g in games), errors=sum(not g['missions'] for g in games),
                                      outcomes=Counter(g['outcome'] for g in games), reasons=Counter(g['failure'] for g in games))
            summaries.append(dict(**label, results=tally))
            print(f'{len(summaries)}/{len(schedule)} {label}: ', {p: s['wins'] for p, s in tally.items()},
                  f'({round(time.monotonic()-start)}s)', flush=True)
    write(panel / 'summary.json', summaries)
    return summaries


def select():
    selected = []
    rationale = []
    preferences = ('classic', 'queued', 'full')
    for row in read(OUT / 'discovery/summary.json'):
        results = row['results']
        best = max(preferences, key=lambda p: results[p]['wins'])
        reference = results[best]['wins']
        gap = reference - results['hour10']['wins']
        entry = {k: row[k] for k in ('variant', 'visibility', 'opponent')}
        rationale.append(dict(**entry, best=best, reference=reference, hour10_gap=gap, advanced=reference >= 6 and gap >= 3))
        if rationale[-1]['advanced']:
            selected.append(dict(**entry, players=['hour8', 'hour10', best], reference=best))
    write(OUT / 'selection.json', dict(selected=selected, rationale=rationale,
                                        rule='Advance cells with ordinary reference >=6/8 and reference minus Hour 10 >=3/8. Reference ties prefer Classic, then Queued and Full. Confirm on 20 fresh matched seeds. Reference must clear >=15/20 and beat Hour 10 by >=6 games.'))
    return selected


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=('build', 'discover', 'confirm'))
    args = parser.parse_args()
    if args.action == 'build':
        build()
    elif args.action == 'discover':
        run('discovery', discovery_schedule(), DISCOVERY)
    else:
        selected = select()
        if selected:
            run('confirmation', selected, CONFIRMATION)
