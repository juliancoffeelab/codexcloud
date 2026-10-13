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
import zipfile

spec = importlib.util.spec_from_file_location('report_combat', Path(__file__).parents[1] / 'scripts/report-combat.py')
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)

runner_spec = importlib.util.spec_from_file_location('run_combat', Path(__file__).parents[1] / 'scripts/run-combat.py')
runner = importlib.util.module_from_spec(runner_spec)
runner_spec.loader.exec_module(runner)


def match(seed, player='a', enemy='b', winner='player', reason='leader'):
    return {'kind': 'arena', 'config': {'seed': seed, 'armies': {'player': player, 'enemy': enemy},
            'player': 'mixed', 'enemy': 'mixed', 'landscape': 'open', 'first': 'player',
            'maxRounds': 30, 'recruitLimit': 1}, 'winner': winner, 'reason': reason, 'rounds': 10,
            'recruits': [], 'casualties': [], 'promotions': [], 'stateHash': str(seed)}


class CompleteDataTests(unittest.TestCase):
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
            with zipfile.ZipFile(root/'out/full-data.zip') as bundle:
                self.assertIn('match_changes.csv', bundle.namelist())
                self.assertNotIn('unit_stats.csv', bundle.namelist())

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

    def test_unit_events_follow_actual_side_when_factions_swap_seats(self):
        item = match(1, 'b', 'a')
        item['recruits'] = [{'side': 'enemy', 'type': 'caster'}]
        item['promotions'] = [{'side': 'enemy', 'type': 'veteran'}]
        item['casualties'] = [{'side': 'player', 'type': 'screen'}]
        _, units, _, _ = report.tally({report.identity(item): item})
        self.assertEqual({(x['race'], x['unit']): (x['recruited'], x['promoted'], x['lost']) for x in units},
                         {('a', 'caster'): (1, 0, 0), ('a', 'veteran'): (0, 1, 0), ('b', 'screen'): (0, 0, 1)})

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
            self.assertIn('1/5 (20.0%)', output.getvalue())
            self.assertIn('grove', (root/'analytics/findings.md').read_text())
            self.assertIn('Recorded findings', (root/'analytics/index.html').read_text())
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

    def test_full_runner_separates_campaign_roles_and_runs_the_analyzer(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)
            calls = []
            def invoke(command, **kwargs):
                calls.append(command)
                if 'ai:simulate' in command:
                    (out/'report.json').write_text(json.dumps(dict(outcomes=[], elapsedSeconds=0)))
            with patch.object(sys, 'argv', ['run-combat.py', '--out', str(out)]), \
                 patch.object(runner.subprocess, 'run', side_effect=invoke), \
                 contextlib.redirect_stdout(io.StringIO()):
                runner.main()
            native = calls[0]
            self.assertEqual(native[native.index('--mode')+1], 'full')
            self.assertEqual(native[native.index('--campaign-players')+1], 'mixed,priority')
            self.assertEqual(native[native.index('--campaign-enemies')+1], 'classic')
            self.assertEqual(native[native.index('--details')+1], 'jsonl')
            self.assertIn('battler', native[native.index('--policies')+1])
            self.assertTrue(calls[1][1].endswith('report-combat.py'))
            self.assertEqual(calls[1][2], str(out/'report.json'))

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
