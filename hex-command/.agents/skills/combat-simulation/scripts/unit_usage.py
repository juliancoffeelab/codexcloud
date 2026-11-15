"""Recorded arena presence, damage/presence ratios and support effects."""
from collections import Counter, defaultdict
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path


# The legacy arena "race" column identifies a faction from config.armies.
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[4] / 'scripts'))
from simulation_journal import analysis_view

DIMS = ('race', 'policy', 'opponentPolicy', 'map')
UNIT_KINDS = {'warrior': 'infantry', 'flank': 'flank', 'ranger': 'ranger', 'healer': 'support'}


def cell_key(tile):
    """Hashable analysis key for current structured hexes and historical journals."""
    return f"{tile['col']},{tile['row']}" if isinstance(tile, dict) else tile


def damage_order(row):
    kind = row.get('unitKind', 'unknown')
    return ((*UNIT_KINDS.values(), 'unknown').index(kind), row['family'],
            row.get('level') or 0, row['name'])


def adjacent(a, b, positions):
    x, y = positions[cell_key(a)]
    u, v = positions[cell_key(b)]
    q, r = x - y // 2 - u + v // 2, y - v
    return abs(q) + abs(r) + abs(q + r) == 2


def journals(inputs, kinds=('arena',)):
    for path in inputs:
        if path.is_dir():
            path = path / ('report.json' if (path / 'report.json').exists() else 'report.json.gz')
        with gzip.open(path, 'rt') if path.suffix == '.gz' else path.open() as stream:
            report = analysis_view(json.load(stream))
        if report.get('status') != 'complete':
            raise ValueError('Unit usage requires a completed run')
        readers = {}
        try:
            for saved in sorted(report['outcomes'], key=lambda o: (o.get('detail', {}).get('file', ''), o.get('detail', {}).get('offset', 0))):
                if saved.get('kind') not in kinds:
                    continue
                ref = saved.get('detail', {})
                name = ref.get('file', '')
                if not name or Path(name).name != name or ref.get('offset', -1) < 0 or ref.get('bytes', 0) <= 0:
                    raise ValueError('Unit usage requires recorded gameplay journals')
                if name not in readers:
                    journal = path.parent / name
                    readers[name] = journal.open('rb') if journal.exists() else gzip.open(str(journal) + '.gz', 'rb')
                stream = readers[name]
                stream.seek(ref['offset'])
                raw = stream.read(ref['bytes'])
                if len(raw) != ref['bytes'] or hashlib.sha256(raw).hexdigest() != ref['sha256']:
                    raise ValueError('Unit usage gameplay checksum mismatch')
                if name.endswith('.arrow'):
                    if importlib.util.find_spec('pyarrow') is None:
                        raise ModuleNotFoundError('pyarrow')
                    decoder_path = Path(__file__).resolve().parents[4] / 'scripts/simulation_journal.py'
                    spec = importlib.util.spec_from_file_location('usage_journal', decoder_path)
                    decoder = importlib.util.module_from_spec(spec)
                    spec.loader.exec_module(decoder)
                    result = decoder.decode_detail(raw, name)
                else:
                    result = json.loads(raw)
                result = analysis_view(result)
                if result['config'] != saved['config']:
                    raise ValueError('Unit usage journal configuration mismatch')
                yield report, result
        finally:
            for stream in readers.values():
                stream.close()


def analyze(inputs, presence=True, support=True):
    census = defaultdict(Counter)
    totals = defaultdict(Counter)
    present = Counter()
    role_events = defaultdict(Counter)
    effects = defaultdict(Counter)
    overlaps = defaultdict(Counter)
    catalogs = None
    matches = 0
    for report, result in journals(inputs):
        config = result['config']
        if config['armies']['player'] == config['armies']['enemy']:
            continue
        profiles = report.get('unitProfiles') or {}
        families = report.get('unitFamilies') or {}
        if support and (report.get('supportTelemetryVersion') != 1 or not profiles):
            raise ValueError('Support effects require a fresh run with support telemetry and recorded unitProfiles')
        if profiles:
            if catalogs is not None and catalogs != profiles:
                raise ValueError('Cannot pool unit usage across different recorded catalogs')
            catalogs = profiles
        matches += 1

        def group(side, landscape):
            other = 'enemy' if side == 'player' else 'player'
            def label(s):
                blocks = config.get(s + 'Blocks')
                return '-'.join(blocks[k] for k in ('recruiter', 'campaign', 'planner')) if blocks else config[s]
            return config['armies'][side], label(side), label(other), landscape

        def leader(unit):
            return unit.get('role') == 'leader'

        def family(unit):
            return profiles.get(unit, {}).get('family', families.get(unit, unit))

        def add_effect(unit, ability, **values):
            # All-unit rows aggregate the same measured events, not extra events.
            for landscape in (config['landscape'], 'all'):
                key = group(unit['side'], landscape)
                for kind in (unit['type'], 'all'):
                    effects[(*key, kind, ability)].update(values)

        positions = {t['id']: (t['col'], t['row']) for t in result['initial']['tiles']}
        seen = {side: set() for side in ('player', 'enemy')}
        for unit in result['initial']['units']:
            if not leader(unit):
                seen[unit['side']].add(unit['type'])
        for side in ('player', 'enemy'):
            for landscape in (config['landscape'], 'all'):
                totals[group(side, landscape)]['battles'] += 1
        for phase in result['phases']:
            side = phase['side']
            if presence:
                own = [u for u in phase['units'] if u['side'] == side and not leader(u)]
                for landscape in (config['landscape'], 'all'):
                    key = group(side, landscape)
                    totals[key]['phases'] += 1
                    totals[key]['unitTurns'] += len(own)
                    for unit in own:
                        census[key][unit['type']] += 1
                for unit in phase.get('before', []) + phase['units'] + phase.get('losses', []):
                    if not leader(unit):
                        seen[unit['side']].add(unit['type'])
            if presence:
                # Match the engine's post-promotion/casualty lookup order, but
                # keep instance roles through the aggregation boundary.
                roster = {u['id']: u for u in phase.get('before', []) + phase.get('losses', []) + phase['units']}
                def role_event(unit, **stats):
                    if leader(unit):
                        return
                    key = (*group(unit['side'], config['landscape']), unit['type'])
                    role_events[key].update(stats)
                for loss in phase.get('losses', []):
                    role_event(loss, lost=1)
                for action in phase.get('actions', []):
                    if action['kind'] == 'advance':
                        role_event(roster[action['unit']], promoted=1)
                    elif action['kind'] == 'recruit':
                        previous = {u['id'] for u in phase.get('before', [])}
                        spawned = [u for u in phase['units'] if u['id'] not in previous and u['type'] == action['type'] and u['tile'] == action['tile'] and u['side'] == side]
                        if len(spawned) != 1:
                            raise ValueError('Cannot identify recorded recruit role')
                        role_event(spawned[0], recruited=1)
                for combat in phase.get('combats', []):
                    role_event(roster[combat['unit']], attacks=1, damage=combat.get('damage', 0))
                    if combat.get('retaliationCount', 0) or combat.get('retaliation', 0):
                        role_event(roster[combat['target']], retaliations=int(combat.get('retaliationCount', 0) > 0), damage=combat.get('retaliation', 0))
            if not support:
                continue
            before = phase['before']
            lookup = {u['id']: u for u in phase.get('losses', []) + phase['units'] + before}
            for ability in ('healing', 'thornAura', 'thorns'):
                # Healing opportunities occur at own-turn start. Reflection
                # opportunities occur at opposing-turn start, after positioning.
                provider_side = side if ability == 'healing' else ('enemy' if side == 'player' else 'player')
                friends = [u for u in before if u['side'] == provider_side]
                providers = [u for u in friends if profiles[u['type']].get(ability, 0) > 0]
                for provider in providers:
                    add_effect(provider, ability, providerTurns=1)
                if ability == 'thorns':
                    continue
                composition = Counter(opportunities=1, eligiblePairs=len(providers) * max(0, len(friends)-1))
                for ally in friends:
                    touching = [p for p in providers if p['id'] != ally['id'] and adjacent(p['tile'], ally['tile'], positions)]
                    composition['contacts'] += len(touching)
                    composition['coveredRecipients'] += bool(touching)
                    composition['overlappingRecipients'] += len(touching) > 1
                    for provider in touching:
                        add_effect(provider, ability, adjacentContacts=1,
                                   coveredRecipients=1 / len(touching),
                                   overlappingRecipients=(1 / len(touching) if len(touching) > 1 else 0))
                if providers:
                    for landscape in (config['landscape'], 'all'):
                        overlaps[(*group(provider_side, landscape), ability, len(providers), max(0, len(friends)-1))].update(composition)
            for event in phase.get('healing', []):
                providers = event['providers']
                if not providers:
                    raise ValueError('Healing event has no provider')
                for provider in providers:
                    add_effect(lookup[provider], 'healing', healedHP=event['amount'] / len(providers),
                               healedRecipients=1 / len(providers))
            for combat in phase['combats']:
                thorns = combat.get('thorns')
                if thorns is None:
                    raise ValueError('Support telemetry missing from combat record')
                raw = thorns['damage']
                if raw <= 0:
                    continue
                own = raw - thorns['auraDamage']
                effective = thorns['effectiveDamage']
                if own:
                    add_effect(lookup[combat['target']], 'thorns', triggeredCombats=1,
                               reflectedDamage=own, effectiveReflectedDamage=effective * own / raw)
                sources = thorns['auraSources']
                if thorns['auraDamage'] and not sources:
                    raise ValueError('Thorn aura event has no provider')
                for source in sources:
                    damage = thorns['auraDamage'] / len(sources)
                    add_effect(lookup[source], 'thornAura', triggeredCombats=1 / len(sources),
                               reflectedDamage=damage, effectiveReflectedDamage=effective * damage / raw)
        for side, units in seen.items():
            for landscape in (config['landscape'], 'all'):
                key = group(side, landscape)
                for unit in units:
                    present[(*key, 'unit', unit)] += 1
                for fam in {family(unit) for unit in units}:
                    present[(*key, 'family', fam)] += 1

    unit_rows, family_rows = [], []
    for key, total in sorted(totals.items()):
        if not presence:
            break
        counts = census[key]
        for grouping, target in [('unit', unit_rows), ('family', family_rows)]:
            units = {k[-1] for k in present if k[:4] == key and k[4] == grouping}
            for unit in sorted(units):
                turns = counts[unit] if grouping == 'unit' else sum(v for k, v in counts.items() if (catalogs.get(k, {}).get('family', k) if catalogs else k) == unit)
                # Legacy presence has only unitFamilies; preserve its recorded grouping.
                if grouping == 'family' and not catalogs:
                    turns = sum(v for k, v in counts.items() if families.get(k, k) == unit)
                target.append(dict(zip(DIMS, key), **{grouping: unit}, unitTurns=turns,
                                   totalUnitTurns=total['unitTurns'], sharePercent=100 * turns / max(1, total['unitTurns']),
                                   observedPhases=total['phases'], meanUnitsPerPhase=turns / max(1, total['phases']),
                                   battlesPresent=present[(*key, grouping, unit)], battles=total['battles'],
                                   battlePresencePercent=100 * present[(*key, grouping, unit)] / max(1, total['battles'])))
    effect_rows = []
    for (*key, unit, ability), counts in sorted(effects.items()):
        turns = counts['providerTurns']
        effect_rows.append(dict(zip(DIMS, key), unit=unit, family=(catalogs[unit]['family'] if unit != 'all' else 'all'),
                               ability=ability, **{k: counts[k] for k in ['providerTurns','adjacentContacts','coveredRecipients','overlappingRecipients','healedHP','healedRecipients','triggeredCombats','reflectedDamage','effectiveReflectedDamage']},
                               adjacentAlliesPerTurn=(counts['adjacentContacts'] / turns if turns and ability != 'thorns' else None),
                               usefulRecipientsPerTurn=(counts['healedRecipients'] / turns if turns and ability == 'healing' else None),
                               healedHPPerTurn=(counts['healedHP'] / turns if turns and ability == 'healing' else None),
                               reflectedDamagePerTurn=(counts['reflectedDamage'] / turns if turns and ability != 'healing' else None),
                               overlapPercent=(100 * counts['overlappingRecipients'] / counts['coveredRecipients'] if counts['coveredRecipients'] else 0)))
    tables = {}
    if presence:
        tables.update(battlefield_presence=family_rows, battlefield_units=unit_rows,
                      nonleader_unit_events=[dict(zip((*DIMS, 'unit'), key), **{field: stats[field] for field in ('recruited','promoted','lost','attacks','retaliations','damage')}) for key, stats in sorted(role_events.items())])
    if support:
        tables['support_effects'] = effect_rows
        tables['support_overlap'] = [dict(zip(DIMS, key[:4]), ability=key[4], providerCount=key[5], recipientsPerProvider=key[6], **stats,
            pairCoverageProbability=stats['contacts'] / max(1, stats['eligiblePairs']),
            overlapPercent=100 * stats['overlappingRecipients'] / max(1, stats['coveredRecipients']),
            redundantContactPercent=100 * (stats['contacts'] - stats['coveredRecipients']) / max(1, stats['contacts']))
            for key, stats in sorted(overlaps.items())]
    return tables, matches


def army_usage(events, purchases, presence, families):
    totals = defaultdict(Counter)
    counts = defaultdict(Counter)
    for event in events:
        if event['race'] == event.get('opponentRace'):
            continue
        for landscape in (event['map'], 'all'):
            key = tuple(event[k] for k in DIMS[:-1]) + (landscape,)
            family = families.get(event['unit'], event['unit'])
            for field in ('recruited','promoted','lost','attacks','retaliations','damage'):
                counts[(*key, family)][field] += event[field]
                totals[key][field] += event[field]
    buys = {tuple(r[k] for k in DIMS) + (r['family'],): r for r in purchases}
    rows = []
    for row in presence:
        key = tuple(row[k] for k in DIMS)
        own = (*key, row['family'])
        stats = {field: counts[own][field] for field in ('recruited','promoted','lost','attacks','retaliations','damage')}
        total = totals[key]
        rows.append(dict(row, purchaseSharePercent=buys.get(own, {}).get('sharePercent', 0),
                         attackSharePercent=100 * stats['attacks'] / max(1, total['attacks']),
                         damageSharePercent=100 * stats['damage'] / max(1, total['damage']),
                         attacksPerUnitTurn=stats['attacks'] / max(1, row['unitTurns']),
                         retaliationsPerUnitTurn=stats['retaliations'] / max(1, row['unitTurns']), **stats))
    return rows


def unit_damage(presence, battlefield_units, profiles, membership=()):
    """Exact-unit damage share / presence share, with one nonleader denominator.

    Weapon HP damage includes retaliation and excludes overkill. Effective
    provider-attributed thorns stay separate from weapon shares and D/P.
    Both come from the maintained battlefield-value audit.
    Faction membership comes from recorded rows, never mechanical race. Units
    recorded in another scope (or the comparison run) retain zero-exposure rows;
    units never recorded for that faction are omitted.
    Never average map percentages or per-unit rates to create faction totals.
    """
    modern = any('nonleaderUnitTurns' in row for row in battlefield_units)
    if modern:
        battlefield_units = [dict(row, unitTurns=row['nonleaderUnitTurns'],
                                  weaponHPDamageDealt=row['nonleaderWeaponHPDamageDealt'],
                                  thornHPDamageDealt=row['nonleaderThornHPDamageDealt'])
                             for row in battlefield_units]
    groups = defaultdict(lambda: dict(presence={}, damage={}))
    faction_units = defaultdict(set)
    for rows in (presence, battlefield_units, membership):
        for row in rows:
            unit = row['unit']
            observed_nonleader = 'nonleaderUnitTurns' not in row or any(row.get(k, 0) for k in ('nonleaderUnitTurns', 'nonleaderWeaponHPDamageDealt', 'nonleaderThornHPDamageDealt'))
            if row.get('kind', 'arena') == 'arena' and unit in profiles and observed_nonleader and (modern or not profiles[unit].get('leader', False)):
                faction_units[row['race']].add(unit)
    for rows, field in [(presence, 'presence'), (battlefield_units, 'damage')]:
        for row in rows:
            if field == 'damage' and row['kind'] != 'arena':
                continue
            unit = row['unit']
            if unit not in profiles:
                raise ValueError('Missing recorded unit profile: ' + unit)
            group = groups[tuple(row[k] for k in DIMS)]
            if modern or not profiles[unit].get('leader', False):
                group[field][unit] = row
    result = []
    for key, group in sorted(groups.items()):
        census, values = group['presence'], group['damage']
        turns = sum(r['unitTurns'] for r in census.values())
        damage = sum(r['weaponHPDamageDealt'] for r in values.values())
        for unit in census.keys() | values.keys():
            if census.get(unit, {}).get('unitTurns', 0) != values.get(unit, {}).get('unitTurns', 0):
                raise ValueError('Damage/presence census mismatch: ' + unit)
        for unit in sorted(faction_units[key[0]]):
            profile = profiles[unit]
            observed = census.get(unit, {}).get('unitTurns', 0)
            stats = values.get(unit, {})
            dealt = stats.get('weaponHPDamageDealt', 0)
            presence_share = 100 * observed / turns if turns else 0.0
            damage_share = 100 * dealt / damage if damage else None
            result.append(dict(zip(DIMS, key), unit=unit, name=profile.get('name', unit),
                level=profile.get('level'), family=profile['family'],
                unitKind=UNIT_KINDS.get(profile.get('role'), 'unknown'), unitTurns=observed,
                totalUnitTurns=turns, damage=dealt, totalDamage=damage,
                weaponDamage=stats.get('weaponHPDamageDealt', 0),
                thornDamage=stats.get('thornHPDamageDealt', 0),
                thornDamagePerUnitTurn=stats.get('thornHPDamageDealt', 0) / observed if observed else None,
                presenceSharePercent=presence_share, damageSharePercent=damage_share,
                damagePerUnitTurn=dealt / observed if observed else None,
                factionDamagePerUnitTurn=damage / turns if turns else None,
                damagePresenceRatio=damage_share / presence_share
                    if presence_share and damage_share is not None else None))
    return sorted(result, key=lambda row: (tuple(row[k] for k in DIMS), damage_order(row)))


def compare_unit_damage(current, baseline):
    """Keep missing rates undefined and compare matching unit/scope rows only."""
    fields = ('unitTurns', 'damage', 'presenceSharePercent', 'damageSharePercent',
              'damagePerUnitTurn', 'damagePresenceRatio', 'thornDamage', 'thornDamagePerUnitTurn')
    index = {tuple(r[k] for k in (*DIMS, 'unit')): r for r in baseline}
    result = []
    for row in current:
        before = index.get(tuple(row[k] for k in (*DIMS, 'unit')))
        compared = dict(row)
        for field in fields:
            old = before.get(field) if before else None
            compared[field + 'Before'] = old
            compared[field + 'Delta'] = row[field] - old if row[field] is not None and old is not None else None
        result.append(compared)
    return result


def damage_markdown(rows, baseline=False):
    """The user-facing damage table: presence, damage/turn and D/P."""
    def number(value, suffix=''):
        return '—' if value is None else f'{value:.2f}{suffix}'

    def cell(row, field, suffix=''):
        after = number(row[field], suffix)
        return f'{number(row[field + "Before"], suffix)} → {after}' if baseline else after

    lines = ['| Unit | Kind | Presence | Damage/turn | D/P | Thorns/turn |', '| --- | --- | ---: | ---: | ---: | ---: |']
    for row in sorted(rows, key=damage_order):
        name = row['name'] + (f' (L{row["level"]})' if row.get('level') else '')
        lines.append(f'| {name} | {row.get("unitKind", "unknown")} | {cell(row, "presenceSharePercent", "%")} | '
                     f'{cell(row, "damagePerUnitTurn")} | {cell(row, "damagePresenceRatio", "×")} | '
                     f'{cell(row, "thornDamagePerUnitTurn")} |')
    return '\n'.join(lines)
