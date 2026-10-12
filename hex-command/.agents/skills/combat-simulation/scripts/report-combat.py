"""Complete reports from recorded combat runs; Python standard library only."""
import argparse
import csv
import html
import json
from collections import Counter, defaultdict
from pathlib import Path
import subprocess
import time
import zipfile

ROOT = Path(__file__).resolve().parents[4]


def identity(outcome):
    return outcome['kind'] + ':' + json.dumps(outcome['config'], sort_keys=True, separators=(',', ':'))


def load_run(paths):
    outcomes, sources = {}, []
    for path in paths:
        report = json.loads(path.read_text())
        source = {k: report.get(k) for k in ('sourceCommit', 'fingerprint', 'status', 'mode', 'schedule', 'seeds', 'elapsedSeconds')}
        source['file'] = str(path.resolve())
        source['matches'] = len(report['outcomes'])
        sources.append(source)
        for item in report['outcomes']:
            key = identity(item)
            if key in outcomes:
                raise ValueError('Duplicate match configuration: ' + key)
            if item['kind'] == 'arena':
                if item.get('winner') not in (None, 'player', 'enemy'):
                    raise ValueError('Invalid winner: ' + key)
                if (item.get('winner') is None) != (item.get('reason') == 'cap'):
                    raise ValueError('Winner/cap inconsistency: ' + key)
            outcomes[key] = item
    return outcomes, sources


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


def catalog(commit):
    prefix = ROOT.name + '/'
    def read(path):
        result = subprocess.run(['git', 'show', f'{commit}:{prefix}{path}'], cwd=ROOT, capture_output=True, text=True, check=True)
        return json.loads(result.stdout)
    manifest = read('rust/data/manifest.json')
    units = {u['id']: u for u in map(read, ('rust/data/' + p for p in manifest['units']))}
    races = {r['id']: r for r in map(read, ('rust/data/' + p for p in manifest['races']))}
    rosters = read('rust/data/' + manifest['simulation']['rosters'])
    reachable = {}
    for roster in rosters:
        found, pending = set(), [roster['leader'], roster['screen'], *roster['pool']]
        while pending:
            name = pending.pop()
            if name in found:
                continue
            found.add(name)
            pending.extend(units[name].get('advancesTo', []))
        reachable[roster['id']] = found
    tree = subprocess.run(['git', 'ls-tree', '-r', '--name-only', commit, prefix + 'pixel/art/units/lineups'], cwd=ROOT.parent, capture_output=True, text=True, check=True)
    coverage = []
    def ids(node):
        if isinstance(node, dict):
            if 'id' in node and 'source' in node:
                yield node['id']
            for value in node.values():
                yield from ids(value)
        elif isinstance(node, list):
            for value in node:
                yield from ids(value)
    for path in tree.stdout.splitlines():
        names = set(ids(read(path.removeprefix(prefix))))
        present = names & units.keys()
        covered = names & set().union(*reachable.values())
        coverage.append({'lineup': Path(path).stem, 'artUnits': len(names), 'gameplayUnits': len(present),
                         'arenaUnits': len(covered), 'arenaRosters': sorted(k for k, v in reachable.items() if names & v),
                         'excludedUnits': sorted(names - covered)})
    stats = []
    for name, u in sorted(units.items()):
        stats.append({'sourceCommit': commit, 'unit': name, 'name': u['name'], 'race': u['race'], 'level': u['level'],
                      'hp': u['hp'], 'move': u['move'], 'gold': u['cost'], 'attacks': u['attacks'],
                      'unitModifiers': u.get('damageModifiers', {}),
                      'raceModifiers': races[u['race']].get('damageModifiers', {}),
                      'terrainModifiers': races[u['race']].get('terrainModifiers', {}),
                      'advancesTo': u.get('advancesTo', []),
                      'arenaRosters': sorted(k for k, v in reachable.items() if name in v), 'authored': u})
    return {'sourceCommit': commit, 'rosters': rosters, 'coverage': coverage, 'units': stats}


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
    catalogs = [catalog(commit) for commit in dict.fromkeys(s['sourceCommit'] for s in sources)]
    baseline_catalogs = [catalog(commit) for commit in dict.fromkeys(s['sourceCommit'] for s in previous_sources)]
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
    tables = {'matchups': rows, 'unit_events': units, 'matches': ledger, 'campaigns': campaigns,
              'race_equal_policy': combine(rows, ['race'], equal=True),
              'race_map_equal_policy': combine(rows, ['race', 'map'], equal=True),
              'race_map_policy': combine(rows, ['race', 'map', 'policy'], equal=True),
              'race_all_policies': combine(rows, ['race']), 'baseline_matchups': before,
              'matchup_changes': comparison, 'match_changes': transitions, 'baseline_unit_events': before_units,
              'lineup_coverage': [r for c in catalogs for r in c['coverage']],
              'unit_stats': [r for c in catalogs for r in c['units']],
              'baseline_unit_stats': [r for c in baseline_catalogs for r in c['units']]}
    out.mkdir(parents=True, exist_ok=True)
    for name, table in tables.items():
        write_csv(out / (name + '.csv'), table)
    metadata = {'sources': sources, 'baselineSources': previous_sources, 'catalogs': catalogs, 'baselineCatalogs': baseline_catalogs,
                'physicalMatches': len(current), 'arenaMatches': sum(o['kind'] == 'arena' for o in current.values()),
                'campaignAttempts': len(campaigns), 'factions': factions, 'maps': maps, 'policies': policies,
                'participantRows': sum(r['games'] for r in rows)}
    (out / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
    lines = ['# Complete combat data', '', f"Physical matches: **{len(current):,}**. Recorded arena factions: **{', '.join(factions)}**.", '',
             'Each matrix cell shows **win percentage (wins/losses/caps)** for the row faction. Caps stay in the denominator. Each arena match contributes one result to each participant; mirrored cells are views of the same matches, not additional independent games. Self-play cells contain both participants. Policy pairings are kept separate.', '',
             '## Source runs', '', markdown(sources, ['sourceCommit', 'fingerprint', 'status', 'mode', 'matches', 'seeds', 'schedule']), '',
             '## Artwork and gameplay coverage', '',
             'Artwork lineups are listed in full. Zero gameplay/arena coverage means these units were not simulated. Historical catalogs come from the recorded commit, never the current checkout.', '',
             markdown(tables['lineup_coverage'], ['lineup', 'artUnits', 'gameplayUnits', 'arenaUnits', 'arenaRosters', 'excludedUnits']), '',
             '## Factions with matching policies', '', markdown(tables['race_equal_policy'], ['race', 'games', 'wins', 'losses', 'caps', 'winRate']), '',
             '## Factions by map with each policy kept separate', '',
             'These totals compare each faction with the other factions using the same policy, excluding self-play.', '',
             markdown(tables['race_map_policy'], ['race', 'map', 'policy', 'games', 'wins', 'losses', 'caps', 'winRate']), '',
             '## Every faction, map and policy pairing', '']
    for m in maps:
        for p in policies:
            for q in policies:
                lines += [f'### {m}: row policy {p}, opponent policy {q}', '', matrix(rows, factions, m, p, q), '']
    if previous:
        lines += ['## Every matchup change versus baseline', '', markdown(comparison, ['race', 'opponentRace', 'policy', 'opponentPolicy', 'map', 'games', 'winRateBefore', 'winRate', 'winRateDelta', 'wins', 'losses', 'caps']), '']
    lines += ['## All unit recruitment, promotion and casualty counts', '', markdown(units, ['race', 'opponentRace', 'policy', 'opponentPolicy', 'map', 'unit', 'recruited', 'promoted', 'lost']), '',
              '## Recorded unit stats', '', markdown(tables['unit_stats'], ['unit', 'race', 'level', 'hp', 'move', 'gold', 'attacks', 'arenaRosters']), '',
              '## Complete exports', '', *[f'- [{name}.csv]({name}.csv)' for name in tables], '',
              'The matches export retains every physical match configuration, result, recruitment, promotion, casualty and detail-journal reference. Original gameplay journals remain beside the input reports. This program reports saved data and runs no battles.', '']
    (out / 'summary.md').write_text('\n'.join(lines))
    data = json.dumps({'metadata': metadata, 'tables': tables}, separators=(',', ':')).replace('<', '\\u003c')
    document = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Complete combat data</title><style>body{font:16px system-ui;background:#111827;color:#e5e7eb;margin:24px}h1,h2{color:#fff}a{color:#93c5fd}select,input{font:inherit;padding:8px;margin:6px;background:#243247;color:white;border:1px solid #64748b}table{border-collapse:collapse;font-size:14px;white-space:nowrap}th,td{border:1px solid #475569;padding:7px;text-align:left}th{background:#243247;position:sticky;top:0}.scroll{overflow:auto;max-height:70vh}small{color:#cbd5e1}pre{white-space:pre-wrap}section{margin:24px 0}</style><h1>Complete combat data</h1><p id="scope"></p><p>Win rates include unresolved round caps. Mirrored faction results describe the same physical matches. Self-play counts both participants.</p><section><h2>Source and roster coverage</h2><pre id="sources"></pre><div class="scroll" id="coverage"></div></section><section><h2>All recorded data</h2><label>Table <select id="table"></select></label><label>Faction <select id="race"></select></label><label>Opponent <select id="opponent"></select></label><label>Map <select id="map"></select></label><label>Policy <select id="policy"></select></label><label>Opponent policy <select id="otherPolicy"></select></label><input id="search" placeholder="Search any field"><p id="count"></p><button id="previous">Previous 100 rows</button><button id="next">Next 100 rows</button> <a id="download">Download complete table CSV</a><div class="scroll" id="results"></div></section><p><a href="summary.md">Every matrix and full Markdown report</a> · <a href="full-data.zip">Download all tables, report and original compact outcomes</a></p><script>const data=DATA;const el=id=>document.getElementById(id);const esc=x=>String(x??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));const fmt=x=>typeof x==='object'&&x!==null?JSON.stringify(x):typeof x==='number'&&!Number.isInteger(x)?x.toFixed(2):x;function render(rows){if(!rows.length)return '<p>No recorded rows for this selection.</p>';const keys=[...new Set(rows.flatMap(Object.keys))];return '<table><thead><tr>'+keys.map(k=>'<th>'+esc(k)+'</th>').join('')+'</tr></thead><tbody>'+rows.map(r=>'<tr>'+keys.map(k=>'<td>'+esc(fmt(r[k]))+'</td>').join('')+'</tr>').join('')+'</tbody></table>'}el('scope').textContent=data.metadata.physicalMatches.toLocaleString()+' physical matches · factions: '+data.metadata.factions.join(', ')+' · maps: '+data.metadata.maps.join(', ')+' · policies: '+data.metadata.policies.join(', ');el('sources').textContent=JSON.stringify({current:data.metadata.sources,baseline:data.metadata.baselineSources},null,2);el('coverage').innerHTML=render(data.tables.lineup_coverage);function options(id,values){el(id).innerHTML='<option value="">All</option>'+values.map(x=>'<option>'+esc(x)+'</option>').join('')}options('table',Object.keys(data.tables));el('table').querySelector('option').remove();el('table').value='matchups';options('race',data.metadata.factions);options('opponent',data.metadata.factions);options('map',data.metadata.maps);options('policy',data.metadata.policies);options('otherPolicy',data.metadata.policies);let page=0;function update(){const name=el('table').value;const rows=data.tables[name].filter(r=>[['race','race'],['opponent','opponentRace'],['map','map'],['policy','policy'],['otherPolicy','opponentPolicy']].every(([id,key])=>!el(id).value||r[key]===el(id).value)&&JSON.stringify(r).toLowerCase().includes(el('search').value.toLowerCase()));el('count').textContent=rows.length.toLocaleString()+' of '+data.tables[name].length.toLocaleString()+' rows';el('download').href=name+'.csv';el('count').textContent+=' · showing '+(rows.length?page*100+1:0)+'–'+Math.min((page+1)*100,rows.length);el('results').innerHTML=render(rows.slice(page*100,(page+1)*100));el('previous').disabled=page===0;el('next').disabled=(page+1)*100>=rows.length}document.querySelectorAll('select,input').forEach(x=>x.addEventListener('input',()=>{page=0;update()}));el('previous').onclick=()=>{page--;update()};el('next').onclick=()=>{page++;update()};update();</script></html>'''.replace('DATA', data)
    (out / 'index.html').write_text(document)
    with zipfile.ZipFile(out / 'full-data.zip', 'w', compression=zipfile.ZIP_DEFLATED) as bundle:
        for path in sorted(out.iterdir()):
            if path.suffix in ('.csv', '.json', '.md', '.html'):
                bundle.write(path, path.name)
        for label, paths in [('current', inputs), ('baseline', baseline_inputs)]:
            for i, path in enumerate(paths):
                journal = path.with_suffix('.outcomes.jsonl')
                if journal.exists():
                    bundle.write(journal, f'{label}-{i}.outcomes.jsonl')
    elapsed = time.perf_counter() - start
    (out / 'timings.json').write_text(json.dumps({'reportSeconds': elapsed, 'physicalMatches': len(current)}, indent=2) + '\n')
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
    except (ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Cannot report combat data: {error}\n')
