"""Create the Marlow Cut cross-opponent qualification report."""
import itertools

from build import BASE, CATALOG, OUT, STARTS, VISIBILITY, arena, location, read, write
from evaluate import ENEMIES, digest
from trace import events


def verify_fixture():
    for start in STARTS:
        original = location(BASE, arena(start, 'clear'))
        old_map = read(original / 'map.json')
        old_roster = read(original / 'roster.json')
        for visibility in VISIBILITY:
            ident = arena(start, visibility)
            stage = location(CATALOG, ident)
            terrain, roster, rule = (read(stage / f'{name}.json') for name in ('map', 'roster', 'simulation-rule'))
            assert rule['territory'] == [{'col': 4, 'row': 2}] and rule['holdTurns'] == 6
            assert roster['factions'] == old_roster['factions']
            assert terrain['mapId'] == ident
            terrain['mapId'] = old_map['mapId']
            if visibility != 'clear':
                for tile in terrain['tiles']:
                    tile['environmentEffects'].remove('fog-reveal-3')
            assert terrain == old_map


def compare_queued_predictor(rows, replay):
    comparable = {(r['cell'][0], r['cell'][1], r['cell'][2], r['player'], r['config']['seed']): r['result'] for r in rows}
    whole_games = [(r, comparable[start, visibility, 'predictor', player, seed])
                   for (start, visibility, opponent, player, seed), r in comparable.items() if opponent == 'queued']
    detailed = {(r['cell'][0], r['cell'][1], r['cell'][2], r['player'], r['config']['seed']): r['objectiveHistory']
                for r in replay if r['panel'] == 'discovery'}
    detailed_pairs = [(r, detailed[start, visibility, 'predictor', player, seed])
                      for (start, visibility, opponent, player, seed), r in detailed.items()
                      if opponent == 'queued' and (start, visibility, 'predictor', player, seed) in detailed]
    enemy_advances = {opp: sum(a['kind'] == 'advance' for r in replay if r['cell'][2] == opp
                              for phase in r['objectiveHistory']['phases'] if phase['side'] == 'ai' for a in phase['actions'])
                      for opp in ENEMIES}
    new_recruits = sum(a['kind'] == 'recruit' for r in replay for phase in r['objectiveHistory']['phases'] for a in phase['actions'])
    assert new_recruits == 0
    assert enemy_advances['queued'] and enemy_advances['predictor']
    return dict(terminalPairs=len(whole_games), terminalDifferences=sum(a != b for a, b in whole_games),
                replayPairs=len(detailed_pairs), replayDifferences=sum(a != b for a, b in detailed_pairs),
                enemyPromotionsInReplays=enemy_advances, recruitsInReplays=new_recruits)


def main():
    verify_fixture()
    discovery = read(OUT / 'discovery/protocol.json')
    confirmation = read(OUT / 'confirmation/protocol.json')
    assert len(discovery['seeds']) == 8 and len(confirmation['seeds']) == 20
    assert set(discovery['seeds']).isdisjoint(confirmation['seeds'])
    for relative, sha in discovery['sha256'].items():
        assert digest(CATALOG / relative) == sha == confirmation['sha256'][relative]
    screen_games = events('discovery')
    confirm_games = events('confirmation')
    assert len(screen_games) == discovery['count'] == 960
    assert len(confirm_games) == confirmation['count'] == 60
    assert not any(row['result']['truncated'] for row in screen_games + confirm_games)
    replay = read(OUT / 'traces/objective-progress.json')
    promotions = compare_queued_predictor(screen_games, replay)
    chosen = read(OUT / 'selection.json')['cells']
    results = read(OUT / 'confirmation/summary.json')
    final = []
    for entry in (entry for entry in chosen if entry['qualifies']):
        wins = {row['player']: row['wins'] for row in results if row['cell'] == entry['cell']}
        reference = entry['reference']
        qualified = wins[reference] >= 15 and wins[reference]-wins['hour10'] >= 6
        final.append(dict(cell=entry['cell'], reference=reference, wins=wins, qualified=qualified,
                          learnerScope='hour10-only' if qualified and wins[reference]-wins['hour8'] < 6 else 'both-learners'))
    write(OUT / 'final-summary.json', dict(
        family='interrupt-or-finish', stage='Marlow Cut', status='clear-classic-only',
        discovery=dict(seeds=discovery['seeds'], totalGames=len(screen_games), results='discovery/summary.json',
                       raw='discovery/games.jsonl', protocol='discovery/protocol.json'),
        confirmation=dict(seeds=confirmation['seeds'], totalGames=len(confirm_games), results=final,
                          raw='confirmation/games.jsonl', protocol='confirmation/protocol.json'),
        nonQualifiedConditions=23, selection='selection.json', nativeObservation='observation-evidence.json',
        traces=dict(games=len(replay), raw='traces/raw.jsonl.gz', objectives='traces/objective-progress.json',
                    protocol='traces/protocol.json'),
        opponents=discovery['opponents'], queuedPredictorAudit=promotions,
        fogVariants={'clear': 'Unlimited for both sides. Exact original Marlow Cut map/roster.',
                     'player-fog': 'Radius three for the player and unlimited enemy vision.',
                     'both-fog': 'Radius three for both sides.'},
        fixtureFolders={f'{start}/{sight}': str(location(CATALOG, arena(start, sight)).relative_to(OUT))
                        for start, sight in itertools.product(STARTS, VISIBILITY)},
        methodSha256={script: digest(OUT / script) for script in ('build.py','evaluate.py','observe.py','trace.py','summarize.py')},
        catalog='catalog', referencePlacement='clear Finish against Classic, Hour 10 criterion met',
    ))
    print(promotions)


if __name__ == '__main__':
    main()
