"""Replay observed arena positions with/without thorns; never estimates long-term wins.

Explicit matching --data-dir is required. Full fingerprint matching is the default;
--allow-fingerprint-mismatch permits a different engine/catalog only after matching
all recorded unit-profile fields, and records that weaker validation prominently.
Uncapped recruitment uses a practical 1000-action ceiling. Beginning-turn effects
are already in phase.before: neither beginTurn nor advanceTurn is executed.
"""
import argparse
import copy
import csv
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

from unit_usage import adjacent, journals


def catalog(root):
    return {d['id']: d for p in sorted((root / 'game/units').glob('*.json'))
            for d in [json.loads(p.read_text())]}


def validate_profiles(profiles, units):
    if not profiles:
        raise ValueError('Report needs recorded unitProfiles')
    for name, profile in profiles.items():
        if name not in units:
            raise ValueError(f'Catalog missing {name}')
        rule = units[name]
        for key, value in profile.items():
            default = False if key == 'leader' else 0 if key in ('healing', 'thornAura', 'thorns') else None
            actual = rule.get(key, default)
            if key == 'family':
                actual = rule.get(key, name)
            if actual != value:
                raise ValueError(f'Catalog/profile mismatch: {name}.{key}: {actual!r} != {value!r}')


def combat_rng_states(detail):
    """Recover phase-start RNG only when every recorded draw reaches the final RNG.

    Every attack and retaliation raw roll consumes one LCG step, even with zero
    spread. Rejected samples consume additional steps absent from the journal;
    the final-state check rejects those histories rather than silently guessing.
    """
    def recorded_rng(boundary):
        value = detail.get(boundary, {}).get('combatRng')
        if type(value) is not int or not 0 <= value <= 0xffffffff:
            raise ValueError(f'Combat RNG reconstruction requires recorded {boundary}.combatRng')
        return value

    rng = recorded_rng('initial')
    expected = recorded_rng('final')
    states = []
    for phase in detail['phases']:
        states.append(rng)
        combats = phase.get('combats')
        if not isinstance(combats, list):
            raise ValueError('Combat RNG reconstruction requires recorded phase combats')
        for combat in combats:
            for field in ('rawRolls', 'responseRawRolls'):
                rolls = combat.get(field)
                if not isinstance(rolls, list) or any(type(roll) is not int for roll in rolls):
                    raise ValueError(f'Combat RNG reconstruction requires recorded {field}')
                for _ in rolls:
                    rng = (rng * 1664525 + 1013904223) & 0xffffffff
    if rng != expected:
        raise ValueError('Combat RNG reconstruction does not match final.combatRng; '
                         'rejected random samples or incomplete/unsupported draw records cannot be replayed exactly')
    return states


def before_state(detail, index, units):
    phase = detail['phases'][index]
    state = copy.deepcopy(detail['initial'])
    state.update(units=copy.deepcopy(phase['before']), round=phase['round'],
                 turn=phase['side'], gold=copy.deepcopy(phase['gold']), winner=None, lastCombat=None,
                 combatRng=combat_rng_states(detail)[index])
    # Gold is recorded after spending. Promotions are free; only recruits spend.
    for action in phase['actions']:
        if action['kind'] == 'recruit':
            state['gold'][phase['side']] += units[action['type']]['cost']
    ownership = (detail['phases'][index - 1]['ownership'] if index else
                 [{'tile': t['id'], 'owner': t.get('owner')} for t in state['tiles']])
    owners = {t['tile']: t['owner'] for t in ownership}
    for tile in state['tiles']:
        tile['owner'] = owners[tile['id']]
    known = [int(u['id'][1:]) for p in detail['phases'][:index]
             for field in ('before', 'units', 'losses') for u in p[field]]
    known += [int(u['id'][1:]) for u in phase['before']]
    state['nextId'] = max(state['nextId'], max(known, default=0) + 1)
    return state


def coverage(state, units, effect='aura'):
    friends = [u for u in state['units'] if u['side'] != state['turn']]
    providers = [u for u in friends if (
        effect in ('aura', 'all') and units[u['type']].get('thornAura', 0)) or (
        effect in ('intrinsic', 'all') and units[u['type']].get('thorns', 0))]
    covered = {u['id'] for u in friends if any(
        p['id'] != u['id'] and units[p['type']].get('thornAura', 0)
        and effect in ('aura', 'all') and adjacent(p['tile'], u['tile']) for p in providers)}
    if effect in ('intrinsic', 'all'):
        covered.update(u['id'] for u in friends if units[u['type']].get('thorns', 0))
    return providers, covered


def without_effect(rule, effect):
    altered = copy.deepcopy(rule)
    for key in ('thornAura', 'thorns') if effect == 'all' else (
            'thornAura' if effect == 'aura' else 'thorns',):
        if key in altered:
            altered[key] = 0
    return altered


def threatened(state, units, covered):
    """Conservative geometric reach; blockers and terrain may prevent an attack."""
    def distance(a, b):
        x, y = map(int, a.split(','))
        u, v = map(int, b.split(','))
        q, r = x - y // 2 - u + v // 2, y - v
        return (abs(q) + abs(r) + abs(q + r)) // 2
    for enemy in state['units']:
        if enemy['side'] != state['turn']:
            continue
        rule = units[enemy['type']]
        if not any(a['category'] == 'melee' for a in rule['attacks']):
            continue
        if any(u['id'] in covered and distance(enemy['tile'], u['tile']) <= rule['move'] + 1
               for u in state['units']):
            return True
    return False


def measurements(response, state, covered):
    combats = response['records']['combats']
    defenders = [u for u in state['units'] if u['side'] != state['turn']]
    attackers = [u for u in state['units'] if u['side'] == state['turn']]
    final = {u['id']: u for u in response['final']['units']}
    result = dict(attacks=len(combats), coveredAttacks=sum(c['target'] in covered for c in combats),
                  attackDamage=sum(c['damage'] for c in combats),
                  retaliation=sum(c['retaliation'] for c in combats),
                  auraReflection=sum(c.get('thorns', {}).get('auraDamage', 0) for c in combats),
                  totalReflection=sum(c.get('thorns', {}).get('damage', 0) for c in combats),
                  effectiveReflection=sum(c.get('thorns', {}).get('effectiveDamage', 0) for c in combats))
    for name, roster in [('defender', defenders), ('attacker', attackers)]:
        result[name + 'OriginalHPLost'] = sum(u['hp'] - final.get(u['id'], {}).get('hp', 0) for u in roster)
        result[name + 'OriginalDeaths'] = sum(u['id'] not in final for u in roster)
    return result


def switches(on, off):
    def attacks(response):
        result = {}
        for c in response['records']['combats']:
            result.setdefault(c['unit'], []).append((c['target'], c['weapon']))
        return result
    a, b = attacks(on), attacks(off)
    common = set(a) & set(b)
    return dict(attackersSkippingWithEffect=len(set(b) - set(a)),
                attackersAttackingOnlyWithEffect=len(set(a) - set(b)),
                targetSwitches=sum([x[0] for x in a[k]] != [x[0] for x in b[k]] for k in common),
                weaponSwitches=sum([x[1] for x in a[k]] != [x[1] for x in b[k]] for k in common))


def verify_original_phase(response, phase):
    if (response['records']['actions'] != phase['actions']
            or response['final']['units'] != phase['units']
            or response['final']['gold'] != phase['gold']):
        raise ValueError('Unchanged phase replay does not reproduce recorded actions, units and gold')


class Replay:
    def __init__(self, binary, data):
        self.errors = tempfile.TemporaryFile(mode='w+t')
        self.process = subprocess.Popen([str(binary), '--phase-stdio', '--data-dir', str(data)],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=self.errors, text=True)

    def run(self, request):
        self.process.stdin.write(json.dumps(request) + '\n')
        self.process.stdin.flush()
        line = self.process.stdout.readline()
        if not line:
            self.errors.seek(0)
            raise RuntimeError('Phase replay failed: ' + self.errors.read())
        return json.loads(line)

    def close(self):
        self.process.stdin.close()
        self.process.wait(timeout=30)
        self.process.stdout.close()
        self.errors.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--data-dir', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True, help='Output prefix; writes .csv and .json')
    parser.add_argument('--binary', type=Path, default=Path('rust/target/simulation/hex-command-sim'))
    parser.add_argument('--limit', type=int, default=50)
    parser.add_argument('--effect', choices=('aura', 'intrinsic', 'all'), default='aura')
    parser.add_argument('--policy', help='Only sample phases executed by this tactic')
    parser.add_argument('--per-seed', type=int, help='Cap sampled positions per seed to spread observations')
    parser.add_argument('--engaged-only', action='store_true', help='Require a potential incoming melee attack within geometric movement reach')
    parser.add_argument('--allow-fingerprint-mismatch', action='store_true')
    args = parser.parse_args()
    if args.limit < 1:
        parser.error('--limit must be positive')
    if args.per_seed is not None and args.per_seed < 1:
        parser.error('--per-seed must be positive')
    report_path = args.report / 'report.json' if args.report.is_dir() else args.report
    report = json.loads(report_path.read_text())
    units = catalog(args.data_dir)
    validate_profiles(report.get('unitProfiles'), units)
    fingerprints = json.loads(subprocess.check_output(
        [str(args.binary), '--fingerprint', '--data-dir', str(args.data_dir)], text=True))
    fingerprint = hashlib.sha256(json.dumps(fingerprints, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    strict = fingerprint == report.get('fingerprint')
    if not strict and not args.allow_fingerprint_mismatch:
        raise ValueError('Runtime/report fingerprint differs; use the original binary/catalog, or explicitly allow weaker profile-only validation')
    rows, pairs, seed_counts = [], [], {}
    with tempfile.TemporaryDirectory(prefix='hex-thorn-aura-off-') as scratch:
        altered = Path(scratch) / 'data'
        shutil.copytree(args.data_dir, altered)
        for path in (altered / 'game/units').glob('*.json'):
            path.write_text(json.dumps(without_effect(json.loads(path.read_text()), args.effect)))
        processes = [Replay(args.binary, root) for root in (args.data_dir, altered)]
        stream = journals([report_path])
        try:
            for _, detail in stream:
                config = detail['config']
                if config['armies']['player'] == config['armies']['enemy']:
                    continue
                if args.per_seed and seed_counts.get(config['seed'], 0) >= args.per_seed:
                    continue
                for index, phase in enumerate(detail['phases']):
                    policy = config[phase['side']]
                    if args.policy and policy != args.policy:
                        continue
                    providers, covered = coverage(dict(units=phase['before'], turn=phase['side']), units, args.effect)
                    if not providers:
                        continue
                    if args.engaged_only and not threatened(dict(units=phase['before'], turn=phase['side']), units, covered):
                        continue
                    state = before_state(detail, index, units)
                    limit = config.get('recruitLimit')
                    if limit is None:
                        limit = 2 if policy == 'classic' else 1000
                    request = dict(state=state, arena=config, policy=policy, seed=config['seed'],
                                   goals=phase['objectives'], recruitmentLimit=limit)
                    if config.get(phase['side'] + 'Blocks'):
                        request['blocks'] = config[phase['side'] + 'Blocks']
                    on, off = [process.run(request) for process in processes]
                    if strict:
                        verify_original_phase(on, phase)
                    row = dict(sample=len(rows), effect=args.effect, seed=config['seed'], map=config['landscape'],
                               round=phase['round'], side=phase['side'], policy=policy,
                               defenderFaction=config['armies']['enemy' if phase['side'] == 'player' else 'player'],
                               providers=len(providers), coveredAllies=len(covered), recruitmentLimit=limit,
                               **switches(on, off))
                    for label, response in [('on', on), ('off', off)]:
                        row.update({label + k[0].upper() + k[1:]: v for k, v in measurements(response, state, covered).items()})
                    rows.append(row)
                    seed_counts[config['seed']] = seed_counts.get(config['seed'], 0) + 1
                    pairs.append(dict(sample=row['sample'], request=request, auraOn=on, auraOff=off))
                    if len(rows) >= args.limit:
                        break
                    if args.per_seed and seed_counts[config['seed']] >= args.per_seed:
                        break
                if len(rows) >= args.limit:
                    break
        finally:
            stream.close()
            for process in processes:
                process.close()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.with_suffix('.json').write_text(json.dumps(dict(
        report=str(report_path), sourceFingerprint=report.get('fingerprint'), runtimeFingerprints=fingerprints,
        validation='strict fingerprint and profiles' if strict else 'profiles only; full catalog/engine match NOT verified',
        originalPhaseReproductionChecked=strict,
        effect=args.effect, policy=args.policy, perSeed=args.per_seed, engagedOnly=args.engaged_only,
        sampling='first observed non-mirror positions with the selected effect in journal order; correlated within battles',
        limitations='One phase, recorded phase-start combat RNG restored and final journal RNG verified; rejected-sample histories fail rather than guess. No long-term win attribution. Uncapped recruitment represented by ceiling 1000. HP deltas include healing/drain and original units only.',
        rows=rows, pairs=pairs), indent=2))
    with args.out.with_suffix('.csv').open('w', newline='') as output:
        writer = csv.DictWriter(output, fieldnames=list(rows[0]) if rows else ['sample'])
        writer.writeheader()
        writer.writerows(rows)
    print(json.dumps(dict(positions=len(rows), strictFingerprint=strict, output=str(args.out))))


if __name__ == '__main__':
    main()
