"""Replay native matches, require outcome parity, and extract objective decisions."""
import argparse
import gzip
import json
import os
from pathlib import Path
import subprocess

from build import OUT, POST, hex_at, read, write
from evaluate import digest

PROBE = OUT / 'probe/marlow-trace'


def territory_owner(phase):
    return next(tile['owner'] for tile in phase['ownership'] if tile['tile'] == hex_at(POST))


def phase_rows(battle, turns):
    control = 'enemy'
    held = {'player': 0, 'enemy': 0}
    for phase in battle['phases']:
        owner = territory_owner(phase)
        before = dict(held)
        final = phase is battle['phases'][-1]
        if not final or battle['reason'] != 'leader':
            for team in held:
                if owner != team:
                    held[team] = 0
                elif (team == 'player') != (phase['side'] == 'player'):
                    held[team] += 1
        actions = [action for action in phase['actions'] if action['kind'] in ('move', 'attack', 'advance')]
        yield dict(round=phase['round'], side=phase['side'], ownerBefore=control, ownerAfter=owner,
                   playerHeldBefore=before['player'], playerHeldAfter=held['player'],
                   enemyHeldBefore=before['enemy'], enemyHeldAfter=held['enemy'],
                   enemyHoldRemainingBefore=turns - before['enemy'], enemyHoldRemainingAfter=turns - held['enemy'],
                   unitsBefore={unit['id']: [*unit['tile'].values(), unit['hp']] for unit in phase['before']},
                   unitsAfter={unit['id']: [*unit['tile'].values(), unit['hp']] for unit in phase['units']},
                   actions=actions, lost=[unit['id'] for unit in phase['losses']])
        control = owner
    assert all(held[team] == battle['objectives'][team]['held'] for team in held)


def compact(player, config, battle, turns):
    phases = list(phase_rows(battle, turns))
    player_phases = [phase for phase in phases if phase['side'] == 'player']
    actions = [(phase['round'], action) for phase in player_phases for action in phase['actions']]
    first_knight = next((action['tile'] for round_, action in actions
                         if action['kind'] == 'move' and action['unit'] == 'u2' and round_ == 1), None)
    first_attack = next((round_ for round_, action in actions
                         if action['kind'] == 'attack' and action['target'] == 'u21'), None)
    capture = next((phase['round'] for phase in player_phases if phase['ownerAfter'] == 'player'), None)
    first_held_before_capture = next((phase['enemyHeldBefore'] for phase in player_phases
                                      if phase['ownerAfter'] == 'player'), None)
    first_enemy_phase = next((phase for phase in phases if phase['side'] == 'ai'), None)
    return dict(player=player, config=config, winner=battle['winner'], reason=battle['reason'], rounds=battle['rounds'],
                objectives=battle['objectives'], firstKnightMove=first_knight,
                firstCommanderAttack=first_attack, firstCapture=capture,
                enemyHeldBeforeFirstCapture=first_held_before_capture,
                enemyUnitsActingRoundOne=sorted({a['unit'] for a in first_enemy_phase['actions']})
                if first_enemy_phase else [], phases=phases)


def run(dest, label):
    protocol = read(dest / 'protocol.json')
    catalog = Path(protocol['catalog'])
    all_games = [json.loads(line) for line in (dest / 'games.jsonl').read_text().splitlines()]
    expected = {(row['player'], row['config']['landscape'], row['config']['seed']): row['result'] for row in all_games}
    arena_rules = {rule['id']: rule for rule in read(catalog / 'simulation/arena.json')['scenarios']}
    target = dest / label
    target.mkdir(exist_ok=True)
    tools = dict(replay=dict(path=str(PROBE), sha256=digest(PROBE)),
                 source=dict(path=str(OUT / 'probe/src/main.rs'), sha256=digest(OUT / 'probe/src/main.rs')))
    write(target / 'replay-protocol.json', tools)
    env = dict(os.environ, HEX_COMMAND_DATA_ROOT=str(catalog), RAYON_NUM_THREADS='1')
    compact_rows = []
    with gzip.open(target / 'raw-phases.jsonl.gz', 'wt') as raw_file:
        for player, cases in protocol['cases'].items():
            if label == 'sample':
                cases = [case for case in cases if case['seed'] == protocol['seeds'][0]]
            case_path = target / f'{player}-cases.json'
            write(case_path, cases)
            model = protocol['models'].get(player, {}).get('path', '-')
            with subprocess.Popen([PROBE, model, str(catalog), str(case_path)], text=True, env=env,
                                  stdout=subprocess.PIPE) as process:
                paired = zip(cases, process.stdout, strict=True)
                for case, line in paired:
                    battle = json.loads(line)
                    want = expected[player, case['landscape'], case['seed']]
                    assert (battle['winner'], battle['rounds'], battle['reason']) == (
                        want['winner'], want['rounds'], want['reason']), (player, case, want)
                    raw_file.write(json.dumps(dict(player=player, config=case, battle=battle)) + '\n')
                    compact_rows.append(compact(player, case, battle, arena_rules[case['landscape']]['holdTurns']))
                assert process.wait() == 0, player
    write(target / 'objective-traces.json', compact_rows)
    print(f'{len(compact_rows)} native outcomes matched their detailed replays')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('panel', default='confirmation', nargs='?')
    parser.add_argument('--sample', action='store_true')
    args = parser.parse_args()
    run(OUT / args.panel, 'sample' if args.sample else 'traces')
