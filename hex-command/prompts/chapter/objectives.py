#!/usr/bin/env python3
"""Sample objective trees by consuming shuffled weighted bags at every decision.

python prompts/chapter/objectives.py --count 10 --seed 42 --output tmp/objectives

START rolls a structure around an initial pivot. Each objective then rolls its
next structure, draws the next pivot and (for AND/OR) a matching partner. Both
branches walk independently to END. Groups contain two branches. END completes
the current branch; a completed group can then have one shared continuation,
sampled by multiplying its terminal objectives' successor probabilities. No source
campaign names, conditions or targets are copied into generated objectives.
Each finishing route draws a result from its terminal-objective distribution;
AND terminals are conditioned jointly, while OR alternatives keep their own results.
"""
import argparse
import json
import math
from pathlib import Path
import random

MODEL = json.loads(Path(__file__).with_name('objective_model.json').read_text())


class RejectedWalk(Exception):
    pass


class Bags:
    """Pre-shuffle every distribution; refill exhausted bags with the same law."""
    def __init__(self, model, rng, size):
        self.rng = rng
        self.size = size
        self.weights = {}
        self.bags = {}
        self.draws = {}
        for name in ('next_shapes', 'next_objectives', 'group_shapes'):
            for key, weights in model[name].items():
                self.weights[(name, key)] = weights
        for op, rows in model['partners'].items():
            for key, weights in rows.items():
                self.weights[(op, key)] = weights
        for row in model['terminal_results'].values():
            key = tuple(sorted(row['terminal_objectives']))
            self.weights[('terminal_results', key)] = row['result_counts']
        for key in self.weights:
            self.refill(key)

    def add(self, name, source, weights):
        """Shuffle a newly encountered terminal-combination bag before drawing."""
        key = (name, source)
        if key not in self.weights:
            self.weights[key] = weights
            self.refill(key)

    def refill(self, key):
        weights = self.weights[key]
        total = sum(weights.values())
        bag = []
        for value, weight in weights.items():
            expected = self.size * weight / total
            amount = math.floor(expected)
            amount += self.rng.random() < expected - amount
            bag.extend([value] * amount)
        if not bag:
            raise ValueError(f'Empty distribution: {key}')
        self.rng.shuffle(bag)
        self.bags[key] = bag

    def take(self, name, source):
        key = (name, source)
        if key not in self.weights:
            raise RejectedWalk(f'No {name} distribution for {source}')
        if not self.bags[key]:
            self.refill(key)
        self.draws[key] = self.draws.get(key, 0) + 1
        return self.bags[key].pop()


def terminals(node):
    """Last objectives on all branches, including nested groups/continuations."""
    if node['type'] == 'ATOM':
        return [node['objective']]
    if node['type'] == 'END':
        return []
    children = node['children']
    if node['type'] == 'TO':
        for child in reversed(children):
            result = terminals(child)
            if result:
                return result
        return []
    return [objective for child in children for objective in terminals(child)]


def shared_weights(objectives, successors):
    """Normalized product of P(next kind | terminal kind), matched by name.

    This is a compatibility heuristic, not a measured joint probability.
    Log weights avoid underflow for nested groups with many terminal operands.
    """
    rows = [successors.get(objective, {}) for objective in sorted(objectives)]
    if not rows or any(not row for row in rows):
        raise RejectedWalk('No common successor for group terminals')
    common = set(rows[0]).intersection(*rows[1:])
    logs = {}
    for candidate in sorted(common):
        if all(row[candidate] > 0 for row in rows):
            logs[candidate] = sum(math.log(row[candidate] / sum(row.values()))
                                  for row in rows)
    if not logs:
        raise RejectedWalk('No common successor for group terminals')
    peak = max(logs.values())
    weights = {candidate: math.exp(value - peak) for candidate, value in logs.items()}
    total = sum(weights.values())
    return {candidate: weight / total for candidate, weight in weights.items()}


def render(node):
    t = node['type']
    if t == 'ATOM':
        return node['objective']
    if t == 'END':
        return 'END'
    join = {'TO': ' → ', 'AND': ' AND ', 'OR': ' OR '}[t]
    children = [child for child in node['children']
                if not (t == 'TO' and child['type'] == 'END')]
    text = join.join(render(child) for child in children)
    return '(' + text + ')' if t in ('AND', 'OR') else text


def ending_cases(node, path='root', max_cases=128):
    """Terminal combinations and OR choices for each distinct completion route."""
    t = node['type']
    if t == 'END':
        return []
    if t == 'ATOM':
        return [{'terminal_objectives': [node['objective']],
                 'terminal_paths': [path], 'choices': []}]
    if t == 'TO':
        for i in range(len(node['children']) - 1, -1, -1):
            cases = ending_cases(node['children'][i], f'{path}.children[{i}]', max_cases)
            if cases:
                return cases
        return []
    cases = [] if t == 'OR' else [
        {'terminal_objectives': [], 'terminal_paths': [], 'choices': []}]
    for i, child in enumerate(node['children']):
        branch = ending_cases(child, f'{path}.children[{i}]', max_cases)
        if t == 'OR':
            for case in branch:
                case['choices'].insert(0, {'path': path, 'branch': i + 1})
            cases.extend(branch)
        elif t == 'AND':
            if len(cases) * len(branch) > max_cases:
                raise RejectedWalk('Too many terminal result combinations')
            cases = [{key: a[key] + b[key] for key in a}
                     for a in cases for b in branch]
        else:
            raise ValueError(f'Unknown generated flow type: {t}')
        if len(cases) > max_cases:
            raise RejectedWalk('Too many terminal result combinations')
    return cases


def render_outcomes(outcomes):
    def label(kind):
        return 'Gain forces / allies' if kind == 'Forces / allies' else kind
    if len(outcomes) == 1:
        return '**Result:** ' + label(outcomes[0]['result']) + '.'
    lines = ['**Results by completion route:**', '']
    for i, outcome in enumerate(outcomes, 1):
        terminals = ' AND '.join(outcome['terminal_objectives'])
        lines.append(f"- Route {i} ({terminals}): {label(outcome['result'])}.")
    return '\n'.join(lines)


def sample(count, seed, model=MODEL, max_nodes=128, max_depth=32, max_attempts=100):
    if min(count, max_nodes, max_depth, max_attempts) < 1:
        raise ValueError('Counts and walk limits must be positive')
    rng = random.Random(seed)
    bags = Bags(model, rng, max(256, count * 32))
    outputs = []
    rejections = {}
    for _ in range(count):
        for attempt in range(max_attempts):
            budget = [max_nodes]
            trace = []

            def take(name, source):
                value = bags.take(name, source)
                trace.append({'distribution': name, 'source': source, 'value': value})
                return value

            def step(depth):
                budget[0] -= 1
                if budget[0] < 0 or depth > max_depth:
                    raise RejectedWalk('Walk exceeded node/depth limit')

            def atom(objective, depth):
                step(depth)
                next_shape = take('next_shapes', objective)
                current = {'type': 'ATOM', 'objective': objective}
                if next_shape == 'END':
                    rest = {'type': 'END'}
                else:
                    pivot = take('next_objectives', objective)
                    rest = structure(next_shape, pivot, depth + 1)
                return {'type': 'TO', 'children': [current, rest]}

            def structure(shape, pivot, depth):
                step(depth)
                if shape == 'END':
                    return {'type': 'END'}
                if shape == 'ATOM':
                    return atom(pivot, depth + 1)
                if shape not in ('AND', 'OR'):
                    raise ValueError(f'Unknown structure {shape}')
                partner = take(shape, pivot)
                group = {'type': shape, 'children': [atom(pivot, depth + 1),
                                                     atom(partner, depth + 1)]}
                next_shape = take('group_shapes', shape)
                if next_shape == 'END':
                    return group
                terminal_kinds = tuple(sorted(terminals(group)))
                weights = shared_weights(terminal_kinds, model['next_objectives'])
                bags.add('shared_next', terminal_kinds, weights)
                next_pivot = take('shared_next', terminal_kinds)
                trace[-1]['weights'] = weights
                # Append once outside the group, never once per branch.
                rest = structure(next_shape, next_pivot, depth + 1)
                return {'type': 'TO', 'children': [group, rest]}

            try:
                pivot = take('next_objectives', 'START')
                shape = take('next_shapes', 'START')
                tree = structure(shape, pivot, 0)
                outcomes = ending_cases(tree, max_cases=max_nodes)
                if not outcomes:
                    raise RejectedWalk('No terminal objective for result sampling')
                for outcome in outcomes:
                    context = tuple(sorted(outcome['terminal_objectives']))
                    outcome['terminal_objectives'] = list(context)
                    outcome['result'] = take('terminal_results', context)
            except RejectedWalk as error:
                reason = str(error)
                rejections[reason] = rejections.get(reason, 0) + 1
                continue
            outputs.append({'id': len(outputs) + 1, 'flow': tree,
                            'expression': render(tree), 'outcomes': outcomes, 'draws': trace})
            break
        else:
            raise ValueError(f'Could not complete walk after {max_attempts} attempts; {rejections}')
    return {'seed': seed, 'count': count, 'method': 'Pre-shuffled weighted bags for every draw; recursive walks to END.',
            'scope': model['scope'], 'group_size': 2,
            'limitations': ['Independent conditional tables can recombine unobserved structures.',
                            'Missing partner, continuation or terminal-result rows and excessive walks reject the whole attempt; no invented END or unrelated-result fallback.',
                            'Rejection can alter the accepted distribution; rejection counts are reported.',
                            'Shared continuation weights multiply terminal successor probabilities as a compatibility heuristic, not a measured joint distribution.'],
            'rejected_attempts': rejections, 'objectives': outputs}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=10)
    parser.add_argument('--seed', type=int)
    parser.add_argument('--output', type=Path, help='New directory for sample.json and objectives.md')
    args = parser.parse_args()
    if args.output and args.output.exists():
        parser.error('--output must be a new directory')
    seed = args.seed if args.seed is not None else random.SystemRandom().getrandbits(64)
    try:
        result = sample(args.count, seed)
    except ValueError as error:
        parser.error(str(error))
    serialized = json.dumps(result, ensure_ascii=False, indent=2) + '\n'
    if args.output:
        args.output.mkdir(parents=True)
        (args.output / 'sample.json').write_text(serialized)
        (args.output / 'objectives.md').write_text('\n\n'.join(
            f"### {r['id']}. {r['expression']}\n\n{render_outcomes(r['outcomes'])}"
            for r in result['objectives']) + '\n')
        print(args.output / 'objectives.md')
    else:
        print(serialized, end='')


if __name__ == '__main__':
    main()
