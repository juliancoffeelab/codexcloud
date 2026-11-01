"""Observed protection, objective contribution, losses and leader risk.

Reads recorded catalogs and checksummed journals, without running decisions.
Leader exposure is a forecast with other units held fixed, not a causal verdict.
"""
from collections import Counter, defaultdict
import json


CAPTURABLE = {'village', 'grove', 'fort', 'ruins'}
FIELDS = ('unitTurns', 'hpDamageDealt', 'hpDamageTaken', 'weaponHPDamageDealt', 'weaponHPDamageTaken',
          'thornHPDamageDealt', 'thornHPDamageTaken', 'armourDamagePrevented',
          'terrainDamagePrevented', 'rawArmourDelta', 'rawTerrainDelta',
          'attacksTaken', 'retaliationsTaken', 'captures', 'objectiveCaptures',
          'ownedTileTurns', 'objectiveControlTurns', 'capturableOccupationTurns',
          'objectiveOccupationTurns', 'casualties', 'recruitGoldLost',
          'casualtyReplacementValue')
DIMS = ('kind', 'campaign', 'diplomacy', 'objective', 'mission', 'race', 'policy', 'opponentPolicy', 'map', 'family')


def analyze(inputs, journals, value=True, leaders=True):
    counts = defaultdict(Counter)
    unit_counts = defaultdict(Counter)
    leader_rows = []
    catalogs = None
    for report, result in journals(inputs, kinds=('arena', 'campaign')):
        kind = result.get('kind', 'campaign' if 'missions' in result else 'arena')
        config = result['config']
        if kind == 'arena' and config['armies']['player'] == config['armies']['enemy']:
            continue
        if value and report.get('combatTelemetryVersion') != 1:
            raise ValueError('Battlefield value requires a fresh run with combat telemetry')
        if leaders and report.get('leaderAuditVersion') != 1:
            raise ValueError('Leader audit requires a fresh run with leader-risk telemetry')
        profiles = report.get('unitProfiles') or {}
        if catalogs is not None and profiles != catalogs:
            raise ValueError('Cannot pool battlefield value across different recorded catalogs')
        catalogs = profiles
        chapters = [result] if kind == 'arena' else result['missions']
        for chapter in chapters:
            mission = chapter.get('id', '')
            initial = chapter['initial']
            tiles = {t['id']: t for t in initial['tiles']}
            owners = {t: u.get('owner') for t, u in tiles.items()}
            captors, investments = {}, {}
            objective_tiles = {side: set() for side in ('player', 'enemy')}
            for phase in chapter['phases']:
                for objective in phase.get('objectives', []):
                    if objective.get('kind') == 'capture':
                        objective_tiles[phase['side']].update(objective['tiles'])

            def policy(side):
                blocks = config.get(side + 'Blocks')
                return '-'.join(blocks[k] for k in ('recruiter', 'campaign', 'planner')) if blocks else config[side]

            def identity(unit, landscape):
                side = unit['side']
                other = 'enemy' if side == 'player' else 'player'
                race = config['armies'][side] if kind == 'arena' else profiles[unit['type']]['race']
                return (kind, config.get('campaign', ''), config.get('diplomacy', ''), config.get('objective', 'authored'), mission,
                        race, policy(side), policy(other), landscape, profiles[unit['type']]['family'])

            def add(unit, **stats):
                landscape = config['landscape'] if kind == 'arena' else mission
                for label in (landscape, 'all'):
                    key = identity(unit, label)
                    counts[key].update(stats)
                    unit_counts[(*key[:-1], unit['type'])].update(stats)

            for phase_index, phase in enumerate(chapter['phases']):
                side = phase['side']
                lookup = {u['id']: u for u in phase['before'] + phase.get('losses', []) + phase['units']}
                if value:
                    for unit in phase['units']:
                        if unit['side'] == side:
                            add(unit, unitTurns=1)
                            tile = unit['tile']
                            if tiles[tile]['terrain'] in CAPTURABLE:
                                add(unit, capturableOccupationTurns=1)
                            if tile in objective_tiles[side]:
                                add(unit, objectiveOccupationTurns=1)
                    for action in phase.get('actions', []):
                        if action['kind'] == 'recruit':
                            # Action.unit is the recruiting leader, not the new
                            # troop. New recruits cannot move on their birth turn.
                            previous_ids = {u['id'] for u in phase['before']}
                            recruits = [u for u in lookup.values() if u['id'] not in previous_ids
                                and u['side'] == side and u['tile'] == action['tile']
                                and u['type'] == action['type'] and u['id'] not in investments]
                            if len(recruits) != 1:
                                raise ValueError('Cannot identify recorded new recruit from its deployment tile/type')
                            investments[recruits[0]['id']] = profiles[action['type']]['cost']
                        elif action['kind'] == 'move':
                            unit = lookup[action['unit']]
                            tile = action['tile']
                            if tiles[tile]['terrain'] in CAPTURABLE and owners[tile] != unit['side']:
                                owners[tile] = unit['side']
                                captors[tile] = dict(unit)
                                add(unit, captures=1, objectiveCaptures=int(tile in objective_tiles[unit['side']]))
                    for loss in phase.get('losses', []):
                        add(loss, casualties=1, recruitGoldLost=investments.pop(loss['id'], 0),
                            casualtyReplacementValue=profiles[loss['type']]['cost'])
                    for combat in phase.get('combats', []):
                        mitigation = combat.get('mitigation')
                        if mitigation is None:
                            raise ValueError('Combat mitigation telemetry missing from combat record')
                        attacker, target = lookup[combat['unit']], lookup[combat['target']]
                        for event, actor, defender, response in (
                            (mitigation['attack'], attacker, target, False),
                            (mitigation.get('retaliation'), target, attacker, True),
                        ):
                            if event is None:
                                continue
                            actual = (event['effectiveDamage'] if event.get('effectiveDamage') is not None
                                      else min(event['afterTerrainDamage'], event['availableHp']))
                            add(actor, hpDamageDealt=actual, weaponHPDamageDealt=actual)
                            add(defender, hpDamageTaken=actual, weaponHPDamageTaken=actual,
                                armourDamagePrevented=event['effectiveArmourPrevented'],
                                terrainDamagePrevented=event['effectiveTerrainPrevented'],
                                rawArmourDelta=event['rawDamage'] - event['afterArmourDamage'],
                                rawTerrainDelta=event['afterArmourDamage'] - event['afterTerrainDamage'],
                                attacksTaken=int(not response), retaliationsTaken=int(response))
                        thorns = combat.get('thorns') or {}
                        effective = thorns.get('effectiveDamage', 0)
                        if effective:
                            raw = thorns['damage']
                            own = raw - thorns['auraDamage']
                            add(attacker, hpDamageTaken=effective, thornHPDamageTaken=effective)
                            if own:
                                reflected = effective * own / raw
                                add(target, hpDamageDealt=reflected, thornHPDamageDealt=reflected)
                            sources = thorns['auraSources']
                            if thorns['auraDamage'] and not sources:
                                raise ValueError('Thorn aura event has no provider')
                            for source in sources:
                                reflected = effective * thorns['auraDamage'] / raw / len(sources)
                                add(lookup[source], hpDamageDealt=reflected, thornHPDamageDealt=reflected)
                    observed_owners = {o['tile']: o['owner'] for o in phase['ownership']}
                    for tile, captor in captors.items():
                        if observed_owners.get(tile) == side and captor['side'] == side:
                            add(captor, ownedTileTurns=1,
                                objectiveControlTurns=int(tile in objective_tiles[side]))
                    owners.update(observed_owners)
                if leaders:
                    # Death diagnostics retain both the death phase and the last
                    # own-phase decision, rather than blaming the killing hit.
                    for audit in phase.get('leaderAudit', []):
                        leader = lookup[audit['leader']]
                        row = dict(zip(DIMS[:-1], identity(leader, config.get('landscape', mission))[:-1]))
                        before, after = audit['armyBefore'], audit['armyAfter']
                        foe = audit['opponentBefore']
                        exposure = audit.get('exposure') or {}
                        row.update(matchId=kind + ':' + json.dumps(config, sort_keys=True, separators=(',', ':')),
                                   seed=config['seed'], first=config.get('first', ''), phase=phase_index,
                                   round=phase['round'], actingSide=side, side=audit['side'],
                                   leader=audit['leader'], unit=leader['type'], died=audit['after'] is None,
                                   beforeHP=audit['before']['hp'], afterHP=(audit['after'] or {}).get('hp', 0),
                                   beforeTile=audit['before']['tile'], afterTile=(audit['after'] or {}).get('tile'),
                                   ownDecision=side == audit['side'],
                                   armyUnitsBefore=before['units'], armyUnitsAfter=after['units'],
                                   armyHPValueBefore=before['hpValue'], armyHPValueAfter=after['hpValue'],
                                   opponentHPValueBefore=foe['hpValue'],
                                   opponentHPValueAfter=audit['opponentAfter']['hpValue'],
                                   armyValueRatioBefore=(before['hpValue'] / foe['hpValue'] if foe['hpValue'] else None),
                                   armyRetainedFraction=(after['hpValue'] / before['hpValue'] if before['hpValue'] else None),
                                   armyHPBefore=before['hp'], armyHPAfter=after['hp'],
                                   **audit['recruitmentBefore'], **exposure)
                        last_combat = next((c for c in reversed(phase.get('combats', []))
                                            if audit['leader'] in (c['unit'], c['target'])), {})
                        row['deathDuringOwnAttack'] = bool(row['died'] and last_combat.get('unit') == audit['leader'])
                        # This describes a snapshot forecast, not avoidable death.
                        chosen = exposure.get('chosenTileDamage')
                        minimum = exposure.get('minDamage')
                        row['forecastLethalWithSafeAlternative'] = (
                            chosen >= audit['before']['hp'] and minimum < audit['before']['hp']
                            if chosen is not None and minimum is not None else None)
                        row['endForecastLethal'] = (exposure['afterDamage'] >= row['afterHP']
                            if exposure.get('afterDamage') is not None and audit['after'] is not None else None)
                        leader_rows.append(row)
    rows = [dict(zip(DIMS, key), **{f: value[f] for f in FIELDS}) for key, value in sorted(counts.items())]
    tables = {}
    if value:
        tables['battlefield_value'] = rows
        tables['battlefield_unit_value'] = [dict(zip((*DIMS[:-1], 'unit'), key),
            family=catalogs[key[-1]]['family'], **{f: stats[f] for f in FIELDS})
            for key, stats in sorted(unit_counts.items())]
    if leaders:
        tables['leader_audit'] = leader_rows
        deaths = []
        last_decision = {}
        peaks = {}
        for row in leader_rows:
            key = tuple(row[k] for k in ('matchId', 'mission', 'side', 'leader'))
            peaks[key] = max(peaks.get(key, 0), row['armyHPValueBefore'], row['armyHPValueAfter'])
            row['armyPeakHPValue'] = peaks[key]
            row['armyFractionOfPeakAfter'] = row['armyHPValueAfter'] / peaks[key] if peaks[key] else None
            if row['ownDecision']:
                last_decision[key] = row
            if row['died']:
                previous = last_decision.get(key, {})
                deaths.append(dict(row, **{'lastDecision' + k[0].upper() + k[1:]: previous.get(k)
                    for k in ('round', 'beforeHP', 'armyUnitsBefore', 'armyHPValueBefore',
                              'armyValueRatioBefore', 'beforeDamage', 'chosenTileDamage',
                              'minDamage', 'safeTiles', 'forecastLethalWithSafeAlternative',
                              'afterHP', 'afterDamage', 'endForecastLethal')}))
        tables['leader_deaths'] = deaths
    return tables
