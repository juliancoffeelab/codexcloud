import importlib.util
import csv
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

spec = importlib.util.spec_from_file_location('report_combat', Path(__file__).parents[1] / 'scripts/report-combat.py')
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


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
