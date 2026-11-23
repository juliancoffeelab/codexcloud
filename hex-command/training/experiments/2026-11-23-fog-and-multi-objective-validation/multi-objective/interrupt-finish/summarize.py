"""Assemble exact condition, outcome, action, and provenance evidence."""
from collections import Counter
import copy
from pathlib import Path

from build import MODES, OUT, POST, ROOT, RUN, VARIANTS, hex_at, read, write
from evaluate import digest


def counts(values):
    return dict(Counter(str(value) if value is not None else 'none' for value in values))


def evidence_metrics(rows):
    return dict(games=len(rows), wins=sum(row['winner'] == 'playerVictory' for row in rows),
                losses=sum(row['winner'] == 'playerDefeat' for row in rows),
                caps=sum(row['winner'] is None for row in rows),
                playerLeaderWins=sum(row['winner'] == 'playerVictory' and row['reason'] == 'leader' for row in rows),
                playerTerritoryWins=sum(row['winner'] == 'playerVictory' and row['reason'] == 'territory' for row in rows),
                enemyTerritoryWins=sum(row['winner'] == 'playerDefeat' and row['reason'] == 'territory' for row in rows),
                firstKnightTurnOne=counts((row['firstKnightMove']['col'], row['firstKnightMove']['row'])
                                          if row['firstKnightMove'] else None for row in rows),
                firstCommanderAttackRound=counts(row['firstCommanderAttack'] for row in rows),
                firstPlayerCaptureRound=counts(row['firstCapture'] for row in rows),
                enemyHeldBeforeFirstCapture=counts(row['enemyHeldBeforeFirstCapture'] for row in rows),
                terminalRound=counts(row['rounds'] for row in rows))


def main():
    discovery = read(OUT / 'discovery-2/protocol.json')
    confirmation = read(OUT / 'confirmation/protocol.json')
    observation = read(OUT / 'confirmation/observation-evidence.json')
    trace_path = OUT / 'confirmation/traces/objective-traces.json'
    traces = read(trace_path)
    catalog = Path(confirmation['catalog'])
    fixtures = {}
    normalized = {}
    for mode in MODES:
        ident = f'rl-marlow-cut-pass-{mode}'
        folder = catalog / 'game/scenario' / ident
        terrain = read(folder / 'map.json')
        roster = read(folder / 'roster.json')
        rule = read(folder / 'simulation-rule.json')
        enemy_leader = next(unit for unit in roster['factions'][1]['units'] if unit['role'] == 'leader')
        assert enemy_leader['tile'] == hex_at(VARIANTS['pass']['far' if mode == 'interrupt' else 'near'])
        for unit in [unit for faction in roster['factions'] for unit in faction['units']]:
            assert unit['aiGroup'] is None and not unit['moved'] and not unit['attackSpent']
        assert rule['territory'] == [hex_at(POST)] and rule['holdTurns'] == 6
        assert next(tile['owner'] for tile in terrain['tiles'] if (tile['col'], tile['row']) == POST) == 'enemy'
        assert discovery['sha256'][str((folder / 'map.json').relative_to(catalog))] == digest(folder / 'map.json')
        assert discovery['sha256'][str((folder / 'roster.json').relative_to(catalog))] == digest(folder / 'roster.json')
        normalized_roster = copy.deepcopy(roster)
        for unit in normalized_roster['factions'][1]['units']:
            if unit['role'] == 'leader':
                unit['tile'] = None
        normalized[mode] = (terrain['tiles'], normalized_roster)
        fixtures[mode] = dict(landscape=ident, commanderTile=enemy_leader['tile'], commanderHp=enemy_leader['hp'],
                              map=str((folder / 'map.json').relative_to(OUT)),
                              roster=str((folder / 'roster.json').relative_to(OUT)),
                              scenario=str((folder / 'scenario.json').relative_to(OUT)),
                              objectiveRule=str((folder / 'simulation-rule.json').relative_to(OUT)),
                              preview=f'renders/{mode}/preview.png', renderInputs=f'renders/{mode}')
    assert normalized['interrupt'] == normalized['finish']
    assert set(discovery['seeds']).isdisjoint(confirmation['seeds'])
    assert len(traces) == 200
    metrics = {mode: {player: evidence_metrics([row for row in traces if row['player'] == player and
                                                row['config']['landscape'] == fixtures[mode]['landscape']])
                      for player in confirmation['cases']} for mode in MODES}
    for mode in MODES:
        assert all(row['games'] == 20 and row['caps'] == 0 for row in metrics[mode].values())
    assert metrics['interrupt']['queued']['playerTerritoryWins'] == 20
    assert metrics['finish']['queued']['playerLeaderWins'] == 20
    assert all(metrics['finish'][player]['playerLeaderWins'] == 0 for player in ('hour8', 'hour10'))
    witnesses = [x for x in traces if x['config']['seed'] in {661135201, 661135202}
                 and x['player'] in {'queued', 'hour10'}]
    write(OUT / 'confirmation/representative-actions.json', witnesses)
    native_commit = (RUN / 'source-commit.txt').read_text().strip()
    write(OUT / 'final-summary.json', dict(
        family='interrupt-or-finish', stage='Marlow Cut', status='validated',
        placement='ignored experimental arena catalog', catalog=str(catalog.relative_to(OUT)),
        victory=dict(territory=hex_at(POST), holdRounds=6, startingOwner='enemy', startingHoldProgress=0,
                     alternative='Defeat the opposing commander. Both sides have that alternative.'),
        opponent='Classic', recruitLimit=0, maxRounds=30, fog='Full visibility for both sides',
        pair='Same terrain, factions, unit stats, and objective rule. Only enemy commander placement differs.',
        conditions=fixtures,
        discovery=dict(seeds=discovery['seeds'], summary='discovery-2/summary.json', games='discovery-2/games.jsonl',
                       protocol='discovery-2/protocol.json'),
        confirmation=dict(seeds=confirmation['seeds'], games='confirmation/games.jsonl', total=len(traces),
                          reference='Queued', protocol='confirmation/protocol.json', results=metrics,
                          rawPhases='confirmation/traces/raw-phases.jsonl.gz',
                          objectiveTraces='confirmation/traces/objective-traces.json',
                          representativeActions='confirmation/representative-actions.json',
                          observationEvidence='confirmation/observation-evidence.json'),
        provenance=dict(nativeSourceCommit=native_commit, native=confirmation['native'],
                        modelDigests=confirmation['models'], nativeFingerprints=observation['nativeFingerprints'],
                        assetDigests=confirmation['sha256'],
                        methods={name: digest(OUT / name) for name in ('build.py', 'evaluate.py', 'observe.py',
                                                                       'trace.py', 'probe/src/main.rs', 'render.py')}),
        limits=['Changing commander placement also changes its ability to pressure the post after the first action.',
                'The finish condition rewards recognizing the earlier, safer finish. A post-first plan can still win.',
                'The pair holds the clock length fixed. It does not isolate sensitivity to clock length or measure training transfer.']))


if __name__ == '__main__':
    main()
