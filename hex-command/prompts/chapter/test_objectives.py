"""Small deterministic checks for shared objective-group continuations."""
import copy
import unittest

import objectives
import results


def fixture(operator):
    return {
        'scope': 'fixture',
        'next_shapes': {
            'START': {operator: 1},
            'A': {'END': 1}, 'B': {'END': 1}, 'Shared': {'END': 1},
        },
        'next_objectives': {
            'START': {'A': 1},
            'A': {'Shared': 3, 'Only A': 1},
            'B': {'Shared': 1, 'Only B': 1},
        },
        'group_shapes': {operator: {'ATOM': 1}},
        'partners': {operator: {'A': {'B': 1}, 'B': {'A': 1}}},
        'terminal_results': {
            name: {'terminal_objectives': [name], 'result_counts': {name + ' outcome': 1}}
            for name in ('A', 'B', 'Shared')
        },
    }


class SharedContinuationTests(unittest.TestCase):
    def test_both_operators_append_one_shared_objective(self):
        for operator in ('AND', 'OR'):
            with self.subTest(operator=operator):
                row = objectives.sample(1, 42, fixture(operator))['objectives'][0]
                self.assertEqual(row['expression'], f'(A {operator} B) → Shared')
                self.assertEqual(row['flow']['type'], 'TO')
                self.assertEqual(row['flow']['children'][0]['type'], operator)
                shared = [draw for draw in row['draws']
                          if draw['distribution'] == 'shared_next']
                self.assertEqual(len(shared), 1)
                self.assertEqual(shared[0]['weights'], {'Shared': 1.0})

    def test_products_match_names_and_are_operand_order_independent(self):
        successors = {'A': {'X': 3, 'Y': 1}, 'B': {'Y': 4, 'X': 1}}
        weights = objectives.shared_weights(('A', 'B'), successors)
        self.assertAlmostEqual(weights['X'], 3 / 7)
        self.assertAlmostEqual(weights['Y'], 4 / 7)
        self.assertEqual(weights, objectives.shared_weights(('B', 'A'), successors))

    def test_no_overlap_rejects_instead_of_inventing_an_end(self):
        model = fixture('AND')
        model['next_objectives']['B'] = {'Unrelated': 1}
        with self.assertRaisesRegex(ValueError, 'No common successor'):
            objectives.sample(1, 42, model, max_attempts=3)

    def test_group_end_does_not_require_a_successor(self):
        model = fixture('OR')
        model['group_shapes']['OR'] = {'END': 1}
        model['next_objectives'].pop('A')
        model['next_objectives'].pop('B')
        row = objectives.sample(1, 42, model)['objectives'][0]
        self.assertEqual(row['expression'], '(A OR B)')
        self.assertFalse(any(d['distribution'] == 'shared_next' for d in row['draws']))

    def test_uses_actual_branch_terminals(self):
        model = fixture('AND')
        model['next_shapes'].update({'A': {'ATOM': 1}, 'A2': {'END': 1}})
        model['next_objectives']['A'] = {'A2': 1}
        model['next_objectives']['A2'] = {'Shared': 1}
        row = objectives.sample(1, 42, model)['objectives'][0]
        self.assertEqual(row['expression'], '(A → A2 AND B) → Shared')
        draw = next(d for d in row['draws'] if d['distribution'] == 'shared_next')
        self.assertEqual(draw['source'], ('A2', 'B'))

    def test_nested_terminal_collection_and_replay(self):
        def atom(name):
            return {'type': 'ATOM', 'objective': name}
        tree = {'type': 'TO', 'children': [
            {'type': 'AND', 'children': [atom('A'), atom('B')]},
            {'type': 'OR', 'children': [atom('C'), atom('D')]},
            {'type': 'END'},
        ]}
        self.assertEqual(objectives.terminals(tree), ['C', 'D'])
        model = fixture('AND')
        original = copy.deepcopy(model)
        self.assertEqual(objectives.sample(10, 42, model), objectives.sample(10, 42, model))
        self.assertEqual(model, original)


class ResultBindingTests(unittest.TestCase):
    @staticmethod
    def task(id, kind, refs=()):
        return {'type': 'task', 'id': id, 'kind': kind, 'result_ids': list(refs)}

    @staticmethod
    def data(flow):
        return {'results': {
            'r1': {'chapter_id': 'chapter', 'result_kind': 'First outcome'},
            'r2': {'chapter_id': 'chapter', 'result_kind': 'Second outcome'},
        }, 'chapters': [{'chapter_id': 'chapter', 'sampling_exclusion': None, 'flow': flow}]}

    def test_alternatives_keep_their_own_results(self):
        flow = {'type': 'any', 'children': [self.task('a', 'A', ['r1']),
                                           self.task('b', 'B', ['r2'])]}
        report = results.derive(self.data(flow))
        self.assertEqual(report['contexts']['A']['result_counts'], {'First outcome': 1})
        self.assertEqual(report['contexts']['B']['result_counts'], {'Second outcome': 1})

    def test_conjunction_is_one_joint_context(self):
        flow = {'type': 'all', 'result_ids': ['r1'],
                'children': [self.task('a', 'A'), self.task('b', 'B')]}
        data = self.data(flow)
        data['results'].pop('r2')
        report = results.derive(data)
        self.assertEqual(set(report['contexts']), {'AND: A + B'})

    def test_only_the_last_step_receives_the_result(self):
        flow = {'type': 'sequence', 'result_ids': ['r1'],
                'children': [self.task('a', 'A'), self.task('b', 'B')]}
        data = self.data(flow)
        data['results'].pop('r2')
        self.assertEqual(set(results.derive(data)['contexts']), {'B'})

    def test_missing_and_conflicting_bindings_fail(self):
        for refs in ([], ['r1', 'r2']):
            with self.subTest(refs=refs):
                with self.assertRaisesRegex(ValueError, 'result bindings'):
                    results.derive(self.data(self.task('a', 'A', refs)))

    def test_cross_chapter_and_unbound_results_fail(self):
        data = self.data(self.task('a', 'A', ['r1']))
        data['results']['r1']['chapter_id'] = 'elsewhere'
        with self.assertRaisesRegex(ValueError, 'cross-chapter'):
            results.derive(data)
        data['results']['r1']['chapter_id'] = 'chapter'
        with self.assertRaisesRegex(ValueError, 'Unbound result'):
            results.derive(data)


class ResultSamplingTests(unittest.TestCase):
    def test_or_routes_draw_their_own_results(self):
        model = fixture('OR')
        model['group_shapes']['OR'] = {'END': 1}
        row = objectives.sample(1, 42, model)['objectives'][0]
        self.assertEqual([o['result'] for o in row['outcomes']], ['A outcome', 'B outcome'])
        self.assertEqual([o['choices'][0]['branch'] for o in row['outcomes']], [1, 2])

    def test_and_draws_one_joint_result(self):
        model = fixture('AND')
        model['group_shapes']['AND'] = {'END': 1}
        model['terminal_results']['joint'] = {
            'terminal_objectives': ['B', 'A'], 'result_counts': {'Joint outcome': 1}}
        row = objectives.sample(1, 42, model)['objectives'][0]
        self.assertEqual(len(row['outcomes']), 1)
        self.assertEqual(row['outcomes'][0]['result'], 'Joint outcome')
        self.assertEqual(row['outcomes'][0]['terminal_objectives'], ['A', 'B'])

    def test_shared_successor_is_the_only_result_context(self):
        row = objectives.sample(1, 42, fixture('AND'))['objectives'][0]
        self.assertEqual(len(row['outcomes']), 1)
        self.assertEqual(row['outcomes'][0]['terminal_objectives'], ['Shared'])
        self.assertEqual(row['outcomes'][0]['choices'], [])

    def test_missing_joint_data_never_falls_back_to_individual_results(self):
        model = fixture('AND')
        model['group_shapes']['AND'] = {'END': 1}
        with self.assertRaisesRegex(ValueError, 'No terminal_results distribution'):
            objectives.sample(1, 42, model, max_attempts=3)

    def test_nested_and_or_has_one_context_per_combination(self):
        def atom(name):
            return {'type': 'ATOM', 'objective': name}
        tree = {'type': 'AND', 'children': [
            {'type': 'OR', 'children': [atom('A'), atom('B')]}, atom('C')]}
        cases = objectives.ending_cases(tree)
        self.assertEqual([c['terminal_objectives'] for c in cases], [['A', 'C'], ['B', 'C']])
        with self.assertRaisesRegex(objectives.RejectedWalk, 'Too many'):
            objectives.ending_cases(tree, max_cases=1)


if __name__ == '__main__':
    unittest.main()
