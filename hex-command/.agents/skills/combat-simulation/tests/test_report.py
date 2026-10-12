import importlib.util
import csv
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('report_combat', Path(__file__).parents[1] / 'scripts/report-combat.py')
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)
spec = importlib.util.spec_from_file_location('run_combat', Path(__file__).parents[1] / 'scripts/run-combat.py')
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


def match(seed, player='a', enemy='b', winner='player', reason='leader'):
    return {'kind': 'arena', 'config': {'seed': seed, 'armies': {'player': player, 'enemy': enemy},
            'player': 'mixed', 'enemy': 'mixed', 'landscape': 'open', 'first': 'player',
            'maxRounds': 30, 'recruitLimit': 1}, 'winner': winner, 'reason': reason, 'rounds': 10,
            'recruits': [], 'casualties': [], 'promotions': [], 'stateHash': str(seed)}


class CompleteDataTests(unittest.TestCase):
    def test_campaign_baseline_exports_completion_and_mission_changes(self):
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
                (root / (name + '.json')).write_text(json.dumps({'sourceCommit': 'example', 'outcomes': outcomes}))
            with patch.object(report, 'catalog', return_value={'coverage': [], 'units': []}):
                tables = report.build_report([root/'after.json'], [root/'before.json'], root/'out')
            changes = tables['match_changes']
            self.assertEqual([(r['resultChanged'], r['gameplayChanged']) for r in changes],
                             [(True, True), (False, True), (False, False), (True, True), (True, True)])
            self.assertEqual((changes[0]['beforeCompleted'], changes[0]['completed']), (False, True))
            self.assertEqual(changes[0]['beforeMissions'], before[0]['missions'])
            self.assertEqual(changes[0]['missions'], after[0]['missions'])
            with (root/'out/match_changes.csv').open() as f:
                exported = list(csv.DictReader(f))
            self.assertEqual(json.loads(exported[0]['missions']), after[0]['missions'])
            self.assertEqual(exported[0]['resultChanged'], 'True')

    def test_runner_rejects_mismatched_data_before_starting_simulations(self):
        contents = {'manifest.json': b'{"units":["units/test.json"]}',
                    'units/test.json': b'{"hp":6}'}
        archive = io.BytesIO()
        with tarfile.open(fileobj=archive, mode='w') as files:
            for name, content in contents.items():
                entry = tarfile.TarInfo(runner.ROOT.name + '/rust/data/' + name)
                entry.size = len(content)
                files.addfile(entry, io.BytesIO(content))
        completed = subprocess.CompletedProcess([], 0, stdout=archive.getvalue())
        with tempfile.TemporaryDirectory() as tmp, patch.object(runner.subprocess, 'run', return_value=completed) as run:
            data = Path(tmp)/'custom-data'
            for name, content in contents.items():
                path = data/name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(content)
            runner.validate_data(data)
            (data/'units/test.json').write_bytes(b'{"hp":9}')
            for option in [['--data-dir', str(data)], ['--data-dir=' + str(data)]]:
                run.reset_mock()
                with patch.object(sys, 'argv', ['run-combat.py', '--out', str(Path(tmp)/'out'), *option]):
                    with self.assertRaisesRegex(ValueError, 'differs from HEAD'):
                        runner.main()
                self.assertEqual(run.call_count, 1)
                self.assertEqual(run.call_args.args[0][:2], ['git', 'archive'])
                self.assertFalse((Path(tmp)/'out').exists())
            (data/'units/test.json').unlink()
            with self.assertRaisesRegex(ValueError, 'differs from HEAD'):
                runner.validate_data(data)

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
