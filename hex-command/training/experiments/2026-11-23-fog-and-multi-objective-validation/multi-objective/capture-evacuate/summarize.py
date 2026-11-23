"""Verify handoffs from native replays and produce the completed validation record."""
from collections import Counter
import gzip
from itertools import product
import json
from statistics import median

from build import CAMPAIGNS, OUT, RUN, read, write
from evaluate import sha256

PANEL = OUT / 'confirmation'
PRIMARY = 'crossfire-late-guard'
RESERVE = 'crossfire-nearside'
REJECTED = 'crossfire-banks'
TRACE_PATH = PANEL / f'traces-{PRIMARY}-{RESERVE}'
CAPTURE = 'Secure the crossing post'
RESCUE = 'Free Mara at the stockade'
MARA_EXIT = 'Get Mara to her exit'
TAVIN_EXIT = 'Get Tavin to his exit'
GOALS = [CAPTURE, RESCUE, MARA_EXIT, TAVIN_EXIT]


def rows(file):
    with file.open() as source:
        return [json.loads(line) for line in source]


def check_trace(trace):
    progress = trace['first']
    if RESCUE in progress:
        assert progress[RESCUE]['round'] >= progress[CAPTURE]['round']
    found = set()
    observed_orders = set()
    mara_ford = None
    reserve_appeared = False
    reserve_acted = False
    for phase in trace['phases']:
        visible = {key for key, value in phase['goals'].items() if value}
        if phase['side'] == 'player':
            orders = {item['id'].removeprefix('required:') for item in phase['orderProgress']}
            assert orders.issubset(phase['revealed'])
            observed_orders.update(orders)
        if RESCUE in visible:
            assert {CAPTURE, RESCUE}.issubset(visible)
            assert set(phase['revealed']) >= {'evacuate-tavin', 'evacuate-mara'}
            if phase['mara']:
                assert phase['mara']['faction'] == 'player'
        else:
            assert not ({'evacuate-tavin', 'evacuate-mara'} & set(phase['revealed']))
            if phase['mara']:
                assert phase['mara']['faction'] == 'prisoners'
        if CAPTURE in visible and CAPTURE not in found:
            assert phase['post_owner'] == 'player'
        if CAPTURE not in visible:
            assert phase['revealed'] == ['secure-crossing']
        found.update(visible)
        if 'u42' in phase['hostiles']:
            assert trace['variant'] == RESERVE and RESCUE in visible
            reserve_appeared = True
        for action in phase['actions']:
            if action.get('unit') == 'u42':
                reserve_acted = True
            if action['kind'] == 'move' and action.get('unit') == 'u41' and mara_ford is None:
                mara_ford = next((step['row'] for step in action.get('path', []) if step['col'] == 7), None)
    if trace['winner'] == 'playerVictory':
        assert found == set(GOALS)
        for name, (col, row) in {'mara': (2, 9), 'tavin': (16, 9)}.items():
            actor = trace['final'][name]
            assert actor['faction'] == 'player' and actor['hp'] > 0
            assert actor['tile'] == dict(col=col, row=row)
    return reserve_appeared, reserve_acted, mara_ford, observed_orders


def metrics(group):
    events = Counter()
    routes = Counter()
    orders = Counter()
    stage_losses = Counter()
    reserve_appeared = reserve_acted = 0
    for trace in group:
        appeared, acted, ford, observed_orders = check_trace(trace)
        orders.update(observed_orders)
        events.update(trace['first'].keys())
        reserve_appeared += appeared
        reserve_acted += acted
        if trace['winner'] == 'playerVictory':
            routes.update([ford])
        else:
            stage_losses.update([tuple(goal for goal in GOALS if goal in trace['first'])])
    round_medians = {goal: median(trace['first'][goal]['round'] for trace in group if goal in trace['first'])
                     for goal in GOALS if events[goal]}
    return dict(matches=len(group), first_reached=dict(events), median_first_round=round_medians,
                stages=[dict(goal=CAPTURE, entered=len(group), completed=events[CAPTURE]),
                        dict(goal=RESCUE, entered=events[CAPTURE], completed=events[RESCUE]),
                        dict(goal='Both exits alive', entered=events[RESCUE], completed=sum(routes.values()))],
                losses_by_goals_reached=[dict(goals_reached=labels, games=count) for labels, count in stage_losses.items()],
                goals_recorded_in_player_orders=dict(orders),
                reserve_observed_after_rescue=reserve_appeared, reserve_acting_matches=reserve_acted,
                mara_westbound_crossing_row_in_wins=dict(routes))


def validate():
    protocol = read(PANEL / 'protocol.json')
    d1 = read(OUT / 'discovery/protocol.json')
    d2 = read(OUT / 'discovery-2/protocol.json')
    assert set(protocol['seeds']).isdisjoint(d1['seeds'] + d2['seeds'])
    assert d1['models'] == d2['models'] == protocol['models']
    all_rows = rows(PANEL / 'results.jsonl')
    indexed = {(r['variant'], r['player'], r['config']['seed']): r['result'] for r in all_rows}
    assert len(indexed) == len(all_rows)
    assert set(indexed) == set(product(protocol['variants'], protocol['players'], protocol['seeds']))
    assert all(not result['truncated'] and len(result['missions']) == 1 for result in indexed.values())
    with gzip.open(TRACE_PATH / 'digest.jsonl.gz', 'rt') as native:
        traces = [json.loads(line) for line in native]
    parity = {(t['variant'], t['player'], t['seed']) for t in traces}
    assert len(traces) == len(parity) == 200
    assert parity == set(product([PRIMARY, RESERVE], protocol['players'], protocol['seeds']))
    for trace in traces:
        expected, = indexed[trace['variant'], trace['player'], trace['seed']]['missions']
        assert (trace['round'], trace['winner']) == (expected['round'], expected['winner'])
    trace_metrics = {
        variant: {player: metrics([t for t in traces if t['variant'] == variant and t['player'] == player])
                  for player in protocol['players']}
        for variant in (PRIMARY, RESERVE)
    }
    comparisons = {}
    for variant in protocol['variants']:
        comparisons[variant] = {}
        for player in ('hour8', 'hour10'):
            comparisons[variant][player] = {}
            for reference in ('classic', 'queued', 'full'):
                outcomes = [(indexed[variant, player, seed]['success'], indexed[variant, reference, seed]['success'])
                            for seed in protocol['seeds']]
                comparisons[variant][player][reference] = dict(
                    reference_only=outcomes.count((False, True)), learner_only=outcomes.count((True, False)),
                    both_win=outcomes.count((True, True)), both_lose=outcomes.count((False, False)))
    final_catalog = OUT / 'catalog-final'
    for variant in protocol['variants']:
        suffix = CAMPAIGNS / f'rl-hollowbank-extraction-{variant}'
        for file in (final_catalog / suffix).glob('*.json'):
            assert sha256(file) == sha256(OUT / 'catalog-discovery-2' / suffix / file.name)
    counts = read(PANEL / 'summary.json')
    for row in counts:
        observed = [indexed[row['variant'], row['player'], seed] for seed in protocol['seeds']]
        assert row['wins'] == sum(r['success'] for r in observed)
    return dict(
        name='Hollowbank Extraction',
        decision={
            PRIMARY: 'Validated challenge for both checkpoints against the strongest ordinary references.',
            RESERVE: 'Validated current-checkpoint challenge with an ordinary Skeleton activated after rescue.',
            REJECTED: 'Rejected after its discovery gap shrank on the fresh panel.',
            'crossfire-fog': 'Rejected in discovery after reference success fell to Classic 2/8.',
        },
        required_sequence=['Own the crossing post at (7,4)', 'Approach neutral Mara at (13,2) after capture and transfer her to the player',
                           'Get Mara to (2,9) and Tavin to (16,9) alive at the same time'],
        scope='Single authored stage, 18×12, fixed rosters and clear vision on both sides for confirmed conditions, unrestricted diplomacy, Classic opposition. Independent random seeds on the same layout.',
        discovery=dict(seeds_per_condition=8, panels=[d1['seeds'], d2['seeds']], games=560),
        confirmation=dict(protocol='confirmation/protocol.json', seeds=protocol['seeds'], games=len(all_rows),
                          caps=0, errors=0, results=counts, paired_comparisons=comparisons),
        trace=dict(path=str(TRACE_PATH.relative_to(OUT)), games=len(traces), matched_terminal_round_and_winner=True,
                   per_policy=trace_metrics, witnesses=[
                       dict(variant=PRIMARY, seed=493021006, reference='classic',
                            observation='Hour10 frees Mara on round 5 and she reaches her exit on 11. With all hostiles gone, Tavin stays on post (7,4) from round 6 until the round 14 deadline. Classic frees Mara on 6 and both actors reach their exits on 11.'),
                       dict(variant=PRIMARY, seed=493021001, reference='classic',
                            observation='Both capture on round 2. Hour10 sees the rescue target but never frees Mara, even after the last hostile disappears on round 7. Classic frees her on round 8 and exits with both actors on 13.'),
                       dict(variant=RESERVE, seed=493021011, reference='classic',
                            observation='Hour10 frees Mara on round 5. The newly spawned Skeleton pursues her, leaves her at 6 HP at her exit on 11 and kills her on 12 while Tavin waits at the post. Classic kills the reserve, secures Mara on round 11 and evacuates Tavin on 13.'),
                   ]),
        artifacts=dict(catalog='validated-catalog.tar.gz', catalog_sha256=sha256(OUT / 'validated-catalog.tar.gz'),
                       builder='build.py', evaluation='evaluate.py', trace='trace.py', report='report.md', fixtures='fixtures',
                       renderer='render-stage.py', primary_preview=f'renders/{PRIMARY}/preview.png',
                       reserve_preview=f'renders/{RESERVE}/preview.png'),
        frozen_training_run=str(RUN), frozen_source_commit=(RUN / 'source-commit.txt').read_text().strip(),
        models=protocol['models'], native=protocol['native'],
    )


if __name__ == '__main__':
    write(OUT / 'final-summary.json', validate())
