"""Readable campaign findings and complete arena comparisons from saved runs."""
import argparse
import csv
import json
import gzip
import hashlib
import importlib.util
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path
import time

DISPLAY_NAMES = {'coastal': 'Merfolk', 'open': 'Plains', 'wooded': 'Forests',
                 'crossings': 'Crossing', 'islands': 'Islands'}
MAP_ORDER = ('open', 'wooded', 'crossings', 'islands')


def display_name(value):
    return DISPLAY_NAMES.get(value, value)


def map_order(value):
    return (MAP_ORDER.index(value) if value in MAP_ORDER else len(MAP_ORDER), value)


def race_order(value):
    return (display_name(value) == 'Merfolk', display_name(value))


def tactic_label(config, side):
    blocks = config.get(side + 'Blocks')
    return '-'.join(blocks[k] for k in ('recruiter', 'campaign', 'planner')) if blocks else config.get(side, 'unrecorded')


def identity(outcome):
    return outcome['kind'] + ':' + json.dumps(outcome['config'], sort_keys=True, separators=(',', ':'))


def chapter_diagnostics(chapter):
    phases = chapter.get('phases', [])
    own = [p for p in phases if p['side'] == 'player']
    final = chapter.get('final', {})
    units = final.get('units', [])
    reason = (final.get('campaign') or {}).get('outcome')
    if chapter['winner'] is None and not reason:
        reason = 'Round limit reached with unfinished objectives'
    initial = chapter.get('initial', {}).get('units', [])
    named_actors = {u['name']: u for u in initial if u.get('name') and u['side'] == 'player'
                    and any(u['name'] in g.get('label', '') for g in chapter.get('goals', []))}
    arrivals = {name: any(u.get('name') == name and u['tile'] in g.get('tiles', [])
                         for p in phases for u in p.get('units', [])
                         for g in chapter.get('goals', []) if name in g.get('label', ''))
                for name in named_actors}
    stalled = []
    if chapter['winner'] is None and own and own[-1].get('planning'):
        last = own[-1]
        roster = {u['id']: u for u in last.get('units', [])}
        planning = last['planning']
        statuses = {d['id']: d for d in planning['report'].get('duties', [])}
        for order in planning['orders']['tactics'].get('objectives', []):
            body = order['objective'].get('GoTo') or order['objective'].get('Occupy')
            if not body:
                continue
            actor = roster.get(f"u{body['actor']}", {})
            stalled.append(dict(actor=actor.get('name') or actor.get('type') or f"u{body['actor']}",
                                tile=actor.get('tile'), hp=actor.get('hp'), target=body.get('destination', body.get('location')),
                                reason=statuses.get(order['id'], {}).get('reason'),
                                allies=[dict(id=u['id'], type=u['type'], tile=u['tile'], rebuilding=u.get('rebuilding')) for u in roster.values() if u['side']=='player']))
    losses = [u for p in phases for u in p.get('losses', []) if u['side'] == 'player']
    idle = 0
    for phase in reversed(own):
        if phase.get('actions'):
            break
        idle += 1
    signatures = [json.dumps([[(u['id'], u['type'], u['tile'], u['hp'])
                              for u in p.get('units', [])], p.get('ownership'), p.get('goals')],
                             sort_keys=True) for p in own[-10:]]
    repeated = max(Counter(signatures).values(), default=0)
    if chapter['winner'] == 'player':
        category = 'victory'
    elif chapter['winner'] is None:
        category = 'round cap'
    elif 'units' in final and not any(u['side'] == 'player' for u in units):
        category = 'all player units lost'
    elif any(u.get('role') == 'leader' for u in losses):
        category = 'leader lost'
    elif any(u.get('name') in named_actors or u.get('role') == 'courier' for u in losses) or (reason and any(word in reason.lower() for word in ('courier', 'companion'))):
        category = 'required actor lost'
    else:
        category = 'defeat; see recorded reason' if reason else 'defeat; cause not recorded'
    player_damage = enemy_damage = 0
    recruits, casualties = Counter(), Counter()
    unmet_orders, deferred_orders = Counter(), Counter()
    rescue_candidates = retaliation_violations = 0
    for phase in phases:
        roster = {u['id']: u for u in phase.get('before', []) + phase.get('units', []) + phase.get('losses', [])}
        for action in phase.get('actions', []):
            if action['kind'] == 'recruit' and phase['side'] == 'player':
                recruits[action['type']] += 1
        for unit in phase.get('losses', []):
            if unit['side'] == 'player':
                casualties[unit['type']] += 1
        for combat in phase.get('combats', []):
            attacker = roster.get(combat['unit'], {})
            if attacker.get('side', phase['side']) == 'player':
                player_damage += combat['damage']
                enemy_damage += combat['retaliation']
            else:
                enemy_damage += combat['damage']
                player_damage += combat['retaliation']
        if phase['side'] == 'player' and phase.get('planning'):
            planning = phase['planning']
            for duty in planning['report'].get('duties', []):
                if duty['status'] == 'Unmet':
                    unmet_orders[duty.get('reason') or 'unspecified'] += 1
                elif duty['status'] == 'Deferred':
                    deferred_orders[duty.get('reason') or 'unspecified'] += 1
            rescue_candidates += planning['report'].get('candidates', 0)
            protected = {f"u{c['Survive']['actor']}" for c in planning['orders']['tactics'].get('constraints', []) if 'Survive' in c}
            retaliation_violations += sum(c['unit'] in protected and c['retaliation'] > 0 for c in phase.get('combats', []))
    return dict(category=category, reason=reason, idleTail=idle, repeatedTail=repeated,
                finalPlayerUnits=sum(u['side'] == 'player' for u in units),
                finalEnemyUnits=sum(u['side'] == 'enemy' for u in units),
                maxGoalProgress=[max((p.get('goals', [])[i]['current'] for p in phases
                                     if len(p.get('goals', [])) > i), default=g.get('current', 0))
                                 for i, g in enumerate(chapter.get('goals', []))],
                goals=chapter.get('goals', []),
                namedLosses=dict(Counter(u['name'] for u in losses if u.get('name'))),
                recruits=dict(recruits), casualties=dict(casualties),
                playerDamage=player_damage, enemyDamage=enemy_damage,
                namedGoalActors=list(named_actors), actorArrivals=arrivals, stalledActors=stalled,
                recordedPhases=bool(phases), unmetOrders=dict(unmet_orders),
                deferredOrders=dict(deferred_orders), rescueCandidates=rescue_candidates,
                protectedRetaliationViolations=retaliation_violations,
                recordedPlanning=any(p.get('planning') for p in own))


def recorded_analysis(result):
    return [dict(id=m['id'], round=m['round'], winner=m['winner'],
                 **chapter_diagnostics(m)) for m in result.get('missions', [])]


def load_run(paths):
    outcomes, sources = {}, []
    for path in paths:
        if path.is_dir() and not (path / 'outcomes.json').exists():
            children = [path / 'report.json'] if (path / 'report.json').exists() else sorted(
                p for p in path.iterdir() if p.is_dir() and (p / 'outcomes.json').exists())
            if not children:
                raise ValueError('No recorded reports in ' + str(path))
            loaded, metadata = load_run(children)
            for key, item in loaded.items():
                if key in outcomes:
                    raise ValueError('Duplicate match configuration: ' + key)
                outcomes[key] = item
            sources.extend(metadata)
            continue
        diagnostics = {}
        if path.is_dir():
            report = {'outcomes': json.loads((path / 'outcomes.json').read_text()), 'mode': 'campaign'}
            metadata = path.parent / 'metadata.json'
            if metadata.exists():
                report.update(json.loads(metadata.read_text()))
            journal = path / 'details.jsonl.gz'
            if journal.exists():
                with gzip.open(journal, 'rt') as stream:
                    for line in stream:
                        raw = json.loads(line)
                        raw.setdefault('kind', 'campaign' if 'missions' in raw else 'arena')
                        diagnostics[identity(raw)] = recorded_analysis(raw)
        else:
            report = json.loads(path.read_text())
        source = {k: report.get(k) for k in ('sourceCommit', 'fingerprint', 'status', 'mode', 'schedule', 'seeds', 'elapsedSeconds', 'detailFormat', 'turnTiming', 'recruitmentLimits', 'unitFamilies', 'unitProfiles', 'supportTelemetryVersion', 'combatTelemetryVersion', 'leaderAuditVersion')}
        source['fingerprint'] = report.get('fingerprint', report.get('fingerprints'))
        source['file'] = str(path.resolve())
        source['matches'] = len(report['outcomes'])
        sources.append(source)
        readers = {}
        try:
            # Native workers append journals in completion order. Read references
            # in physical order so compressed baselines never repeatedly rewind.
            ordered = sorted(report['outcomes'], key=lambda item: (item.get('detail', {}).get('file', ''), item.get('detail', {}).get('offset', 0)))
            for saved in ordered:
                item = dict(saved)
                item.setdefault('kind', 'campaign' if 'campaign' in item['config'] else 'arena')
                key = identity(item)
                if key in outcomes:
                    raise ValueError('Duplicate match configuration: ' + key)
                if item['kind'] == 'arena':
                    if item.get('winner') not in (None, 'player', 'enemy'):
                        raise ValueError('Invalid winner: ' + key)
                    if (item.get('winner') is None) != (item.get('reason') == 'cap'):
                        raise ValueError('Winner/cap inconsistency: ' + key)
                else:
                    item['analysis'] = diagnostics.get(key)
                    reference = item.get('detail', {})
                    if item['analysis'] is None and all(k in reference for k in ('file', 'offset', 'bytes', 'sha256')):
                        name = reference['file']
                        if Path(name).name != name or reference['offset'] < 0 or reference['bytes'] <= 0:
                            raise ValueError('Invalid gameplay journal reference')
                        reader = readers.setdefault(name, None)
                        if reader is None:
                            journal = path.parent / name
                            reader = readers[name] = journal.open('rb') if journal.exists() else gzip.open(str(journal) + '.gz', 'rb')
                        reader.seek(reference['offset'])
                        raw = reader.read(reference['bytes'])
                        if len(raw) != reference['bytes'] or hashlib.sha256(raw).hexdigest() != reference['sha256']:
                            raise ValueError('Invalid gameplay journal checksum')
                        if name.endswith('.arrow'):
                            if importlib.util.find_spec('pyarrow') is None:
                                raise ModuleNotFoundError('pyarrow')
                            decoder_path = Path(__file__).resolve().parents[4] / 'scripts/simulation_journal.py'
                            spec = importlib.util.spec_from_file_location('simulation_journal', decoder_path)
                            decoder = importlib.util.module_from_spec(spec)
                            spec.loader.exec_module(decoder)
                            result = decoder.decode_detail(raw, name)
                        else:
                            result = json.loads(raw)
                        result.setdefault('kind', 'campaign')
                        if identity(result) != key:
                            raise ValueError('Gameplay journal configuration mismatch')
                        item['analysis'] = recorded_analysis(result)
                    if item['analysis'] is None:
                        item['analysis'] = recorded_analysis(item)
                outcomes[key] = item
            # Keep public tables in source order even though journal I/O is sorted.
            for saved in report['outcomes']:
                key = identity({**saved, 'kind': saved.get('kind', 'campaign' if 'campaign' in saved['config'] else 'arena')})
                outcomes[key] = outcomes.pop(key)
        finally:
            for reader in readers.values():
                if reader is not None:
                    reader.close()
    return outcomes, sources


def campaign_tables(outcomes):
    groups, chapters, reasons, units = {}, {}, Counter(), Counter()
    examples, runs = defaultdict(set), []
    for outcome in outcomes.values():
        if outcome['kind'] != 'campaign':
            continue
        config = outcome['config']
        fields = ('campaign', 'player', 'enemy', 'diplomacy', 'maxRounds', 'recruitLimit')
        key = tuple(tactic_label(config, k) if k in ('player', 'enemy') else config.get(k, 'unrecorded') for k in fields)
        group = groups.setdefault(key, Counter())
        group['attempts'] += 1
        group['completed'] += bool(outcome['completed'])
        missions = outcome.get('analysis') or recorded_analysis(outcome)
        if not missions:
            group['unclassified'] += not outcome['completed']
        elif not outcome['completed']:
            group['caps' if missions[-1]['winner'] is None else 'defeats' if missions[-1]['winner'] == 'enemy' else 'unclassified'] += 1
        for index, mission in enumerate(missions):
            chapter_key = (*key, index + 1, mission['id'])
            runs.append(dict(zip(fields, key), seed=config.get('seed'), stage=index+1, map=mission['id'],
                             winner=mission['winner'], round=mission['round'], category=mission['category'],
                             reason=mission['reason'], finalPlayerUnits=mission['finalPlayerUnits'],
                             idleTail=mission['idleTail'], repeatedTail=mission['repeatedTail'],
                             maxGoalProgress=mission['maxGoalProgress'], goals=mission['goals'],
                             namedLosses=mission['namedLosses'], actorArrivals=mission['actorArrivals'],
                             unmetOrders=mission.get('unmetOrders', {}), deferredOrders=mission.get('deferredOrders', {}),
                             rescueCandidates=mission.get('rescueCandidates', 0), protectedRetaliationViolations=mission.get('protectedRetaliationViolations', 0),
                             recordedPlanning=mission.get('recordedPlanning', False), stalledActors=mission.get('stalledActors', []),
                             detail=outcome.get('detail')))
            chapter = chapters.setdefault(chapter_key, Counter())
            chapter['reached'] += 1
            chapter['victories'] += mission['winner'] == 'player'
            chapter['defeats'] += mission['winner'] == 'enemy'
            chapter['caps'] += mission['winner'] is None
            chapter['rounds'] += mission['round']
            chapter['detailed'] += mission['recordedPhases']
            chapter['idleCaps'] += mission['winner'] is None and mission['idleTail'] >= 5
            chapter['repeatedCaps'] += mission['winner'] is None and mission['repeatedTail'] >= 5
            chapter['zeroProgressDefeats'] += mission['winner'] == 'enemy' and bool(mission['maxGoalProgress']) and all(x == 0 for x in mission['maxGoalProgress'])
            chapter['actorAttempts'] += bool(mission['namedGoalActors'])
            chapter['actorArrivals'] += bool(mission['namedGoalActors']) and all(mission['actorArrivals'].values())
            chapter['actorDefeatsBeforeArrival'] += mission['winner'] == 'enemy' and bool(mission['namedGoalActors']) and not all(mission['actorArrivals'].values())
            chapter['defeatsWithSurvivors'] += mission['winner'] == 'enemy' and mission['finalPlayerUnits'] > 0
            chapter['survivorsInDefeats'] += mission['finalPlayerUnits'] if mission['winner'] == 'enemy' else 0
            chapter['playerDamage'] += mission['playerDamage']
            chapter['enemyDamage'] += mission['enemyDamage']
            if mission['winner'] != 'player':
                reason_key = (*chapter_key, mission['category'], mission['reason'] or 'No terminal explanation recorded')
                reasons[reason_key] += 1
                if config.get('seed') is not None:
                    examples[reason_key].add(config['seed'])
            for event, values in [('recruited', mission['recruits']), ('lost', mission['casualties']), ('namedLost', mission['namedLosses'])]:
                for unit, count in values.items():
                    units[(*chapter_key, event, unit)] += count
    campaigns = []
    for key, counts in sorted(groups.items(), key=lambda item: repr(item[0])):
        campaigns.append(dict(zip(fields, key), **{k: counts[k] for k in ('attempts', 'completed', 'defeats', 'caps', 'unclassified')}, completionRate=100 * counts['completed'] / counts['attempts']))
    stages = []
    for key, counts in sorted(chapters.items(), key=lambda item: repr(item[0])):
        stages.append(dict(zip((*fields, 'stage', 'map'), key), **counts,
                           victoryRate=100 * counts['victories'] / counts['reached'],
                           campaignAttempts=groups[key[:6]]['attempts'],
                           meanRounds=counts['rounds'] / counts['reached']))
    failures = [dict(zip((*fields, 'stage', 'map', 'category', 'reason'), key), attempts=count, exampleSeeds=sorted(examples[key])[:3]) for key, count in sorted(reasons.items(), key=lambda item: repr(item[0]))]
    events = [dict(zip((*fields, 'stage', 'map', 'event', 'unit'), key), count=count) for key, count in sorted(units.items(), key=lambda item: repr(item[0]))]
    return campaigns, stages, failures, events, runs


def campaign_prose(campaigns, stages, failures, events):
    lines = ['## Campaign findings', '', 'Stage rates use attempts reaching that stage. Campaign rates include all caps and defeats. Diplomacy, enemy tactic and round limits remain separate.', '']
    fields = ('campaign', 'player', 'enemy', 'diplomacy', 'maxRounds', 'recruitLimit')
    for row in campaigns:
        same = lambda r: all(r[k] == row[k] for k in fields)
        maps = [r for r in stages if same(r)]
        lines += [f"### {row['campaign']}: {row['player']} against {row['enemy']}, {row['diplomacy']}, {row['maxRounds']} rounds", '',
                  f"Completed **{row['completed']}/{row['attempts']} ({row['completionRate']:.1f}%)**; {row['defeats']} defeats, {row['caps']} round caps, {row['unclassified']} unclassified endings.", '']
        if maps:
            bottleneck = max(maps, key=lambda r: r.get('defeats', 0) + r.get('caps', 0))
            lines.append(f"The largest recorded loss of attempts is **{bottleneck['map']}**: {bottleneck['victories']}/{bottleneck['reached']} stage victories, {bottleneck.get('defeats', 0)} defeats and {bottleneck.get('caps', 0)} caps.")
        for stage in maps:
            lines.append(f"**{stage['map']}**: {stage['victories']}/{stage['reached']} victories; {stage.get('defeats', 0)} defeats; {stage.get('caps', 0)} caps.")
            details = [r for r in failures if same(r) and r['map'] == stage['map']]
            if details:
                lines.append(f"- **{stage['map']}**: " + '; '.join(f"{r['attempts']} {r['category']}: {r['reason']}" for r in details) + '.')
            if stage.get('detailed'):
                lines.append(f"  Recorded damage dealt/taken: {stage.get('playerDamage', 0)}/{stage.get('enemyDamage', 0)}; {stage.get('idleCaps', 0)} caps ended with at least five empty player phases; {stage.get('repeatedCaps', 0)} repeated physical tail states. {stage.get('zeroProgressDefeats', 0)} defeats never recorded progress on any stage goal.")
                if stage.get('actorAttempts'):
                    lines.append(f"  Named goal actors reached their destination in {stage.get('actorArrivals', 0)}/{stage['actorAttempts']} attempts; {stage.get('actorDefeatsBeforeArrival', 0)} defeats occurred before arrival.")
                names = [r for r in events if same(r) and r['map'] == stage['map'] and r['event'] == 'namedLost']
                if names:
                    lines.append('  Named player casualties: ' + ', '.join(f"{r['unit']} {r['count']}" for r in names) + '.')
            else:
                lines.append('  Phase diagnostics unavailable in this input; no inactivity or combat explanation inferred.')
        lines.append('')
    return lines


def arena_race_map_summary(rows, opponent='classic'):
    selected = [r for r in rows if r['opponentPolicy'] == opponent
                and (r['policy'] != opponent or r['race'] != r['opponentRace'])]
    return combine(selected, ['race', 'map', 'policy'], cross=True)


def overview(campaigns, stages, failures, events, arena_rows, opponent=None):
    opponent = opponent or ('balanced' if any(r['opponentPolicy'] == 'balanced' for r in arena_rows) else 'classic')
    lines = ['# Recorded simulation findings', '']
    if arena_rows:
        lines += [f'## Arena wins by race and map against {opponent.title()}', '',
                  'Each cell shows wins/all games (win percentage), then round caps. Caps remain in the denominator. Opponent races, seats and initiatives are pooled; player policies stay separate. All same-faction games are excluded so every tactic faces the same opposing-faction pool. Detailed opponent and equal-policy comparisons follow.', '']
        summary = arena_race_map_summary(arena_rows, opponent)
        factions = sorted({r['race'] for r in arena_rows}, key=race_order)
        maps = sorted({r['map'] for r in arena_rows}, key=map_order)
        if not summary:
            lines += [f'No non-self-play games against {opponent.title()} were recorded; see the recorded matchups below.', '']
        for policy in sorted({r['policy'] for r in summary}):
            lookup = {(r['race'], r['map']): r for r in summary if r['policy'] == policy}
            rows = []
            for faction in factions:
                row = {'Race': faction}
                for map_name in maps:
                    item = lookup.get((faction, map_name))
                    row[map_name] = (f"{item['wins']}/{item['games']} ({item['winRate']:.1f}%); caps: {item['caps']}"
                                     if item else 'not recorded')
                rows.append(row)
            lines += [f'### {policy} against {opponent.title()}', '', markdown(rows, ['Race', *maps]), '']
    dimensions = sorted({(r['enemy'], r['maxRounds'], r['recruitLimit']) for r in campaigns}, key=repr)
    players = sorted({r['player'] for r in campaigns})
    for enemy, rounds, recruits in dimensions:
        recruitment = (f'{recruits} recruits per phase (explicit override)' if recruits is not None
                       else 'policy recruitment defaults: Classic 2 per phase; other tactics uncapped')
        selected = [r for r in campaigns if (r['enemy'], r['maxRounds'], r['recruitLimit']) == (enemy, rounds, recruits)]
        lines += [f'## Campaigns against {enemy}: {rounds} rounds, {recruitment}', '',
                  'Completion counts / all attempts. Campaigns with multiple player diplomacy modes have separate rows; modes are never pooled.', '']
        rows = []
        for campaign in sorted({r['campaign'] for r in selected}):
            modes = sorted({r.get('diplomacy', 'pragmatic') for r in selected if r['campaign'] == campaign})
            for mode in modes:
                label = f'{campaign} ({mode})' if len(modes) > 1 or mode != 'pragmatic' else campaign
                row = {'Campaign': label}
                for player in players:
                    values = [r for r in selected if r['campaign'] == campaign and r['player'] == player
                              and r.get('diplomacy', 'pragmatic') == mode]
                    n, wins = sum(r['attempts'] for r in values), sum(r['completed'] for r in values)
                    row[player] = f'{wins}/{n}' if n else 'not recorded'
                rows.append(row)
        lines += [markdown(rows, ['Campaign', *players]), '']
    if dimensions:
        lines += ['## Campaign stage outcomes and failure findings', '']
    for enemy, rounds, recruits in dimensions:
        recruitment = (f'{recruits} recruits per phase (explicit override)' if recruits is not None
                       else 'policy recruitment defaults: Classic 2 per phase; other tactics uncapped')
        selected = [r for r in campaigns if (r['enemy'], r['maxRounds'], r['recruitLimit']) == (enemy, rounds, recruits)]
        lines += [f'### Against {enemy}: {rounds} rounds, {recruitment}', '',
                  'Stage cells are victories/reached; D = defeats, C = round caps. Later-stage denominators exclude attempts stopped earlier.', '']
        rows = []
        selected_stages = [r for r in stages if (r['enemy'], r['maxRounds'], r['recruitLimit']) == (enemy, rounds, recruits)]
        for campaign, index, stage in sorted({(r['campaign'], r['stage'], r['map']) for r in selected_stages}):
            row = {'Stage': stage}
            for player in players:
                values = [r for r in selected_stages if r['map'] == stage and r['player'] == player]
                n, wins, deaths, caps = (sum(r.get(k, 0) for r in values) for k in ('reached', 'victories', 'defeats', 'caps'))
                row[player] = f'{wins}/{n}; {deaths}D {caps}C' if n else 'not reached'
            rows.append(row)
        lines += [markdown(rows, ['Stage', *players]), '']
        for campaign in sorted({r['campaign'] for r in selected}):
            lines += [f'### Why {campaign} attempts stop', '']
            for player in players:
                values = [r for r in selected_stages if r['campaign'] == campaign and r['player'] == player]
                if not values:
                    continue
                by_map = defaultdict(Counter)
                for r in values:
                    for field in ('reached', 'victories', 'defeats', 'caps', 'idleCaps', 'repeatedCaps', 'zeroProgressDefeats', 'actorAttempts', 'actorArrivals', 'actorDefeatsBeforeArrival', 'detailed', 'playerDamage', 'enemyDamage', 'defeatsWithSurvivors', 'survivorsInDefeats'):
                        by_map[r['map']][field] += r.get(field, 0)
                stage, count = max(by_map.items(), key=lambda p: p[1]['defeats']+p[1]['caps'])
                stopped = count['defeats']+count['caps']
                if not stopped:
                    lines.append(f'- **{player}**: no defeats or caps recorded.')
                    continue
                selected_failures = [r for r in failures if r['campaign']==campaign and r['player']==player and r['enemy']==enemy and r['map']==stage and r['maxRounds']==rounds and r['recruitLimit']==recruits]
                causes = Counter()
                for r in selected_failures:
                    causes[r['reason'] if r['category'] != 'round cap' else 'round cap'] += r['attempts']
                text = f"- **{player}**: largest bottleneck **{stage}**, {stopped}/{count['reached']} attempts stopped: " + '; '.join(f'{n} {reason.rstrip(".")}' for reason,n in causes.most_common()) + '.'
                if count['detailed']:
                    text += f" {count['idleCaps']} idle caps and {count['repeatedCaps']} repeated-state caps; {count['zeroProgressDefeats']} defeats with zero recorded goal progress."
                    if count['actorAttempts']:
                        text += f" Named goal actors arrived in {count['actorArrivals']}/{count['actorAttempts']} attempts."
                    text += f" {count['defeatsWithSurvivors']} defeats still had surviving player units."
                    text += f" Damage dealt/taken {count['playerDamage']}/{count['enemyDamage']}."
                else:
                    text += ' Full phase diagnostics unavailable; no deeper cause inferred.'
                lines.append(text)
                for other_stage, other_count in by_map.items():
                    if other_stage == stage or not other_count['defeats'] + other_count['caps']:
                        continue
                    other_causes = Counter()
                    for r in failures:
                        if r['campaign']==campaign and r['player']==player and r['enemy']==enemy and r['map']==other_stage and r['maxRounds']==rounds and r['recruitLimit']==recruits:
                            other_causes[r['reason'] if r['category'] != 'round cap' else 'round cap'] += r['attempts']
                    lines.append(f"  **{other_stage}** also stopped {other_count['defeats'] + other_count['caps']}/{other_count['reached']} attempts: " + '; '.join(f'{n} {reason.rstrip(".")}' for reason,n in other_causes.most_common()) + '.')
            lines.append('')
    if arena_rows:
        lines += ['## Arena results by tactic, race, opponent and map', '',
                  'Caps remain in denominators; self-play is excluded here. Each table names both tactics. Fixed-opponent and equal-tactic comparisons are shown; every recorded policy pairing remains in the complete report.', '']
        factions = sorted({r['race'] for r in arena_rows})
        for policy in sorted({r['policy'] for r in arena_rows}):
            for opponent_policy in dict.fromkeys([opponent, 'balanced', 'classic', policy]):
                for map_name in sorted({r['map'] for r in arena_rows}, key=map_order):
                    selected = [r for r in arena_rows if r['policy']==policy and r['opponentPolicy']==opponent_policy and r['map']==map_name and (r['race']!=r['opponentRace'] or policy!=opponent_policy)]
                    if not selected:
                        continue
                    lines += [f'### {policy} against {opponent_policy}: {display_name(map_name)}', '', matrix(selected, factions, map_name, policy, opponent_policy), '']
    return lines


def tally(outcomes):
    games, events, ledger, campaigns = defaultdict(Counter), defaultdict(Counter), [], []
    for match_id, o in outcomes.items():
        c = o['config']
        row = {'matchId': match_id, 'kind': o['kind'], **{k: v for k, v in c.items() if k != 'armies'},
               'winnerSeat': o.get('winner'), 'reason': o.get('reason'), 'rounds': o.get('rounds'),
               'stateHash': o.get('stateHash'), 'detail': o.get('detail'), 'recruits': o.get('recruits'),
               'promotions': o.get('promotions'), 'casualties': o.get('casualties')}
        row['combatUsage'] = o.get('combatUsage')
        if o['kind'] != 'arena':
            row['missions'] = o.get('missions')
            row['completed'] = o.get('completed')
            campaigns.append(row)
            ledger.append(row)
            continue
        row.update(playerRace=c['armies']['player'], enemyRace=c['armies']['enemy'],
                   race=c['armies']['player'], opponentRace=c['armies']['enemy'],
                   policy=tactic_label(c, 'player'), opponentPolicy=tactic_label(c, 'enemy'), map=c['landscape'])
        ledger.append(row)
        for side, other in [('player', 'enemy'), ('enemy', 'player')]:
            group = (c['armies'][side], c['armies'][other], tactic_label(c, side), tactic_label(c, other), c['landscape'])
            g = games[group]
            g['games'] += 1
            g['wins'] += o['winner'] == side
            g['losses'] += o['winner'] == other
            g['caps'] += o['winner'] is None
            g['leaderWins'] += o['winner'] == side and o['reason'] == 'leader'
            g['territoryWins'] += o['winner'] == side and o['reason'] == 'territory'
            g['rounds'] += o['rounds']
            for label, field in [('recruited', 'recruits'), ('promoted', 'promotions'), ('lost', 'casualties')]:
                for unit in o.get(field, []):
                    if unit['side'] == side:
                        events[group + (unit['type'],)][label] += 1
            for unit in o.get('combatUsage', []):
                if unit['side'] == side:
                    for field in ('attacks', 'retaliations', 'damage'):
                        events[group + (unit['type'],)][field] += unit[field]
    rows = []
    for group, g in sorted(games.items()):
        row = dict(zip(('race', 'opponentRace', 'policy', 'opponentPolicy', 'map'), group))
        row.update(g)
        row['winRate'] = 100 * g['wins'] / g['games']
        row['lossRate'] = 100 * g['losses'] / g['games']
        row['capRate'] = 100 * g['caps'] / g['games']
        row['meanRounds'] = g['rounds'] / g['games']
        assert g['games'] == g['wins'] + g['losses'] + g['caps']
        rows.append(row)
    units = []
    for group, counts in sorted(events.items()):
        row = dict(zip(('race', 'opponentRace', 'policy', 'opponentPolicy', 'map', 'unit'), group))
        row.update({k: counts[k] for k in ('recruited', 'promoted', 'lost', 'attacks', 'retaliations', 'damage')})
        units.append(row)
    return rows, units, ledger, campaigns


def combine(rows, keys, equal=False, cross=True):
    groups = defaultdict(Counter)
    for row in rows:
        if equal and row['policy'] != row['opponentPolicy']:
            continue
        if cross and row['race'] == row['opponentRace']:
            continue
        g = groups[tuple(row[k] for k in keys)]
        for k in ('games', 'wins', 'losses', 'caps', 'leaderWins', 'territoryWins', 'rounds'):
            g[k] += row[k]
    output = []
    for group, counts in sorted(groups.items(), key=lambda item: repr(item[0])):
        output.append({**dict(zip(keys, group)), **counts,
                       'winRate': 100 * counts['wins'] / counts['games']})
    return output


def purchase_shares(events, families):
    """New purchases only; exclude starting units, promotions and same-faction games.

    Opposing factions are pooled; tactic, opponent tactic, faction and map stay
    separate. An additional 'all' map row pools maps with purchase-count weights.
    Older reports without recorded families retain individual unit identifiers.
    """
    groups = defaultdict(Counter)
    for event in events:
        if event['race'] == event['opponentRace']:
            continue
        for map_name in (event['map'], 'all'):
            key = (event['race'], event['policy'], event['opponentPolicy'], map_name)
            groups[key][families.get(event['unit'], event['unit'])] += event['recruited']
    result = []
    for key, purchases in sorted(groups.items()):
        total = sum(purchases.values())
        if total:
            for family, count in sorted(purchases.items()):
                if count:
                    result.append(dict(zip(('race','policy','opponentPolicy','map'), key),
                                       family=family, purchased=count, totalPurchases=total,
                                       sharePercent=100*count/total))
    return result


def write_csv(path, rows):
    fields = list(dict.fromkeys(k for r in rows for k in r))
    with path.open('w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        for r in rows:
            writer.writerow({k: json.dumps(v, separators=(',', ':')) if isinstance(v, (dict, list)) else v for k, v in r.items()})


def markdown(rows, fields):
    race_field = next((field for field in ('race', 'Race', 'Faction') if field in fields), None)
    if race_field:
        rows = sorted(rows, key=lambda row: race_order(row.get(race_field, '')))
    def cell(v, field):
        if field == 'recruitLimit' and v is None:
            return 'policy default'
        if isinstance(v, str) and field in ('race', 'opponentRace', 'Race', 'Faction', 'map'):
            v = display_name(v)
        if isinstance(v, float):
            return f'{v:.1f}'
        if isinstance(v, (dict, list)):
            v = json.dumps(v)
        return str(v).replace('|', '\\|').replace('\n', ' ')
    return '\n'.join(['| ' + ' | '.join(display_name(field) for field in fields) + ' |', '| ' + ' | '.join('---' for _ in fields) + ' |',
                      *['| ' + ' | '.join(cell(r.get(k, ''), k) for k in fields) + ' |' for r in rows]])


def matrix(rows, factions, map_name, policy, opponent_policy):
    lookup = {(r['race'], r['opponentRace']): r for r in rows if r['map'] == map_name and r['policy'] == policy and r['opponentPolicy'] == opponent_policy}
    result = []
    for race in factions:
        row = {'Faction': race}
        for opponent in factions:
            r = lookup.get((race, opponent))
            row[opponent] = f"{r['winRate']:.1f}% ({r['wins']}/{r['losses']}/{r['caps']})" if r else 'NO RECORDED MATCHES'
        result.append(row)
    return markdown(result, ['Faction', *factions])


def compare_match(key, before, current):
    if current['kind'] == 'campaign':
        def result(o):
            return o['completed'], [(m['id'], m['winner']) for m in o['missions']]
        def gameplay(o):
            return o['completed'], [(m['id'], m['round'], m['stateHash']) for m in o['missions']]
        return {'matchId': key, 'kind': 'campaign',
                'beforeCompleted': before['completed'], 'completed': current['completed'],
                'beforeMissions': before['missions'], 'missions': current['missions'],
                'resultChanged': result(before) != result(current),
                'gameplayChanged': gameplay(before) != gameplay(current)}
    return {'matchId': key, 'kind': 'arena',
            'beforeWinner': before.get('winner'), 'winner': current.get('winner'),
            'beforeReason': before.get('reason'), 'reason': current.get('reason'),
            'beforeRounds': before.get('rounds'), 'rounds': current.get('rounds'),
            'resultChanged': (before.get('winner'), before.get('reason')) != (current.get('winner'), current.get('reason')),
            'gameplayChanged': before.get('stateHash') != current.get('stateHash')}


def build_report(inputs, baseline_inputs, out, unit_usage=False, support_effects=False, battlefield_presence=False, battlefield_value=False, leader_audit=False):
    start = time.perf_counter()
    current, sources = load_run(inputs)
    previous, previous_sources = load_run(baseline_inputs) if baseline_inputs else ({}, [])
    if baseline_inputs and current.keys() != previous.keys():
        raise ValueError(f'Unmatched configurations: {len(current.keys() - previous.keys())} current only, {len(previous.keys() - current.keys())} baseline only')
    rows, units, ledger, campaigns = tally(current)
    before, before_units, _, _ = tally(previous)
    factions = sorted(set(r['race'] for r in rows), key=race_order)
    maps = sorted(set(r['map'] for r in rows), key=map_order)
    policies = sorted(set(r['policy'] for r in rows))
    old_index = {(r['race'], r['opponentRace'], r['policy'], r['opponentPolicy'], r['map']): r for r in before}
    comparison = []
    for row in rows:
        key = tuple(row[k] for k in ('race', 'opponentRace', 'policy', 'opponentPolicy', 'map'))
        old = old_index.get(key)
        if old:
            comparison.append({**row, **{k + 'Before': old[k] for k in ('wins', 'losses', 'caps', 'winRate')}, 'winRateDelta': row['winRate'] - old['winRate']})
    transitions = []
    for key, o in current.items():
        if previous:
            transitions.append(compare_match(key, previous[key], o))
    campaign_rows, stage_rows, failure_rows, campaign_events, campaign_runs = campaign_tables(current)
    baseline_campaigns, _, _, _, _ = campaign_tables(previous)
    campaign_fields = ('campaign', 'player', 'enemy', 'diplomacy', 'maxRounds', 'recruitLimit')
    old_campaigns = {tuple(row[k] for k in campaign_fields): row for row in baseline_campaigns}
    campaign_changes = []
    for row in campaign_rows:
        old = old_campaigns.get(tuple(row[k] for k in campaign_fields))
        if old:
            campaign_changes.append({**row, 'completedBefore': old['completed'], 'completionRateBefore': old['completionRate'], 'completionRateDelta': row['completionRate'] - old['completionRate']})
    policy_timings = []
    for label, runs in [('current', sources), ('baseline', previous_sources)]:
        for source in runs:
            timing = source.get('turnTiming') or {}
            policy_timings.extend(dict(run=label, clock=timing.get('clock'), source=source['file'], **row) for row in timing.get('policies', []))
    families = {}
    for source in sources:
        for unit, family in (source.get('unitFamilies') or {}).items():
            if unit in families and families[unit] != family:
                raise ValueError('Conflicting recorded unit family: ' + unit)
            families[unit] = family
    opponents = {s.get('schedule', {}).get('opponent') for s in sources if s.get('schedule')}
    opponent = next(iter(opponents)) if len(opponents) == 1 else None
    tables = {'campaign_runs': campaign_runs, 'campaign_summary': campaign_rows, 'campaign_stages': stage_rows,
              'campaign_failures': failure_rows, 'campaign_unit_events': campaign_events, 'matchups': rows, 'unit_events': units, 'matches': ledger, 'campaigns': campaigns,
              'policy_matchups': combine([r for r in rows if r['policy'] != r['opponentPolicy']],
                                         ['policy', 'opponentPolicy'], cross=False),
              'race_map_vs_classic': arena_race_map_summary(rows),
              'race_map_vs_balanced': arena_race_map_summary(rows, 'balanced'),
              'purchase_shares': purchase_shares(units, families),
              'race_equal_policy': combine(rows, ['race'], equal=True),
              'race_map_equal_policy': combine(rows, ['race', 'map'], equal=True),
              'race_map_policy': combine(rows, ['race', 'map', 'policy'], equal=True),
              'race_all_policies': combine(rows, ['race']), 'baseline_matchups': before,
              'matchup_changes': comparison, 'match_changes': transitions, 'baseline_unit_events': before_units,
              'campaign_changes': campaign_changes, 'policy_timings': policy_timings}
    if unit_usage or support_effects or battlefield_presence or battlefield_value or leader_audit:
        if unit_usage and any(s.get('combatTelemetryVersion') != 1 for s in sources + previous_sources):
            raise ValueError('Damage tables require fresh runs with combat telemetry, including the baseline')
        spec = importlib.util.spec_from_file_location('unit_usage', Path(__file__).with_name('unit_usage.py'))
        usage = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(usage)
        if unit_usage or support_effects or battlefield_presence:
            extra, measured = usage.analyze(inputs, presence=unit_usage or battlefield_presence, support=unit_usage or support_effects)
            tables.update(extra)
        if unit_usage or battlefield_presence:
            nonleaders = [e for e in units if not any((source.get('unitProfiles') or {}).get(e['unit'], {}).get('leader', False) for source in sources)]
            tables['army_usage'] = usage.army_usage(nonleaders, tables['purchase_shares'], tables['battlefield_presence'], families)
        value_enabled = battlefield_value or (unit_usage and all(s.get('combatTelemetryVersion') == 1 for s in sources))
        leaders_enabled = leader_audit or (unit_usage and all(s.get('leaderAuditVersion') == 1 for s in sources))
        if value_enabled or leaders_enabled:
            spec = importlib.util.spec_from_file_location('battle_value', Path(__file__).with_name('battle_value.py'))
            value = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(value)
            tables.update(value.analyze(inputs, usage.journals, value_enabled, leaders_enabled))
        if unit_usage:
            tables['unit_damage'] = usage.unit_damage(tables['battlefield_units'],
                tables['battlefield_unit_value'], sources[0]['unitProfiles'])
            if baseline_inputs:
                old_presence, _ = usage.analyze(baseline_inputs, presence=True, support=False)
                old_value = value.analyze(baseline_inputs, usage.journals, value=True, leaders=False)
                tables['baseline_unit_damage'] = usage.unit_damage(old_presence['battlefield_units'],
                    old_value['battlefield_unit_value'], previous_sources[0]['unitProfiles'])
                tables['unit_damage_changes'] = usage.compare_unit_damage(
                    tables['unit_damage'], tables['baseline_unit_damage'])
    out.mkdir(parents=True, exist_ok=True)
    for name, table in tables.items():
        write_csv(out / (name + '.csv'), table)
    metadata = {'sources': sources, 'baselineSources': previous_sources,
                'physicalMatches': len(current), 'arenaMatches': sum(o['kind'] == 'arena' for o in current.values()),
                'campaignAttempts': len(campaigns), 'factions': factions, 'maps': sorted(set(maps) | {r['map'] for r in stage_rows}), 'policies': sorted(set(policies) | {r['player'] for r in campaign_rows}),
                'participantRows': sum(r['games'] for r in rows)}
    (out / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
    findings = overview(campaign_rows, stage_rows, failure_rows, campaign_events, rows, opponent)
    findings[2:2] = [f"Coverage: {len(current):,} physical matches; {metadata['arenaMatches']:,} arena games and {metadata['campaignAttempts']:,} campaign attempts.", '']
    if 'unit_damage' in tables:
        findings += ['## Damage tables', '',
            'Presence is each exact unit/tier’s share of living nonleader unit-turns. Damage means actual weapon HP removed, including retaliation; overkill and healing are excluded. Effective personal/aura thorns are credited to their source and shown separately, excluded from weapon damage shares and D/P. Leaders are excluded from both shares. Damage/turn divides damage by that unit’s unit-turns. D/P = damage share / presence share: 10% presence and 50% damage is 5.00×. The faction average uses summed damage and unit-turns, not an average of unit rates. Unobserved units have 0% presence and undefined rates (—); a faction with no damage has undefined D/P. Small samples are not reliable rankings.', '']
        damage_rows = tables.get('unit_damage_changes', tables['unit_damage'])
        if 'unit_damage_changes' in tables:
            findings += ['Comparison cells show baseline → current. Scopes match the report configuration; all-map rows sum counts before calculating shares and rates.', '']
        for p in policies:
            for q in sorted({r['opponentPolicy'] for r in damage_rows if r['policy'] == p}):
                for race in factions:
                    selected = [r for r in damage_rows if r['policy'] == p and r['opponentPolicy'] == q
                                and r['race'] == race and r['map'] == 'all']
                    if selected:
                        findings += [f'### {display_name(race)}: {p} against {q}, all maps', '',
                            usage.damage_markdown(selected, baseline='unit_damage_changes' in tables), '']
    if 'army_usage' in tables:
        findings += ['## Battlefield presence and recruitment flow', '',
                     'Lead composition analysis with field share and actual contributions. Purchases are recruitment flow, including replacement churn; compare them with field presence and casualties, not as army composition or strength. Neither purchase counts nor purchase/presence ratios establish survival without cohort evidence. Field share counts living non-leader units at the end of each own phase, including starting units and promotions. Longer-lived units contribute more unit-turns. Attack and damage shares exclude leaders; damage includes recorded retaliation and overkill. Opposing factions are pooled, same-faction games excluded. For opponent-specific composition, use a restricted two-army run or equivalent scoped census; do not pair these pooled shares with exact-opponent purchases. All-map shares use summed counts.', '',
                     markdown([r for r in tables['army_usage'] if r['map'] != 'all'], ['race','policy','opponentPolicy','map','family','sharePercent','unitTurns','purchaseSharePercent','recruited','lost','attackSharePercent','damageSharePercent']), '']
    if 'support_effects' in tables and tables['support_effects']:
        findings += ['## Actual support effects', '',
                     'Provider turns include zero-benefit turns. Healing adjacency is sampled at own-turn start; thorn adjacency at opposing-turn start. Healing and reflection are engine-recorded events. Tied strongest providers split credit. Effective reflection excludes overkill and counts only HP removed beyond ordinary retaliation. Intrinsic thorns and auras are separate. All-unit rows aggregate providers, not extra events.', '',
                     markdown([r for r in tables['support_effects'] if r['unit'] == 'all' and r['map'] != 'all'], ['race','policy','opponentPolicy','map','ability','providerTurns','adjacentAlliesPerTurn','healedHP','healedHPPerTurn','reflectedDamage','effectiveReflectedDamage','reflectedDamagePerTurn','overlapPercent']), '']
    if tables.get('battlefield_value'):
        findings += ['## Protection, objectives and casualty value', '',
                     'Protection is signed and capped at HP available per hit; armour is attributed before terrain. Objective control credits the last captor, including after death; occupation credits the living occupant. Recruit gold lost covers purchases in this arena/chapter; replacement value includes free starters and earned promotions. Total HP damage includes effective thorn reflection attributed to intrinsic/aura sources, with weapon and thorn subtotals; drain healing is separate.', '',
                     markdown([r for r in tables['battlefield_value'] if r['map'] == 'all' and r['kind'] == 'arena'],
                              ['race','policy','opponentPolicy','family','unitTurns','hpDamageDealt','hpDamageTaken','armourDamagePrevented','terrainDamagePrevented','captures','objectiveOccupationTurns','casualties','recruitGoldLost','casualtyReplacementValue']), '']
    if 'army_usage' not in tables:
        findings += ['**Field composition unavailable:** this report has no battlefield presence. Purchase flow includes replacement churn; do not infer field composition, survival or unit strength from purchases. For balance investigations, regenerate with `--unit-usage` or battlefield presence and relevant value telemetry.', '']
    findings += ['## Purchase shares', '', 'New purchases / all new purchases for the same faction, tactic, opponent tactic and map. Starting units, promotions and same-faction games are excluded. Opposing factions are pooled. The all-map rows sum purchases across maps; they do not average map percentages. Families come from the recorded catalog; older reports without family metadata use individual unit identifiers.', '', markdown([r for r in tables['purchase_shares'] if r['map'] == 'all'], ['race','policy','opponentPolicy','family','purchased','totalPurchases','sharePercent']), '']
    if 'leader_deaths' in tables:
        deaths = tables['leader_deaths']
        findings += ['## Leader death evidence', '',
                     f'{len(deaths)} recorded leader deaths. See leader_deaths.csv for army strength at death and the last own-phase decision. Forecasts sum independently reachable enemy attacks, holding other units fixed; a forecast-safe tile is not proof of a mission-compatible escape or an avoidable death.', '',
                     markdown([r for r in deaths if r['kind'] == 'campaign' and r['side'] == 'player'],
                              ['campaign','mission','policy','seed','round','armyUnitsAfter','armyHPValueAfter','lastDecisionArmyValueRatioBefore','lastDecisionForecastLethalWithSafeAlternative','deathDuringOwnAttack']), '']
    if tables['policy_matchups']:
        matchups = tables['policy_matchups']
        lookup = {(r['policy'], r['opponentPolicy']): r for r in matchups}
        matrix_rows = []
        for policy in policies:
            row = {'Tactic': policy}
            for opponent in policies:
                item = lookup.get((policy, opponent))
                row[opponent] = ('—' if policy == opponent else
                                 f"{item['winRate']:.1f}% (caps: {item['caps']})" if item else 'not recorded')
            matrix_rows.append(row)
        findings += ['## Tactic versus tactic arena matrix', '',
                     'Row tactic win percentages. All races, maps, seats and initiatives are included; caps stay in denominators. Same-tactic games are excluded.', '',
                     markdown(matrix_rows, ['Tactic', *policies]), '']
    if campaign_changes:
        findings += ['## Campaign completion versus baseline', '', markdown(campaign_changes, ['campaign', 'player', 'diplomacy', 'attempts', 'completedBefore', 'completed', 'completionRateDelta']), '']
    if policy_timings:
        findings += ['## Chooser and phase CPU', '',
                     'Each turn is one side’s completed phase. These policy averages pool arenas and campaigns; phase CPU excludes turn-transition events and journal encoding. Runs can contain different numbers of turns.', '',
                     markdown(policy_timings, ['run', 'policy', 'turns', 'meanDecisionMicrosPerTurn', 'meanTurnMicros']), '']
    stalled_examples = []
    seen_stalls = set()
    for run in campaign_runs:
        key = (run['campaign'], run['player'], run['map'])
        if run['idleTail'] < 5 or key in seen_stalls or not run['stalledActors']:
            continue
        seen_stalls.add(key)
        for actor in run['stalledActors']:
            allies = ', '.join(f"{u['id']} {u['type']} at {u['tile']}" + (' (rebuilding)' if u['rebuilding'] else '') for u in actor['allies'])
            stalled_examples.append(dict(campaign=run['campaign'], player=run['player'], map=run['map'], seed=run['seed'], actor=actor['actor'], tile=actor['tile'], target=actor['target'], reason=actor['reason'], allies=allies))
    if stalled_examples:
        findings += ['## Idle cap actor positions', '', 'One recorded example per campaign, tactic and stage. Positions come from the last player phase; these observations identify a stall, without assuming its cause.', '', markdown(stalled_examples, ['campaign', 'player', 'map', 'seed', 'actor', 'tile', 'target', 'reason', 'allies']), '']
    diagnostics = Counter()
    for outcome in current.values():
        for chapter in outcome.get('analysis') or []:
            diagnostics.update(chapter.get('unmetOrders', {}))
            diagnostics['protected retaliation violations'] += chapter.get('protectedRetaliationViolations', 0)
    if any(any(chapter.get('recordedPlanning') for chapter in outcome.get('analysis') or []) for outcome in current.values()):
        findings += ['## Recorded planner diagnostics', '', 'Counts are duty observations per phase, not independent failed attempts. UnsafeFallback means the forecast constraint remained unmet; it does not prove unavoidable death.', '',
                     markdown([dict(reason=reason, observations=count) for reason, count in sorted(diagnostics.items())], ['reason', 'observations']), '']
    lines = findings + ['# Complete combat data', '', f"Physical matches: **{len(current):,}**. Recorded arena factions: **{', '.join(display_name(f) for f in factions)}**.", '',
             'Each matrix cell shows **win percentage (wins/losses/caps)** for the row faction. Caps stay in the denominator. Each arena match contributes one result to each participant; mirrored cells are views of the same matches, not additional independent games. Self-play cells contain both participants. Policy pairings are kept separate.', '',
             '## Source runs', '', markdown(sources, ['sourceCommit', 'fingerprint', 'status', 'mode', 'matches', 'seeds', 'schedule']), '',
             '## Factions with matching policies', '', markdown(tables['race_equal_policy'], ['race', 'games', 'wins', 'losses', 'caps', 'winRate']), '',
             '## Factions by map with each policy kept separate', '',
             'These totals compare each faction with the other factions using the same policy, excluding self-play.', '',
             markdown(tables['race_map_policy'], ['race', 'map', 'policy', 'games', 'wins', 'losses', 'caps', 'winRate']), '',
             '## Every faction, map and policy pairing', '']
    for m in maps:
        for p in policies:
            for q in policies:
                lines += [f'### {display_name(m)}: row policy {p}, opponent policy {q}', '', matrix(rows, factions, m, p, q), '']
    lines += campaign_prose(campaign_rows, stage_rows, failure_rows, campaign_events)
    lines += ['## Campaign completion by tactic and opponent', '', markdown(campaign_rows, ['campaign', 'player', 'enemy', 'diplomacy', 'maxRounds', 'attempts', 'completed', 'completionRate', 'defeats', 'caps']), '',
              '## Every campaign stage', '', markdown(stage_rows, ['campaign', 'player', 'enemy', 'diplomacy', 'maxRounds', 'stage', 'map', 'reached', 'victories', 'victoryRate', 'defeats', 'caps']), '']
    if previous:
        lines += ['## Every matchup change versus baseline', '', markdown(comparison, ['race', 'opponentRace', 'policy', 'opponentPolicy', 'map', 'games', 'winRateBefore', 'winRate', 'winRateDelta', 'wins', 'losses', 'caps']), '']
    lines += ['## Unit recruitment, promotions, casualties and combat use', '',
              'Attacks, retaliations and damage come from resolved combat records, including units that died. Older reports without combat-use summaries have no recorded combat-use evidence.', '',
              markdown(units, ['race', 'opponentRace', 'policy', 'opponentPolicy', 'map', 'unit', 'recruited', 'promoted', 'lost', 'attacks', 'retaliations', 'damage']), '',
              '## Complete exports', '', *[f'- [{name}.csv]({name}.csv)' for name in tables], '',
              'The matches export retains every physical match configuration, result, recruitment, promotion, casualty and detail-journal reference. Original gameplay journals remain beside the input reports. This program reports saved data and runs no battles.', '']
    (out / 'findings.md').write_text('\n'.join(findings) + '\n')
    (out / 'summary.md').write_text('\n'.join(lines))
    elapsed = time.perf_counter() - start
    (out / 'timings.json').write_text(json.dumps({'reportSeconds': elapsed, 'physicalMatches': len(current)}, indent=2) + '\n')
    print('\n'.join(findings))
    print(f'{len(current):,} physical matches; {len(factions)} recorded factions; {len(rows)} matchup rows; {len(units)} unit-event rows; {elapsed:.2f}s. {out / "summary.md"}')
    return tables


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('inputs', nargs='+', type=Path)
    parser.add_argument('--baseline', action='append', default=[], type=Path)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--unit-usage', action='store_true', help='Analyze per-unit damage/presence ratios, battlefield presence and actual support effects')
    parser.add_argument('--support-effects', action='store_true', help='Analyze support adjacency, healing and reflected damage')
    parser.add_argument('--battlefield-presence', action='store_true', help='Analyze unit-turn army shares and actual combat use')
    parser.add_argument('--battlefield-value', action='store_true', help='Analyze recorded mitigation, captures, occupation and casualty gold')
    parser.add_argument('--leader-audit', action='store_true', help='Analyze leader exposure, army strength and leader deaths')
    args = parser.parse_args()
    try:
        build_report(args.inputs, args.baseline, args.out, args.unit_usage, args.support_effects, args.battlefield_presence, args.battlefield_value, args.leader_audit)
    except ModuleNotFoundError as error:
        if error.name == 'pyarrow' or str(error) == 'pyarrow':
            print('Arrow campaign diagnostics require PyArrow; loading it through uv.', flush=True)
            try:
                raise SystemExit(subprocess.call(['uv', 'run', '--no-project', '--with', 'pyarrow', 'python',
                                                  str(Path(__file__).resolve()), *sys.argv[1:]]))
            except FileNotFoundError:
                parser.exit(1, 'Arrow diagnostics require pyarrow or uv. Install pyarrow, or use JSONL journals.\n')
        raise
    except (ValueError, KeyError, OSError) as error:
        parser.exit(1, f'Cannot report combat data: {error}\n')
