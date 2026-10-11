import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('report_combat', Path(__file__).parents[1] / 'scripts/report-combat.py')
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


def match(seed, player='a', enemy='b', winner='player', reason='leader'):
    return {'kind': 'arena', 'config': {'seed': seed, 'armies': {'player': player, 'enemy': enemy},
            'player': 'mixed', 'enemy': 'mixed', 'landscape': 'open', 'first': 'player',
            'maxRounds': 30, 'recruitLimit': 1}, 'winner': winner, 'reason': reason, 'rounds': 10,
            'recruits': [], 'casualties': [], 'promotions': [], 'stateHash': str(seed)}


class CompleteDataTests(unittest.TestCase):
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
