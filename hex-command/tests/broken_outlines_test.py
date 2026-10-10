import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / '.agents/skills/pixel-cleaning/scripts'))
from check_broken_outlines import inspect


class BrokenOutlinesTest(unittest.TestCase):
    def test_all_annotated_jumper_regions_are_reported(self):
        rows = (ROOT / 'tests/fixtures/broken-outlines/jumper.txt').read_text().splitlines()
        findings = inspect(rows)
        regions = {
            'cloak': (0, 21, 11, 29),
            'hair hole and torn contour': (20, 34, 8, 19),
            'right arm': (35, 44, 22, 39),
            'knee': (23, 35, 34, 39),
            'left boot': (10, 22, 40, 51),
        }
        for label, (x0, x1, y0, y1) in regions.items():
            with self.subTest(region=label):
                self.assertTrue(any(x0 <= f['x'] <= x1 and y0 <= f['y'] <= y1
                                    for f in findings), label)
        self.assertTrue(any(f['kind'] == 'enclosed-hole' and 20 <= f['x'] <= 34
                            and 8 <= f['y'] <= 19 for f in findings))

    def test_warden_cloak_leaks_require_material_ownership(self):
        rows = (ROOT / 'tests/fixtures/broken-outlines/warden.txt').read_text().splitlines()
        ownership = json.loads((ROOT / 'tests/fixtures/broken-outlines/warden.ownership.json').read_text())
        findings = inspect(rows, ownership=ownership)
        leaks = {(f['x'], f['y']) for f in findings if f['kind'] == 'material-leak'}
        self.assertEqual(leaks, {(22, 41), (22, 42), (28, 46), (28, 47)})
        self.assertTrue(any(f['kind'] == 'thick-outline' for f in findings))

    def test_clean_box_has_no_broken_border(self):
        self.assertEqual(inspect(['.....', '.OOO.', '.OBO.', '.OOO.', '.....']), [])

    def test_diagonal_gap_and_exposed_material(self):
        findings = inspect(['O.B', '.O.', '...'])
        self.assertTrue(any(f['kind'] == 'diagonal-gap' for f in findings))
        self.assertTrue(any(f['kind'] == 'missing-border' and f['x'] == 2 for f in findings))


if __name__ == '__main__':
    unittest.main()
