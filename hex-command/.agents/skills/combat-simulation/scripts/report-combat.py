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
import zipfile


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
                namedGoalActors=list(named_actors), actorArrivals=arrivals,
                recordedPhases=bool(phases))


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
        source = {k: report.get(k) for k in ('sourceCommit', 'fingerprint', 'status', 'mode', 'schedule', 'seeds', 'elapsedSeconds', 'detailFormat')}
        source['fingerprint'] = report.get('fingerprint', report.get('fingerprints'))
        source['file'] = str(path.resolve())
        source['matches'] = len(report['outcomes'])
        sources.append(source)
        readers = {}
        try:
            for saved in report['outcomes']:
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
                            reader = readers[name] = (path.parent / name).open('rb')
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
        key = tuple(config.get(k, 'unrecorded') for k in fields)
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
    for key, counts in sorted(groups.items()):
        campaigns.append(dict(zip(fields, key), **{k: counts[k] for k in ('attempts', 'completed', 'defeats', 'caps', 'unclassified')}, completionRate=100 * counts['completed'] / counts['attempts']))
    stages = []
    for key, counts in sorted(chapters.items()):
        stages.append(dict(zip((*fields, 'stage', 'map'), key), **counts,
                           victoryRate=100 * counts['victories'] / counts['reached'],
                           campaignAttempts=groups[key[:6]]['attempts'],
                           meanRounds=counts['rounds'] / counts['reached']))
    failures = [dict(zip((*fields, 'stage', 'map', 'category', 'reason'), key), attempts=count, exampleSeeds=sorted(examples[key])[:3]) for key, count in sorted(reasons.items())]
    events = [dict(zip((*fields, 'stage', 'map', 'event', 'unit'), key), count=count) for key, count in sorted(units.items())]
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


def overview(campaigns, stages, failures, events, arena_rows):
    lines = ['# Recorded simulation findings', '']
    dimensions = sorted({(r['enemy'], r['maxRounds'], r['recruitLimit']) for r in campaigns})
    players = sorted({r['player'] for r in campaigns})
    for enemy, rounds, recruits in dimensions:
        selected = [r for r in campaigns if (r['enemy'], r['maxRounds'], r['recruitLimit']) == (enemy, rounds, recruits)]
        lines += [f'## Campaigns against {enemy}: {rounds} rounds, {recruits} recruits per phase', '',
                  'Completion counts / all attempts, with percentages. Recorded diplomacy modes are pooled here; detailed tables keep them separate.', '']
        rows = []
        for campaign in sorted({r['campaign'] for r in selected}):
            row = {'Campaign': campaign}
            for player in players:
                values = [r for r in selected if r['campaign'] == campaign and r['player'] == player]
                n, wins = sum(r['attempts'] for r in values), sum(r['completed'] for r in values)
                row[player] = f'{wins}/{n} ({100*wins/n:.1f}%)' if n else 'not recorded'
            rows.append(row)
        lines += [markdown(rows, ['Campaign', *players]), '',
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
                  'Caps remain in denominators; self-play is excluded here. Each table names both tactics. Fixed-Classic and equal-tactic comparisons are shown; every recorded policy pairing remains in the complete report.', '']
        factions = sorted({r['race'] for r in arena_rows})
        for policy in sorted({r['policy'] for r in arena_rows}):
            for opponent_policy in dict.fromkeys(['classic', policy]):
                for map_name in sorted({r['map'] for r in arena_rows}):
                    selected = [r for r in arena_rows if r['policy']==policy and r['opponentPolicy']==opponent_policy and r['map']==map_name and (r['race']!=r['opponentRace'] or policy!=opponent_policy)]
                    if not selected:
                        continue
                    lines += [f'### {policy} against {opponent_policy}: {map_name}', '', matrix(selected, factions, map_name, policy, opponent_policy), '']
    return lines


def tally(outcomes):
    games, events, ledger, campaigns = defaultdict(Counter), defaultdict(Counter), [], []
    for match_id, o in outcomes.items():
        c = o['config']
        row = {'matchId': match_id, 'kind': o['kind'], **{k: v for k, v in c.items() if k != 'armies'},
               'winnerSeat': o.get('winner'), 'reason': o.get('reason'), 'rounds': o.get('rounds'),
               'stateHash': o.get('stateHash'), 'detail': o.get('detail'), 'recruits': o.get('recruits'),
               'promotions': o.get('promotions'), 'casualties': o.get('casualties')}
        if o['kind'] != 'arena':
            row['missions'] = o.get('missions')
            row['completed'] = o.get('completed')
            campaigns.append(row)
            ledger.append(row)
            continue
        row.update(playerRace=c['armies']['player'], enemyRace=c['armies']['enemy'],
                   race=c['armies']['player'], opponentRace=c['armies']['enemy'],
                   policy=c['player'], opponentPolicy=c['enemy'], map=c['landscape'])
        ledger.append(row)
        for side, other in [('player', 'enemy'), ('enemy', 'player')]:
            group = (c['armies'][side], c['armies'][other], c[side], c[other], c['landscape'])
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
        row.update({k: counts[k] for k in ('recruited', 'promoted', 'lost')})
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
    for group, counts in sorted(groups.items()):
        output.append({**dict(zip(keys, group)), **counts,
                       'winRate': 100 * counts['wins'] / counts['games']})
    return output


def write_csv(path, rows):
    fields = list(dict.fromkeys(k for r in rows for k in r))
    with path.open('w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        for r in rows:
            writer.writerow({k: json.dumps(v, separators=(',', ':')) if isinstance(v, (dict, list)) else v for k, v in r.items()})


def markdown(rows, fields):
    def cell(v):
        if isinstance(v, float):
            return f'{v:.1f}'
        if isinstance(v, (dict, list)):
            v = json.dumps(v)
        return str(v).replace('|', '\\|').replace('\n', ' ')
    return '\n'.join(['| ' + ' | '.join(fields) + ' |', '| ' + ' | '.join('---' for _ in fields) + ' |',
                      *['| ' + ' | '.join(cell(r.get(k, '')) for k in fields) + ' |' for r in rows]])


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


def build_report(inputs, baseline_inputs, out):
    start = time.perf_counter()
    current, sources = load_run(inputs)
    previous, previous_sources = load_run(baseline_inputs) if baseline_inputs else ({}, [])
    if baseline_inputs and current.keys() != previous.keys():
        raise ValueError(f'Unmatched configurations: {len(current.keys() - previous.keys())} current only, {len(previous.keys() - current.keys())} baseline only')
    rows, units, ledger, campaigns = tally(current)
    before, before_units, _, _ = tally(previous)
    factions = sorted(set(r['race'] for r in rows))
    maps = sorted(set(r['map'] for r in rows))
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
    tables = {'campaign_runs': campaign_runs, 'campaign_summary': campaign_rows, 'campaign_stages': stage_rows,
              'campaign_failures': failure_rows, 'campaign_unit_events': campaign_events, 'matchups': rows, 'unit_events': units, 'matches': ledger, 'campaigns': campaigns,
              'race_equal_policy': combine(rows, ['race'], equal=True),
              'race_map_equal_policy': combine(rows, ['race', 'map'], equal=True),
              'race_map_policy': combine(rows, ['race', 'map', 'policy'], equal=True),
              'race_all_policies': combine(rows, ['race']), 'baseline_matchups': before,
              'matchup_changes': comparison, 'match_changes': transitions, 'baseline_unit_events': before_units}
    out.mkdir(parents=True, exist_ok=True)
    for name, table in tables.items():
        write_csv(out / (name + '.csv'), table)
    metadata = {'sources': sources, 'baselineSources': previous_sources,
                'physicalMatches': len(current), 'arenaMatches': sum(o['kind'] == 'arena' for o in current.values()),
                'campaignAttempts': len(campaigns), 'factions': factions, 'maps': sorted(set(maps) | {r['map'] for r in stage_rows}), 'policies': sorted(set(policies) | {r['player'] for r in campaign_rows}),
                'participantRows': sum(r['games'] for r in rows)}
    (out / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
    findings = overview(campaign_rows, stage_rows, failure_rows, campaign_events, rows)
    lines = findings + ['# Complete combat data', '', f"Physical matches: **{len(current):,}**. Recorded arena factions: **{', '.join(factions)}**.", '',
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
                lines += [f'### {m}: row policy {p}, opponent policy {q}', '', matrix(rows, factions, m, p, q), '']
    lines += campaign_prose(campaign_rows, stage_rows, failure_rows, campaign_events)
    lines += ['## Campaign completion by tactic and opponent', '', markdown(campaign_rows, ['campaign', 'player', 'enemy', 'diplomacy', 'maxRounds', 'attempts', 'completed', 'completionRate', 'defeats', 'caps']), '',
              '## Every campaign stage', '', markdown(stage_rows, ['campaign', 'player', 'enemy', 'diplomacy', 'maxRounds', 'stage', 'map', 'reached', 'victories', 'victoryRate', 'defeats', 'caps']), '']
    if previous:
        lines += ['## Every matchup change versus baseline', '', markdown(comparison, ['race', 'opponentRace', 'policy', 'opponentPolicy', 'map', 'games', 'winRateBefore', 'winRate', 'winRateDelta', 'wins', 'losses', 'caps']), '']
    lines += ['## All unit recruitment, promotion and casualty counts', '', markdown(units, ['race', 'opponentRace', 'policy', 'opponentPolicy', 'map', 'unit', 'recruited', 'promoted', 'lost']), '',
              '## Complete exports', '', *[f'- [{name}.csv]({name}.csv)' for name in tables], '',
              'The matches export retains every physical match configuration, result, recruitment, promotion, casualty and detail-journal reference. Original gameplay journals remain beside the input reports. This program reports saved data and runs no battles.', '']
    (out / 'findings.md').write_text('\n'.join(findings) + '\n')
    (out / 'summary.md').write_text('\n'.join(lines))
    data = json.dumps({'metadata': metadata, 'tables': tables, 'findings': '\n'.join(findings)}, separators=(',', ':')).replace('<', '\\u003c')
    document = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Complete combat data</title><style>body{font:16px system-ui;background:#111827;color:#e5e7eb;margin:24px}h1,h2{color:#fff}a{color:#93c5fd}select,input{font:inherit;padding:8px;margin:6px;background:#243247;color:white;border:1px solid #64748b}table{border-collapse:collapse;font-size:14px;white-space:nowrap}th,td{border:1px solid #475569;padding:7px;text-align:left}th{background:#243247;position:sticky;top:0}.scroll{overflow:auto;max-height:70vh}small{color:#cbd5e1}pre{white-space:pre-wrap}section{margin:24px 0}</style><h1>Complete combat data</h1><p id="scope"></p><p>Win rates include unresolved round caps. Mirrored faction results describe the same physical matches. Self-play counts both participants.</p><section><h2>Source runs</h2><pre id="sources"></pre></section><section><h2>Recorded findings</h2><p><a href="findings.md">Campaign and stage outcomes, failure explanations and arena matrices</a></p><pre id="findings"></pre></section><section><h2>All recorded data</h2><label>Table <select id="table"></select></label><label>Faction <select id="race"></select></label><label>Opponent <select id="opponent"></select></label><label>Map <select id="map"></select></label><label>Policy <select id="policy"></select></label><label>Opponent policy <select id="otherPolicy"></select></label><input id="search" placeholder="Search any field"><p id="count"></p><button id="previous">Previous 100 rows</button><button id="next">Next 100 rows</button> <a id="download">Download complete table CSV</a><div class="scroll" id="results"></div></section><p><a href="summary.md">Every matrix and full Markdown report</a> · <a href="full-data.zip">Download all tables, report and original compact outcomes</a></p><script>const data=DATA;const el=id=>document.getElementById(id);const esc=x=>String(x??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));const fmt=x=>typeof x==='object'&&x!==null?JSON.stringify(x):typeof x==='number'&&!Number.isInteger(x)?x.toFixed(2):x;function render(rows){if(!rows.length)return '<p>No recorded rows for this selection.</p>';const keys=[...new Set(rows.flatMap(Object.keys))];return '<table><thead><tr>'+keys.map(k=>'<th>'+esc(k)+'</th>').join('')+'</tr></thead><tbody>'+rows.map(r=>'<tr>'+keys.map(k=>'<td>'+esc(fmt(r[k]))+'</td>').join('')+'</tr>').join('')+'</tbody></table>'}el('scope').textContent=data.metadata.physicalMatches.toLocaleString()+' physical matches · factions: '+data.metadata.factions.join(', ')+' · maps: '+data.metadata.maps.join(', ')+' · policies: '+data.metadata.policies.join(', ');el('sources').textContent=JSON.stringify({current:data.metadata.sources,baseline:data.metadata.baselineSources},null,2);function options(id,values){el(id).innerHTML='<option value="">All</option>'+values.map(x=>'<option>'+esc(x)+'</option>').join('')}options('table',Object.keys(data.tables));el('table').querySelector('option').remove();el('table').value=data.metadata.arenaMatches?'matchups':'campaign_summary';el('findings').textContent=data.findings;options('race',data.metadata.factions);options('opponent',data.metadata.factions);options('map',data.metadata.maps);options('policy',data.metadata.policies);options('otherPolicy',data.metadata.policies);let page=0;function update(){const name=el('table').value;const rows=data.tables[name].filter(r=>[['race','race'],['opponent','opponentRace'],['map','map'],['policy','policy'],['otherPolicy','opponentPolicy']].every(([id,key])=>!el(id).value||r[key]===el(id).value)&&JSON.stringify(r).toLowerCase().includes(el('search').value.toLowerCase()));el('count').textContent=rows.length.toLocaleString()+' of '+data.tables[name].length.toLocaleString()+' rows';el('download').href=name+'.csv';el('count').textContent+=' · showing '+(rows.length?page*100+1:0)+'–'+Math.min((page+1)*100,rows.length);el('results').innerHTML=render(rows.slice(page*100,(page+1)*100));el('previous').disabled=page===0;el('next').disabled=(page+1)*100>=rows.length}document.querySelectorAll('select,input').forEach(x=>x.addEventListener('input',()=>{page=0;update()}));el('previous').onclick=()=>{page--;update()};el('next').onclick=()=>{page++;update()};update();</script></html>'''.replace('DATA', data)
    (out / 'index.html').write_text(document)
    with zipfile.ZipFile(out / 'full-data.zip', 'w', compression=zipfile.ZIP_DEFLATED) as bundle:
        for name in [*(name + '.csv' for name in tables), 'metadata.json', 'summary.md', 'findings.md', 'index.html']:
            bundle.write(out / name, name)
        for label, paths in [('current', inputs), ('baseline', baseline_inputs)]:
            for i, path in enumerate(paths):
                journal = path.with_suffix('.outcomes.jsonl')
                if journal.exists():
                    bundle.write(journal, f'{label}-{i}.outcomes.jsonl')
    elapsed = time.perf_counter() - start
    (out / 'timings.json').write_text(json.dumps({'reportSeconds': elapsed, 'physicalMatches': len(current)}, indent=2) + '\n')
    print('\n'.join(findings))
    print(f'{len(current):,} physical matches; {len(factions)} recorded factions; {len(rows)} matchup rows; {len(units)} unit-event rows; {elapsed:.2f}s. {out / "index.html"}')
    return tables


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('inputs', nargs='+', type=Path)
    parser.add_argument('--baseline', action='append', default=[], type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    try:
        build_report(args.inputs, args.baseline, args.out)
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
