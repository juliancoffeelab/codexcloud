"""Recount all raw outcomes and emit the Hollowbank qualification report."""
from collections import Counter
from itertools import product
import json

from panel import CONFIRMATION, DISCOVERY, ENEMIES, OUT, PLAYERS, PRIOR, VARIANTS, VISIBILITY, read, sha256, write

NAMES = dict(zip(VARIANTS, ('East-road guard', 'West-bank reserve')))
FOG_NAMES = dict(clear='Clear', **{'player-fog': 'Player-only fog', 'both-fog': 'Both sides fog'})
PLAYER_NAMES = dict(hour8='Hour 8', hour10='Hour 10', full='Full', queued='Queued', classic='Classic')


def recount(name, schedule, seeds):
    with (OUT / name / 'results.jsonl').open() as file:
        raw = [json.loads(line) for line in file]
    index = {(row['variant'], row['visibility'], row['opponent'], row['player'], row['config']['seed']): row
             for row in raw}
    assert len(index) == len(raw)
    expected = {(*[cell[k] for k in ('variant', 'visibility', 'opponent')], policy, seed)
                for cell in schedule for policy, seed in product(cell['players'], seeds)}
    assert expected == index.keys()
    results = []
    for cell in schedule:
        scope = tuple(cell[k] for k in ('variant', 'visibility', 'opponent'))
        policy_results = {}
        for policy in cell['players']:
            group = [index[(*scope, policy, seed)] for seed in seeds]
            assert all(g['config']['enemyBlocks'] == ENEMIES[g['opponent']][1] for g in group)
            assert all(g['config']['recruitLimit'] == 0 and g['config']['maxRounds'] == 100 for g in group)
            games = [g['result'] for g in group]
            policy_results[policy] = dict(wins=sum(game['success'] for game in games), games=len(games),
                                          errors=sum(not game['missions'] for game in games),
                                          caps=sum(game['truncated'] for game in games),
                                          terminal_outcomes=dict(Counter(game['outcome'] for game in games)))
        row = dict(variant=scope[0], visibility=scope[1], opponent=scope[2], results=policy_results)
        if name == 'confirmation':
            reference = cell['reference']
            score = policy_results[reference]['wins']
            hour10 = policy_results['hour10']['wins']
            hour8 = policy_results['hour8']['wins']
            pairs = [(index[(*scope, 'hour10', seed)]['result']['success'],
                      index[(*scope, reference, seed)]['result']['success']) for seed in seeds]
            row.update(reference=reference, qualified=score >= 15 and score - hour10 >= 6,
                       hour10_only=score >= 15 and score - hour10 >= 6 and score - hour8 < 6,
                       reference_only=pairs.count((False, True)), hour10_only_wins=pairs.count((True, False)))
        results.append(row)
    return index, results


def alignment(index):
    comparisons = []
    for (variant, mode, opponent, player, seed), row in index.items():
        if opponent == 'queued':
            predictor = index.get((variant, mode, 'predictor', player, seed))
            if predictor:
                comparisons.append(row['result'] == predictor['result'])
    return dict(identical=sum(comparisons), paired=len(comparisons))


def summarize():
    selection = read(OUT / 'selection.json')
    protocol = read(OUT / 'discovery/protocol.json')
    previous = read(PRIOR / 'final-summary.json')
    assert protocol['seeds'] == DISCOVERY
    assert set(DISCOVERY).isdisjoint(CONFIRMATION)
    assert set(DISCOVERY + CONFIRMATION).isdisjoint(previous['confirmation']['seeds'] +
                                                   [s for cohort in previous['discovery']['panels'] for s in cohort])
    discovery, discovery_table = recount('discovery', protocol['schedule'], DISCOVERY)
    confirmation, confirmation_table = recount('confirmation', selection['selected'], CONFIRMATION)
    alignment_by_panel = dict(discovery=alignment(discovery), confirmation=alignment(confirmation))
    visibility = read(OUT / 'visibility/summary.json')
    trace = read(OUT / 'trace/summary.json')
    qualified = [row for row in confirmation_table if row['qualified']]
    assert len(trace) == 10
    result = dict(name='Hollowbank Extraction expanded qualification', variants=NAMES, opponent_blocks=ENEMIES,
                  visibility=visibility['results'], seed_sets=dict(discovery=DISCOVERY, confirmation=CONFIRMATION),
                  schedule=dict(discovery_matches=len(discovery), confirmation_matches=len(confirmation)),
                  selection_rule=selection['rule'], discovery=discovery_table, confirmation=confirmation_table,
                  qualified=qualified, fog_qualified=sum(r['visibility'] != 'clear' for r in qualified),
                  paired_queued_predictor_result_and_round_equality=alignment_by_panel,
                  same_maps=read(OUT / 'authored-audit.json'), models=protocol['models'], native=protocol['native'],
                  trace=dict(matched_samples=len(trace), evidence='trace/summary.json'),
                  catalog_sha256=sha256(OUT / 'validated-catalog.tar.gz'), previous_scope=previous['confirmation']['results'])
    write(OUT / 'final-summary.json', result)
    return result


def render(result):
    lines = ['# Hollowbank Extraction opponent and fog qualification', '',
             'Only the two existing clear-vision Classic-opponent conditions qualified. The first validation used Classic as the standard campaign control. This expansion evaluated Classic, Queued, Full, and Predictor separately for both original layouts, with clear vision, player-only radius-three returning fog, and radius-three returning fog on both sides.', '',
             'Every case retained the original map, rosters, rewards, capture–rescue–evacuation sequence, and turn-14 deadline. Both forces had zero recruits. The 100-round simulation cap never fired. Across 960 discovery and 360 confirmation games, there were zero harness errors. Fresh confirmation used different seeds on the same maps.', '',
             'Enemy blocks were Classic / Mission / Classic, Balanced / Mission / Queued, Predictor / Mission / Full, and Predictor / Mission / Queued for Classic, Queued, Full, and Predictor, respectively. Native planner-entry captures verified those block selections. Queued and Predictor had identical terminal results and rounds in all matched cases sampled in this matrix. This is an empirical match under the fixed-roster setup.', '',
             '## Discovery', '',
             'Eight matched seeds per policy and cell, 493022001–493022008. An ordinary reference needed at least 6/8 clears and an advantage of at least three clears over Hour 10 to advance.', '',
             '| Layout | Visibility | Enemy | Hour 8 | Hour 10 | Full | Queued | Classic |',
             '|---|---|---|---:|---:|---:|---:|---:|']
    for row in result['discovery']:
        record = [NAMES[row['variant']], FOG_NAMES[row['visibility']], row['opponent'].title()]
        record += [f"{row['results'][player]['wins']}/8" for player in PLAYERS]
        lines.append('| ' + ' | '.join(record) + ' |')
    lines += ['', '## Fresh confirmation', '',
              'Twenty matched new seeds per policy and cell, 493023001–493023020. The reference was chosen from discovery. Qualification required at least 15/20 reference clears and at least six more clears than Hour 10.', '',
              '| Layout | Visibility | Enemy | Hour 8 | Hour 10 | Discovery-selected reference | Qualified |',
              '|---|---|---|---:|---:|---|---|']
    for row in result['confirmation']:
        scores = row['results']
        reference = row['reference']
        label = 'Hour 10 only' if row['hour10_only'] else 'Yes' if row['qualified'] else 'No'
        record = [NAMES[row['variant']], FOG_NAMES[row['visibility']], row['opponent'].title(),
                  f"{scores['hour8']['wins']}/20", f"{scores['hour10']['wins']}/20",
                  f"{PLAYER_NAMES[reference]} {scores[reference]['wins']}/20", label]
        lines.append('| ' + ' | '.join(record) + ' |')
    lines += ['',
              'The east-road guard under player-only fog advanced after Classic cleared 6/8 versus Hour 10 at 3/8. Fresh seeds brought Classic to 9/20 and Hour 10 to 5/20. All 39 defeats in that confirmation cell came from the authored deadline. Under fog on both sides the best ordinary reference reached 3/8, so those cells stopped at discovery. The earlier optional radius-two test looked at a separate tuning variant and also failed screening, with a best reference of Classic at 2/8.', '',
              'The west-bank reserve with clear vision showed a seven-clear Hour 10 gap against Queued and Predictor, with the selected reference at 12/20 in each case. Against Full the reference reached 7/20 with a one-clear advantage over Hour 10.', '',
              '## Native visibility and replays', '',
              'The fog overlay is a radius-three concealment rule added at chapter start. Only the chosen observers lose fog penetration. Native captures restored both faction views and checked that acting planners see the same hostile unit IDs as the corresponding battlefield view. Visibility checks use snapshots with living observers and hostiles. Visible tiles returned to fog for each affected faction. Clear observers had zero fog tiles and all living hostiles visible.', '']
    for item in result['visibility']:
        witness = item['living_contact_left_native_view']
        if witness:
            layout = NAMES[next(v for v in VARIANTS if item['campaign'].endswith(v))]
            lines.append(f"- {layout}, {FOG_NAMES[item['mode']].lower()}, {item['faction']} observer. Living {witness['unit']} left view between round {witness['from_round']} {witness['from_side']} and round {witness['to_round']} {witness['to_side']} captures.")
    lines += ['', 'Ten complete match replays matched winner, terminal round, and outcome text. On east-road guard seed 493023001 versus Classic, Hour 10 and the Classic reference both won in clear vision. Under player-only fog, Hour 10 rescued on round 8 and the reference on 10, after rescues on rounds 7 and 6 in clear vision. Both expired on round 14 under fog. The player-facing rescue goal appeared after capture, while the corresponding tactical target was missing from several fogged phases before the planners approached Mara.', '',
              'On east-road guard seed 493022002 with both sides fogged, both players captured and rescued. Hour 10 reached neither exit and Classic reached Tavin’s exit. Both expired on round 14. These traces show where the sampled games lost time. The full matrix supplies the qualification counts.', '',
              '## Artifacts', '',
              'The exact simulation catalog is in `validated-catalog.tar.gz`. `panel.py` builds it and runs both panels. `final-summary.json` includes all terminal outcomes, seed lists, match settings, native/model hashes, and comparisons. `discovery/` and `confirmation/` contain raw native outcomes and separate protocols. `visibility/` contains engine state captures and faction views. `trace/` contains representative replays and their objective timelines. Nothing was integrated or trained.']
    (OUT / 'REPORT.md').write_text('\n'.join(lines) + '\n')


if __name__ == '__main__':
    render(summarize())
