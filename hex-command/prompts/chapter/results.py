#!/usr/bin/env python3
"""Validate completion bindings and derive P(result | terminal objectives).

python prompts/chapter/results.py --output tmp/objective-results

Reads the linked source records beside this file. It does not match completion
prose at runtime. AND completions contribute a joint terminal context; OR paths
contribute their own contexts. Intermediate tasks never receive ending results.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path

SOURCE = Path(__file__).with_name('objective_results.json')


def completion_cases(node):
    """Enumerate successful structural paths with terminal IDs and result refs."""
    t = node['type']
    own = set(node.get('result_ids', []))
    if t == 'task':
        return [{'terminals': [node['id']], 'results': own, 'conditions': []}]
    if t == 'empty':
        return [{'terminals': [], 'results': own, 'conditions': []}]
    if t == 'condition':
        cases = completion_cases(node['child'])
        for case in cases:
            case['results'].update(own)
            case['conditions'].append(node['condition'])
        return cases
    children = [completion_cases(child) for child in node['children']]
    if t == 'any':
        cases = [case for branch in children for case in branch]
        for case in cases:
            case['results'].update(own)
        return cases
    if t not in ('all', 'sequence'):
        raise ValueError(f'Unknown source flow type: {t}')
    cases = [{'terminals': [], 'results': own, 'conditions': []}]
    for branch in children:
        joined = []
        for a in cases:
            for b in branch:
                terminals = (a['terminals'] + b['terminals'] if t == 'all'
                             else b['terminals'] or a['terminals'])
                joined.append({'terminals': terminals,
                               'results': a['results'] | b['results'],
                               'conditions': a['conditions'] + b['conditions']})
        cases = joined
    return cases


def task_index(flow):
    tasks = {}

    def visit(node):
        if node['type'] == 'task':
            if node['id'] in tasks:
                raise ValueError(f"Duplicate task ID: {node['id']}")
            tasks[node['id']] = node
        for child in node.get('children', []):
            visit(child)
        if 'child' in node:
            visit(node['child'])
    visit(flow)
    return tasks


def derive(data):
    results = data['results']
    chapters = data['chapters']
    if len({c['chapter_id'] for c in chapters}) != len(chapters):
        raise ValueError('Duplicate chapter IDs')
    used_results = set()
    chapter_cases = []
    counters = defaultdict(Counter)
    context_objectives = {}
    context_evidence = defaultdict(list)
    seen = set()
    included_results = set()
    for chapter in chapters:
        cid = chapter['chapter_id']
        tasks = task_index(chapter['flow'])
        for case in completion_cases(chapter['flow']):
            refs = case['results']
            if len(refs) != 1:
                raise ValueError(f'{cid}: completion has {len(refs)} result bindings: {sorted(refs)}')
            rid = next(iter(refs))
            if rid not in results or results[rid]['chapter_id'] != cid:
                raise ValueError(f'{cid}: invalid/cross-chapter result reference {rid}')
            used_results.add(rid)
            terminal_kinds = sorted(tasks[id]['kind'] for id in case['terminals'])
            if not terminal_kinds:
                raise ValueError(f'{cid}: result {rid} has no terminal objective')
            kind = 'ATOM' if len(terminal_kinds) == 1 else 'AND'
            context = (terminal_kinds[0] if kind == 'ATOM'
                       else 'AND: ' + ' + '.join(terminal_kinds))
            record = {'chapter_id': cid, 'result_id': rid,
                      'terminal_task_ids': case['terminals'],
                      'terminal_objectives': terminal_kinds,
                      'context': context, 'conditions': case['conditions'],
                      'sampling_exclusion': chapter['sampling_exclusion']}
            chapter_cases.append(record)
            if chapter['sampling_exclusion']:
                continue
            included_results.add(rid)
            # Several paths through earlier choices can finish with the same
            # objective kinds/result. Count that source result/context once.
            key = (rid, tuple(terminal_kinds))
            if key in seen:
                continue
            seen.add(key)
            result_kind = results[rid]['result_kind']
            counters[context][result_kind] += 1
            context_objectives[context] = terminal_kinds
            context_evidence[context].append(record)
    missing = set(results) - used_results
    if missing:
        raise ValueError(f'Unbound result records: {sorted(missing)}')
    contexts = {}
    for context, counts in sorted(counters.items()):
        total = sum(counts.values())
        contexts[context] = {
            'terminal_objectives': context_objectives[context],
            'count': total, 'result_counts': dict(counts),
            'probabilities': {kind: n / total for kind, n in counts.items()},
            'evidence': context_evidence[context],
        }
    return {
        'scope': {'chapters': len(chapters), 'result_records': len(results),
                  'included_result_records': len(included_results),
                  'successful_structural_paths': len(chapter_cases),
                  'result_terminal_contexts': len(seen)},
        'method': 'Each successful source path must resolve to exactly one same-chapter result. Count each distinct (source result, terminal-kind combination) once. AND terminals stay joint; OR paths are distinct. Bonus/training records are validated but excluded from probabilities.',
        'contexts': contexts, 'completion_cases': chapter_cases,
    }


def render(report):
    lines = ['# Results conditioned on finishing objectives', '',
             'Counts describe source result/terminal combinations, not measured player decisions. AND rows require every listed terminal. Earlier intermediate tasks do not receive the ending result.', '']
    for context, row in sorted(report['contexts'].items(), key=lambda kv: -kv[1]['count']):
        lines.extend(['## ' + context, '', f"n = {row['count']}", '',
                      '| Result | Count | Probability |', '|---|---:|---:|'])
        for kind, n in sorted(row['result_counts'].items(), key=lambda kv: -kv[1]):
            lines.append(f"| {kind} | {n} | {row['probabilities'][kind]:.1%} |")
        lines.append('')
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='New output directory')
    args = parser.parse_args()
    if args.output and args.output.exists():
        parser.error('--output must be a new directory')
    report = derive(json.loads(SOURCE.read_text()))
    if args.output:
        args.output.mkdir(parents=True)
        (args.output / 'conditional_results.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
        (args.output / 'conditional_results.md').write_text(render(report) + '\n')
        print(json.dumps(report['scope']))
        print(args.output / 'conditional_results.md')
    else:
        print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
