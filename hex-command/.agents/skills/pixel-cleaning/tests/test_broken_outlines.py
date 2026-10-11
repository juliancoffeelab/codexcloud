import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / '.agents/skills/pixel-cleaning/scripts'))
from check_broken_outlines import inspect


class BrokenOutlinesTest(unittest.TestCase):
    def test_clean_box_has_no_broken_border(self):
        self.assertEqual(inspect(['.....', '.OOO.', '.OBO.', '.OOO.', '.....']), [])

    def test_diagonal_gap_and_exposed_material(self):
        findings = inspect(['O.B', '.O.', '...'])
        self.assertTrue(any(f['kind'] == 'diagonal-gap' for f in findings))
        self.assertTrue(any(f['kind'] == 'missing-border' and f['x'] == 2 for f in findings))

    def test_holes_include_one_to_three_pixels_but_not_limb_gaps(self):
        for size in (1, 2, 3, 4, 12):
            with self.subTest(size=size):
                width = size + 4
                rows = ['.' * width, '.' + 'O' * (size + 2) + '.',
                        '.O' + '.' * size + 'O.',
                        '.' + 'O' * (size + 2) + '.', '.' * width]
                holes = [f for f in inspect(rows) if f['kind'] == 'enclosed-hole']
                self.assertEqual(len(holes), size if size <= 3 else 0)
                self.assertTrue(all(f['component_size'] == size for f in holes))

    def test_open_gap_is_not_an_enclosed_hole(self):
        self.assertFalse(any(f['kind'] == 'enclosed-hole'
                             for f in inspect(['.....', '.OOO.', '.O...', '.OOO.', '.....'])))

    def test_floating_black_pair_is_highlighted_separately(self):
        findings = inspect(['.......', '.OB.O..', '.OO.O..', '.......'])
        floating = [f for f in findings if f['kind'] == 'detached-outline']
        self.assertEqual({(f['x'], f['y']) for f in floating}, {(4, 1), (4, 2)})
        self.assertTrue(all(f['component_size'] == 2 for f in floating))

    def test_diagonal_outline_attached_to_material_is_not_floating(self):
        self.assertFalse(any(f['kind'] == 'detached-outline'
                             for f in inspect(['O...', '.O..', '..B.', '....'])))

    def test_fur_in_owned_thorn_is_material_leak(self):
        findings = inspect(['OLHO'], ownership={
            'mask': ['?tt?'], 'allowed': {'t': ['L', 'l', 'O']},
        })
        self.assertEqual({(f['x'], f['y']) for f in findings if f['kind'] == 'material-leak'}, {(2, 0)})


if __name__ == '__main__':
    unittest.main()
