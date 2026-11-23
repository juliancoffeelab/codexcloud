from collections import Counter, defaultdict
import json
from pathlib import Path

from build import OUT, catalog_hash, read, write
from run import hash_file


def main():
    tag = 'fork-vigil'
    case = read(OUT / 'pressure-variants.json')[tag]
    evaluation = read(OUT / 'pressure-confirmation-summary.json')
    evidence = read(OUT / 'pressure-confirmation-trace-summary.json')
    freeze = read(OUT / 'pressure-freeze.json')
    assert evaluation['catalogSha256'] == evidence['catalogSha256'] == freeze['catalogSha256']
    assert evaluation['catalogSha256'] == catalog_hash(Path(evaluation['catalog']))
    assert evaluation['seeds'] == freeze['confirmationSeeds']
    outcomes = [r for r in evaluation['outcomes'] if r['variant'] == tag]
    replays = [r for r in evidence['aggregate'] if r['variant'] == tag]
    assert sum(row['replays'] for row in replays) == 100
    assert all(row['wins'] + row['losses'] + row['caps'] == 20 for row in outcomes)
    traces = defaultdict(dict)
    with Path(evidence['objectiveTrajectories']).open() as stream:
        for line in stream:
            row = json.loads(line)
            source = row['source']
            if source['variant'] == tag:
                traces[source['config']['seed']][source['policy']] = row
    assert sorted(traces) == evaluation['seeds']
    pairing = {}
    for policy in ('hour8', 'hour10'):
        comparisons = Counter()
        for attempt in traces.values():
            reference = attempt['classic']['source']['result']['success']
            learner = attempt[policy]['source']['result']['success']
            label = ('classicWin' if reference else 'classicLoss') + ('LearnerWin' if learner else 'LearnerLoss')
            comparisons[label] += 1
        pairing[policy] = comparisons
    example_seed = 571239002
    example = {policy: traces[example_seed][policy] for policy in ('classic', 'hour8', 'hour10')}
    example_path = OUT / 'matched-example-571239002.json'
    write(example_path, example)
    stage = Path(case['stage'])
    stage_files = {path.stem: dict(path=str(path), sha256=hash_file(path))
                   for path in sorted(stage.glob('*.json'))}
    render_dir = OUT / 'render'
    render_files = {path.name: dict(path=str(path), sha256=hash_file(path))
                    for path in sorted(render_dir.iterdir()) if path.is_file()}
    assert (render_dir / 'preview.png').is_file()
    raw_path = Path(evaluation['raw'])
    final = dict(
        title='Ashbank Relay',
        status='validated candidate',
        case=case,
        stageFiles=stage_files,
        render=render_files,
        summaryPath=str(OUT / 'REPORT.md'),
        catalog=evaluation['catalog'],
        catalogSha256=evaluation['catalogSha256'],
        native=evaluation['native'],
        nativeSha256=evaluation['nativeSha256'],
        models=evaluation['models'],
        confirmationSeeds=evaluation['seeds'],
        discoverySeeds=read(OUT / 'discovery-pressure-summary.json')['seeds'],
        fixedOpponent='classic',
        visibility='Full for both sides',
        maxRounds=100,
        recruitLimit=0,
        outcomes=outcomes,
        replayEvidence=replays,
        matchedPairsAgainstClassic=pairing,
        matchedExample=dict(seed=example_seed, path=str(example_path)),
        rawConfirmation=dict(path=str(raw_path), sha256=hash_file(raw_path)),
        traceMetadata=str(OUT / 'pressure-confirmation-trace-summary.json'),
        objectiveTrajectories=evidence['objectiveTrajectories'],
        rawReplays=evidence['raw'],
        comparisonPanels=[
            dict(label='Early timing-only test', variants=['breach-west-swift', 'breach-two-fronts'],
                 summary=str(OUT / 'confirmation-summary.json'), trace=str(OUT / 'confirmation-trace-summary.json')),
            dict(label='Contested ownership comparisons', variants=['fork-veteran', 'fork-eastern'],
                 summary=str(OUT / 'pressure-confirmation-summary.json'),
                 trace=str(OUT / 'pressure-confirmation-trace-summary.json')),
        ],
        discoverySummaries=[str(OUT / f'{label}-summary.json') for label in
                            ('discovery', 'discovery-long', 'discovery-breach', 'discovery-pressure')],
    )
    write(OUT / 'final-summary.json', final)


if __name__ == '__main__':
    main()
