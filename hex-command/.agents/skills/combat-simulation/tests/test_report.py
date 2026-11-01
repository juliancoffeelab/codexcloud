import importlib.util
import csv
import gzip
import hashlib
import contextlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('report_combat', Path(__file__).parents[1] / 'scripts/report-combat.py')
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)



def match(seed, player='a', enemy='b', winner='player', reason='leader'):
    return {'kind': 'arena', 'config': {'seed': seed, 'armies': {'player': player, 'enemy': enemy},
            'player': 'mixed', 'enemy': 'mixed', 'landscape': 'open', 'first': 'player',
            'maxRounds': 30, 'recruitLimit': 1}, 'winner': winner, 'reason': reason, 'rounds': 10,
            'recruits': [], 'casualties': [], 'promotions': [], 'stateHash': str(seed)}


class CompleteDataTests(unittest.TestCase):
    def test_compressed_reports_keep_effective_recruitment_profiles(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'report.json.gz'
            profiles = {'standard': {'speed': 0}, 'aggressivePredictor': {'speed': 0.5}}
            with gzip.open(path, 'wt') as stream:
                json.dump({'outcomes': [match(1)], 'recruitmentProfiles': profiles}, stream)
            outcomes, metadata = report.load_run([path])
            self.assertEqual(len(outcomes), 1)
            self.assertEqual(metadata[0]['recruitmentProfiles'], profiles)

    def test_unit_usage_writes_damage_presence_comparison(self):
        profiles = {u: dict(name=u.title(), family=u, race=race, level=level)
                    for u, race, level in [('foot', 'a', 1), ('unused', 'a', 2), ('foe', 'b', 1)]}
        roster = [dict(id='f', type='foot', side='player', tile='0,0', hp=10),
                  dict(id='e', type='foe', side='enemy', tile='1,0', hp=10)]
        initial = dict(units=roster, tiles=[dict(id=u['tile'], terrain='plains') for u in roster])
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            paths = []
            for label, damage in [('before', 2), ('after', 4)]:
                directory = root/label
                directory.mkdir()
                saved = match(1)
                saved['config'].update(player='situational', enemy='situational')
                stage = dict(rawDamage=damage, afterArmourDamage=damage, afterTerrainDamage=damage,
                    availableHp=10, effectiveArmourPrevented=0, effectiveTerrainPrevented=0)
                raw = json.dumps(dict(kind='arena', config=saved['config'], initial=initial,
                    phases=[dict(side='player', before=roster, units=roster, losses=[], actions=[],
                        ownership=[], combats=[dict(unit='f', target='e', damage=damage, retaliation=0,
                            thorns=dict(damage=0, auraDamage=0, effectiveDamage=0, auraSources=[]),
                            mitigation=dict(attack=stage, retaliation=None))])])).encode()
                (directory/'details.jsonl').write_bytes(raw)
                saved['detail'] = dict(file='details.jsonl', offset=0, bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())
                path = directory/('report.json.gz' if label == 'after' else 'report.json')
                with gzip.open(path, 'wt') if path.suffix == '.gz' else path.open('w') as stream:
                    json.dump(dict(status='complete', outcomes=[saved], unitProfiles=profiles,
                        supportTelemetryVersion=1, combatTelemetryVersion=1), stream)
                paths.append(path)
            with contextlib.redirect_stdout(io.StringIO()):
                tables = report.build_report([paths[1]], [paths[0]], root/'out', unit_usage=True)
            row = next(r for r in tables['unit_damage_changes'] if r['unit']=='foot' and r['map']=='all')
            self.assertEqual((row['damagePerUnitTurnBefore'], row['damagePerUnitTurn']), (2, 4))
            self.assertEqual((row['presenceSharePercent'], row['damagePresenceRatio']), (100, 1))
            text = (root/'out/findings.md').read_text()
            self.assertIn('| Foot (L1) | unknown | 100.00% → 100.00% | 2.00 → 4.00 | 1.00× → 1.00× |', text)
            paired = text.split('## Battlefield presence and recruitment flow', 1)[1].split('## Purchase shares', 1)[0]
            self.assertIn('| a | situational | situational | Plains | foot |', paired)
            self.assertLess(paired.index('sharePercent'), paired.index('purchaseSharePercent'))
            self.assertLess(text.index('## Battlefield presence'), text.index('## Purchase shares'))
            self.assertIn('replacement churn', paired)
            self.assertTrue((root/'out/unit_damage_changes.csv').is_file())
            self.assertTrue((root/'out/baseline_unit_damage.csv').is_file())

    def test_standalone_report_compares_campaigns_without_git_or_catalogs(self):
        def campaign(seed, completed, missions):
            return {'kind': 'campaign', 'config': {'seed': seed, 'campaign': 'example'},
                    'completed': completed, 'missions': missions}
        def mission(winner, state_hash, round=10, id='chapter-one'):
            return {'id': id, 'winner': winner, 'stateHash': state_hash, 'round': round}
        before = [campaign(1, False, [mission('enemy', 'defeat')]),
                  campaign(2, True, [mission('player', 'old')]),
                  campaign(3, True, [mission('player', 'same')]),
                  campaign(4, False, [mission('player', 'same')]), match(5)]
        after = [campaign(1, True, [mission('player', 'victory')]),
                 campaign(2, True, [mission('player', 'new')]),
                 {**before[2], 'elapsedMs': 99, 'detail': {'file': 'different.jsonl'}},
                 campaign(4, False, [mission('player', 'same'), mission(None, 'cap', id='chapter-two')]),
                 {**match(5), 'winner': 'enemy', 'stateHash': 'changed'}]
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, outcomes in [('before', before), ('after', after)]:
                saved = {'outcomes': outcomes}
                if name == 'after':
                    saved['sourceCommit'] = 'unavailable-commit'
                (root / (name + '.json')).write_text(json.dumps(saved))
            (root/'out').mkdir()
            (root/'out/unit_stats.csv').write_text('stale catalog export')
            (root/'analyze.py').write_text(Path(report.__file__).read_text())
            result = subprocess.run([sys.executable, 'analyze.py', 'after.json',
                                     '--baseline', 'before.json', '--out', 'out'],
                                    cwd=root, env={'PATH': ''}, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            with (root/'out/match_changes.csv').open() as f:
                changes = list(csv.DictReader(f))
            self.assertEqual([(r['resultChanged'] == 'True', r['gameplayChanged'] == 'True') for r in changes],
                             [(True, True), (False, True), (False, False), (True, True), (True, True)])
            self.assertEqual((changes[0]['beforeCompleted'], changes[0]['completed']), ('False', 'True'))
            self.assertEqual(json.loads(changes[0]['beforeMissions']), before[0]['missions'])
            self.assertEqual(json.loads(changes[0]['missions']), after[0]['missions'])
            metadata = json.loads((root/'out/metadata.json').read_text())
            self.assertEqual(metadata['sources'][0]['sourceCommit'], 'unavailable-commit')
            self.assertIsNone(metadata['baselineSources'][0]['sourceCommit'])
            self.assertNotIn('catalogs', metadata)
            self.assertTrue((root/'out/summary.md').is_file())
            self.assertIn('Field composition unavailable', (root/'out/findings.md').read_text())

    def test_swapped_seats_caps_and_selfplay_do_not_inflate_physical_matches(self):
        items = [match(1), match(2, 'b', 'a'), match(3, winner=None, reason='cap'), match(4, 'a', 'a')]
        rows, _, ledger, _ = report.tally({report.identity(x): x for x in items})
        self.assertEqual(len(ledger), 4)
        self.assertEqual(sum(x['games'] for x in rows), 8)
        cross = next(x for x in rows if x['race'] == 'a' and x['opponentRace'] == 'b')
        self.assertEqual((cross['wins'], cross['losses'], cross['caps']), (1, 1, 1))
        self.assertAlmostEqual(cross['winRate'], 100/3)
        own = next(x for x in rows if x['race'] == x['opponentRace'])
        self.assertEqual((own['games'], own['wins'], own['losses']), (2, 1, 1))
        totals = report.combine(rows, ['race', 'map', 'policy'], equal=True)
        a = next(x for x in totals if x['race'] == 'a')
        self.assertEqual((a['games'], a['wins'], a['losses'], a['caps']), (3, 1, 1, 1))
        different_policy = match(5)
        different_policy['config']['enemy'] = 'rush'
        items.append(different_policy)
        expanded, _, _, _ = report.tally({report.identity(x): x for x in items})
        self.assertEqual(totals, report.combine(expanded, ['race', 'map', 'policy'], equal=True))

    def test_race_map_overview_counts_classic_opponents_before_campaign_details(self):
        first = match(1)
        first['config']['enemy'] = 'classic'
        capped = match(2, enemy='c', winner=None, reason='cap')
        capped['config']['enemy'] = 'classic'
        swapped = match(3, 'b', 'a', winner='enemy')
        swapped['config']['player'] = 'classic'
        other_policy = match(4)
        other_policy['config']['enemy'] = 'rush'
        classic_selfplay = match(5, 'a', 'a')
        classic_selfplay['config'].update(player='classic', enemy='classic')
        island = match(6)
        island['config'].update(enemy='classic', landscape='islands')
        items = [first, capped, swapped, other_policy, classic_selfplay, island]
        rows, _, _, _ = report.tally({report.identity(x): x for x in items})
        summary = report.arena_race_map_summary(rows)
        a = next(r for r in summary if r['race']=='a' and r['map']=='open')
        self.assertEqual((a['policy'], a['games'], a['wins'], a['losses'], a['caps']),
                         ('mixed', 3, 2, 0, 1))
        self.assertFalse(any(r['race']=='a' and r['policy']=='classic' for r in summary))
        campaigns = [dict(campaign='sample', player='mixed', enemy='classic', maxRounds=30,
                          recruitLimit=1, attempts=2, completed=1)]
        text = '\n'.join(report.overview(campaigns, [], [], [], rows))
        self.assertIn('Additional pairing: mixed against classic', text)
        self.assertIn('not recorded', text)
        self.assertLess(text.index('Arena faction matchups'), text.index('Campaigns against'))
        self.assertLess(text.index('1/2'), text.index('Campaign stage outcomes'))
        self.assertLess(text.index('Matching-policy faction mirrors'), text.index('Campaign stage outcomes'))
        missing_rows, _, _, _ = report.tally({report.identity(other_policy): other_policy})
        self.assertIn('Additional pairing: mixed against rush',
                      '\n'.join(report.overview([], [], [], [], missing_rows)))

    def test_purchase_shares_use_recorded_families_and_only_new_cross_faction_purchases(self):
        rows = [dict(race='a', opponentRace='b', policy='mixed', opponentPolicy='balanced', map='open', unit='base', recruited=2, promoted=0),
                dict(race='a', opponentRace='c', policy='mixed', opponentPolicy='balanced', map='open', unit='upgrade', recruited=1, promoted=8),
                dict(race='a', opponentRace='b', policy='mixed', opponentPolicy='balanced', map='open', unit='other', recruited=1, promoted=0),
                dict(race='a', opponentRace='a', policy='mixed', opponentPolicy='balanced', map='open', unit='other', recruited=100, promoted=0)]
        result = report.purchase_shares(rows, {'base':'tank','upgrade':'tank'})
        pooled = [r for r in result if r['map']=='all']
        self.assertEqual([(r['family'], r['purchased'], r['totalPurchases'], r['sharePercent']) for r in pooled], [('other',1,4,25.0),('tank',3,4,75.0)])
        summary = report.arena_race_map_summary([dict(race='a',opponentRace='b',policy='mixed',opponentPolicy='balanced',map='open',games=2,wins=1,losses=0,caps=1,leaderWins=1,territoryWins=0,rounds=40)], 'balanced')
        self.assertEqual((summary[0]['games'],summary[0]['winRate'],summary[0]['caps']), (2,50.0,1))
        mirror = dict(race='a',opponentRace='a',policy='mixed',opponentPolicy='balanced',map='open',games=100,wins=100,losses=0,caps=0,leaderWins=100,territoryWins=0,rounds=100)
        cross = dict(mirror, opponentRace='b', games=2, wins=1, caps=1, leaderWins=1)
        fixed = report.arena_race_map_summary([mirror,cross], 'balanced')
        self.assertEqual((fixed[0]['games'],fixed[0]['winRate']), (2,50.0))


    def test_default_arena_panels_group_policies_keep_missing_cells_and_scope(self):
        items = []
        def arena(seed, p, q, winner='player', landscape='open', races=('human', 'skeletons')):
            item = match(seed, *races, winner=winner, reason='cap' if winner is None else 'leader')
            item['config'].update(player=p, enemy=q, landscape=landscape)
            items.append(item)
        arena(1, 'balanced', 'balanced')
        arena(2, 'balanced', 'balanced', winner=None)
        arena(3, 'situational', 'balanced')
        arena(4, 'predictor', 'balanced', landscape='crossings')
        arena(5, 'situational', 'situational', winner='enemy')
        arena(6, 'predictor', 'predictor', winner=None)
        arena(7, 'custom', 'classic', landscape='islands', races=('woodland', 'coastal'))
        arena(8, 'custom', 'classic', landscape='wooded', races=('other', 'human'))
        rows, _, _, _ = report.tally({report.identity(x): x for x in items})
        text = '\n'.join(report.overview([], [], [], [], rows))
        self.assertIn('Arrow order: **Balanced → Situational → Predictor**', text)
        self.assertIn('Matching-policy self-play: each arrow changes both armies', text)
        self.assertIn('Fixed opponent: Balanced; arrows change only the row policy', text)
        self.assertIn('50.0% (1) → 100.0% (0) → —', text)
        self.assertIn('| Faction | Humans | Skeletons | Woodland | Merfolk | other |', text)
        self.assertIn('| Humans | Mirror |', text)
        self.assertIn('Additional pairing: custom against classic', text)
        self.assertNotIn('Arena wins by race and map', text)
        self.assertNotIn('### balanced against balanced:', text)
        self.assertIn('Physical outcomes unavailable', text)
        fixed = text.split('### Fixed opponent:', 1)[1]
        self.assertLess(fixed.index('#### Plains'), fixed.index('#### Crossing'))
        # A default fixed-opponent run must not repeat B/B as four self-play tables.
        restricted = [r for r in rows if r['opponentPolicy'] == 'balanced']
        default = '\n'.join(report.overview([], [], [], [], restricted))
        self.assertNotIn('Matching-policy self-play:', default)
        self.assertEqual(default.count('Fixed opponent: Balanced;'), 1)

    def test_mirror_initiative_uses_physical_games_once_and_excludes_policy_comparisons(self):
        physical = []
        for seed in range(100):
            first = 'enemy' if seed % 2 else 'player'
            winner = first if seed < 60 else ('enemy' if first == 'player' else 'player') if seed < 90 else None
            item = match(seed, 'a', 'a', winner=winner)
            item['config'].update(first=first, player='predictor', enemy='predictor')
            physical.append(item)
        comparison = match(101, 'a', 'a')
        comparison['config'].update(player='predictor', enemy='situational')
        physical.append(comparison)
        rows, _, _, _ = report.tally({report.identity(x): x for x in physical})
        mirror = next(r for r in rows if r['policy'] == r['opponentPolicy'] == 'predictor')
        self.assertEqual(mirror['games'], 200)
        text = '\n'.join(report.overview([], [], [], [], rows, physical_outcomes=physical))
        initiative = text.split('### Matching-policy faction mirrors:', 1)[1]
        self.assertIn('60.0% (10; n=100)', initiative)
        self.assertNotIn('n=200', initiative)
        self.assertNotIn('n=101', initiative)
        self.assertNotIn('50.0%', initiative)
        incomplete = {**physical[0], 'config': {k:v for k,v in physical[0]['config'].items() if k != 'first'}}
        unavailable = '\n'.join(report.overview([], [], [], [], rows, physical_outcomes=[incomplete]))
        self.assertIn('lack recorded initiative/outcome', unavailable)
        self.assertIn('No matching-policy faction mirrors', unavailable)


    def test_campaign_overview_keeps_diplomacy_outcomes_separate(self):
        rows = [dict(campaign='choice', player='mixed', enemy='classic', maxRounds=30,
                     recruitLimit=1, diplomacy=mode, attempts=10, completed=wins)
                for mode, wins in [('pragmatic', 3), ('restrained', 8)]]
        rows.append(dict(campaign='single', player='mixed', enemy='classic', maxRounds=30,
                         recruitLimit=1, diplomacy='pragmatic', attempts=10, completed=10))
        text = '\n'.join(report.overview(rows, [], [], [], []))
        table = text.split('## Campaign stage outcomes')[0]
        self.assertIn('| choice (pragmatic) | 3/10 |', table)
        self.assertIn('| choice (restrained) | 8/10 |', table)
        self.assertIn('| single | 10/10 |', table)
        self.assertNotIn('11/20', table)
        self.assertNotIn('%', table)
        stages = [dict(campaign='choice', player='mixed', enemy='classic', maxRounds=30,
                       recruitLimit=1, diplomacy=mode, stage=1, map='entry', reached=10,
                       victories=wins, defeats=10-wins, caps=0)
                  for mode, wins in [('pragmatic', 3), ('restrained', 8)]]
        detailed = '\n'.join(report.overview(rows, stages, [], [], []))
        self.assertIn('| choice: entry (pragmatic) | 3/10; 7D 0C |', detailed)
        self.assertIn('| choice: entry (restrained) | 8/10; 2D 0C |', detailed)
        self.assertIn('Why choice (pragmatic) attempts stop', detailed)
        self.assertIn('Why choice (restrained) attempts stop', detailed)
        self.assertNotIn('11/20', detailed)

    def test_campaign_overview_distinguishes_policy_defaults_from_explicit_caps(self):
        outcomes = {}
        for seed, limit in [(1, None), (2, 1)]:
            item = dict(kind='campaign', config=dict(seed=seed, campaign='sample', player='mixed',
                        enemy='classic', diplomacy='pragmatic', maxRounds=30, recruitLimit=limit),
                        completed=True, missions=[])
            outcomes[report.identity(item)] = item
        campaigns, stages, failures, events, _ = report.campaign_tables(outcomes)
        text = '\n'.join(report.overview(campaigns, stages, failures, events, []))
        self.assertIn('Classic 2 per phase; other tactics uncapped', text)
        self.assertIn('1 recruits per phase (explicit override)', text)
        self.assertEqual(len(campaigns), 2)

    def test_policy_matrix_includes_same_race_opponents_and_keeps_caps(self):
        won = match(1, 'a', 'a')
        won['config'].update(player='priority', enemy='mixed')
        capped = match(2, 'a', 'b', winner=None, reason='cap')
        capped['config'].update(player='priority', enemy='mixed')
        reverse = match(3, 'b', 'a', winner='enemy')
        reverse['config'].update(player='mixed', enemy='priority')
        same_policy = match(4, 'a', 'b')
        rows, _, _, _ = report.tally({report.identity(x): x for x in [won, capped, reverse, same_policy]})
        combined = report.combine([r for r in rows if r['policy'] != r['opponentPolicy']],
                                  ['policy', 'opponentPolicy'], cross=False)
        lookup = {(r['policy'], r['opponentPolicy']): r for r in combined}
        self.assertEqual(set(lookup), {('priority', 'mixed'), ('mixed', 'priority')})
        self.assertEqual((lookup['priority', 'mixed']['games'], lookup['priority', 'mixed']['wins'],
                          lookup['priority', 'mixed']['caps']), (3, 2, 1))
        self.assertEqual(lookup['mixed', 'priority']['losses'], 2)

    def test_custom_blocks_keep_tactic_labels_distinct_in_arena_and_campaigns(self):
        custom = match(1)
        custom['config'].update(player='balanced', enemy='classic',
                                playerBlocks=dict(recruiter='rotation', campaign='arena', planner='queued'))
        named = match(2)
        named['config'].update(player='balanced', enemy='classic')
        rows, _, _, _ = report.tally({report.identity(x): x for x in [custom, named]})
        self.assertEqual({r['policy'] for r in rows}, {'rotation-arena-queued', 'balanced', 'classic'})
        campaign = dict(kind='campaign', completed=True, missions=[], config=dict(seed=1,
                        campaign='sample', player='balanced', enemy='balanced', diplomacy='pragmatic',
                        maxRounds=100, recruitLimit=None,
                        enemyBlocks=dict(recruiter='rotation', campaign='mission', planner='queued')))
        summary, _, _, _, _ = report.campaign_tables({report.identity(campaign): campaign})
        self.assertEqual((summary[0]['player'], summary[0]['enemy']), ('balanced', 'rotation-mission-queued'))

    def test_unit_events_follow_actual_side_when_factions_swap_seats(self):
        item = match(1, 'b', 'a')
        item['recruits'] = [{'side': 'enemy', 'type': 'caster'}]
        item['promotions'] = [{'side': 'enemy', 'type': 'veteran'}]
        item['casualties'] = [{'side': 'player', 'type': 'screen'}]
        _, units, _, _ = report.tally({report.identity(item): item})
        self.assertEqual({(x['race'], x['unit']): (x['recruited'], x['promoted'], x['lost']) for x in units},
                         {('a', 'caster'): (1, 0, 0), ('a', 'veteran'): (0, 1, 0), ('b', 'screen'): (0, 0, 1)})

    def test_combat_use_is_attributed_to_actual_sides_without_inventing_missing_records(self):
        item = match(1, 'b', 'a')
        item['combatUsage'] = [dict(side='player', type='mounted', attacks=3, retaliations=1, damage=25),
                               dict(side='enemy', type='guard', attacks=1, retaliations=2, damage=9)]
        _, units, _, _ = report.tally({report.identity(item): item})
        self.assertEqual({(r['race'], r['unit']): (r['attacks'], r['retaliations'], r['damage']) for r in units},
                         {('b', 'mounted'): (3, 1, 25), ('a', 'guard'): (1, 2, 9)})

    def test_campaign_journals_explain_conditional_stages_deaths_and_active_caps(self):
        def chapter(winner, failure=None, id='grove'):
            leader = dict(id='u1', type='leader', side='player', name='Captain', role='leader', tile='0,0', hp=20)
            actor = dict(id='u2', type='caster', side='player', name='Moss', tile='1,1', hp=5)
            enemy = dict(id='u3', type='guard', side='enemy', tile='3,3', hp=10)
            initial = [leader, actor, enemy]
            losses = [actor] if failure == 'actor' else [leader] if failure == 'leader' else [leader, actor] if failure == 'all' else []
            final = [u for u in initial if u not in losses]
            if winner == 'player':
                final = [dict(u, tile='5,5') if u['id']=='u2' else u for u in final]
            goals = [dict(label='Keep Moss in the grove', tiles=['5,5'], current=int(winner=='player'), total=1)]
            phases = [dict(side='player', round=i, before=initial, units=final, ownership=[], goals=goals,
                           actions=[dict(kind='setUnitIntent')], losses=[], combats=[])
                      for i in range(1, 11 if winner is None else 2)]
            if losses:
                phases.append(dict(side='enemy', round=1, before=initial, units=final, ownership=[], goals=goals,
                                   actions=[], losses=losses,
                                   combats=[dict(unit='u3', target='u2', damage=7, retaliation=2)]))
            reason = 'Moss fell. The company cannot continue.' if failure=='actor' else 'Your leader fell.' if failure=='leader' else None
            return dict(id=id, winner=winner, round=len(phases), initial=dict(units=initial),
                        final=dict(units=final, campaign=dict(outcome=reason)), goals=goals, phases=phases)
        histories = [[chapter('player', id='entry'), chapter('player')],
                     [chapter(None, id='entry')],
                     [chapter('player', id='entry'), chapter('enemy', 'actor')],
                     [chapter('player', id='entry'), chapter('enemy', 'leader')],
                     [chapter('enemy', 'all', id='entry')]]
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            records, outcomes, offset = [], [], 0
            for seed, missions in enumerate(histories):
                config = dict(seed=seed, campaign='sample', player='mixed', enemy='classic', diplomacy='restrained', maxRounds=10, recruitLimit=1)
                raw = dict(kind='campaign', config=config, completed=seed==0, missions=missions)
                line = (json.dumps(raw)+'\n').encode()
                outcomes.append(dict(kind='campaign', config=config, completed=seed==0,
                                     missions=[{k: m[k] for k in ('id','winner','round')} for m in missions],
                                     detail=dict(file='report.details.jsonl', offset=offset, bytes=len(line), sha256=hashlib.sha256(line).hexdigest())))
                records.append(line);offset += len(line)
            (root/'report.details.jsonl').write_bytes(b''.join(records))
            path = root/'report.json';path.write_text(json.dumps(dict(outcomes=outcomes)))
            with contextlib.redirect_stdout(io.StringIO()) as output:
                tables = report.build_report([path], [], root/'analytics')
            row = tables['campaign_summary'][0]
            self.assertEqual((row['attempts'],row['completed'],row['defeats'],row['caps']), (5,1,3,1))
            stages = {r['map']: r for r in tables['campaign_stages']}
            self.assertEqual((stages['grove']['reached'],stages['grove']['victories'],stages['grove']['defeats']), (3,1,2))
            self.assertEqual((stages['entry']['idleCaps'], stages['entry']['repeatedCaps']), (0,1))
            self.assertEqual({r['category'] for r in tables['campaign_failures']}, {'round cap','required actor lost','leader lost','all player units lost'})
            self.assertIn('Moss fell', output.getvalue())
            actor_loss = next(r for r in tables['campaign_runs'] if r['category']=='required actor lost')
            self.assertEqual((actor_loss['seed'],actor_loss['map'],actor_loss['finalPlayerUnits']), (2,'grove',1))
            self.assertEqual(next(r for r in tables['campaign_failures'] if r['category']=='required actor lost')['exampleSeeds'], [2])
            self.assertIn('| sample (restrained) | 1/5 |', output.getvalue())
            self.assertIn('grove', (root/'analytics/findings.md').read_text())
            with gzip.open(root/'report.details.jsonl.gz', 'wb') as compressed:
                compressed.write((root/'report.details.jsonl').read_bytes())
            (root/'report.details.jsonl').unlink()
            with contextlib.redirect_stdout(io.StringIO()):
                compressed_tables = report.build_report([path], [], root/'compressed')
            self.assertEqual(tables['campaign_summary'], compressed_tables['campaign_summary'])

            outcomes[0]['detail']['sha256']='wrong'
            path.write_text(json.dumps(dict(outcomes=outcomes)))
            with self.assertRaisesRegex(ValueError, 'checksum'):
                report.load_run([path])
            legacy = root/'legacy';legacy.mkdir()
            (legacy/'outcomes.json').write_text(json.dumps(outcomes[1:]))
            with gzip.open(legacy/'details.jsonl.gz','wb') as stream:
                stream.write(b''.join(records[1:]))
            loaded, _ = report.load_run([legacy])
            self.assertEqual(len(loaded),4)
            self.assertTrue(all(o['analysis'] for o in loaded.values()))

    def test_planner_diagnostics_and_cpu_are_reported_without_extra_queries(self):
        phase = dict(side='player', before=[], units=[], actions=[], losses=[], goals=[], combats=[dict(unit='u2', target='u3', damage=4, retaliation=2)],
                     planning=dict(orders=dict(tactics=dict(constraints=[dict(Survive=dict(actor=2))])),
                                   report=dict(candidates=7, duties=[dict(id='survive:u2', status='Unmet', reason='UnsafeFallback')])))
        diagnostic = report.chapter_diagnostics(dict(winner=None, goals=[], phases=[phase], final=dict(units=[])))
        self.assertEqual(diagnostic['unmetOrders'], {'UnsafeFallback': 1})
        self.assertEqual((diagnostic['protectedRetaliationExchanges'], diagnostic['rescueCandidates']), (1, 7))
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'report.json'
            path.write_text(json.dumps(dict(outcomes=[match(1)], turnTiming=dict(clock='thread_cpu', policies=[dict(policy='mixed', turns=10, meanDecisionMicrosPerTurn=12, meanTurnMicros=15)]))))
            with contextlib.redirect_stdout(io.StringIO()) as output:
                tables = report.build_report([path], [path], Path(tmp)/'analytics')
            self.assertEqual(len(tables['policy_timings']), 2)
            self.assertIn('Chooser and phase CPU', output.getvalue())

    def test_invalid_and_duplicate_outcomes_and_unpaired_baselines_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            path, baseline = Path(tmp)/'report.json', Path(tmp)/'baseline.json'
            path.write_text(json.dumps({'outcomes': [match(1), match(1)]}))
            with self.assertRaisesRegex(ValueError, 'Duplicate'):
                report.load_run([path])
            path.write_text(json.dumps({'outcomes': [match(1, winner=None)]}))
            with self.assertRaisesRegex(ValueError, 'inconsistency'):
                report.load_run([path])
            path.write_text(json.dumps({'outcomes': [match(1)]}))
            baseline.write_text(json.dumps({'outcomes': [match(2)]}))
            with self.assertRaisesRegex(ValueError, 'Unmatched configurations'):
                report.build_report([path], [baseline], Path(tmp)/'out')


if __name__ == '__main__':
    unittest.main()
