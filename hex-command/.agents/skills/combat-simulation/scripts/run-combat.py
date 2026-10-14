"""Run an authorized native simulation and generate its complete data report."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=Path, help='Directory for raw results and complete report')
    parser.add_argument('--baseline', action='append', default=[], type=Path)
    parser.add_argument('--all-pairs', action='store_true',
                        help='Run the full arena policy matrix instead of players versus Classic')
    parser.add_argument('--data-dir', type=Path, default=ROOT / 'rust/data',
                        help='Authored catalog directory for the native simulation')
    args, simulation_options = parser.parse_known_args()
    if args.all_pairs and '--vs-classic' in simulation_options:
        parser.error('--all-pairs cannot be combined with --vs-classic')
    data_dir = args.data_dir.resolve()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    for flag, default in [('--mode', 'full'), ('--seeds', '10'),
                          ('--policies', 'classic,rush,mixed,priority,battler'),
                          ('--details', 'jsonl')]:
        if flag not in simulation_options:
            simulation_options += [flag, default]
    mode = simulation_options[simulation_options.index('--mode') + 1]
    if mode != 'campaign' and not args.all_pairs and '--self-play' not in simulation_options:
        if '--vs-classic' not in simulation_options:
            simulation_options += ['--vs-classic']
    if mode != 'arena':
        selected = simulation_options[simulation_options.index('--policies') + 1].split(',')
        players = [p for p in selected if p in ('mixed', 'priority')]
        for flag, default in [('--campaign-players', ','.join(players)),
                              ('--campaign-enemies', 'classic')]:
            if flag not in simulation_options:
                if not default:
                    parser.error('No simulation-player campaign tactics selected; use --mode arena or explicit --campaign-players.')
                simulation_options += [flag, default]
    simulation_options += ['--data-dir', str(data_dir)]
    print('Running complete native batch; progress and raw output: ' + str(out / 'native.log'), flush=True)
    with (out / 'native.log').open('w') as log:
        subprocess.run(['npm', 'run', 'ai:simulate', '--', *simulation_options, '--out', str(out / 'report.json')],
                       cwd=ROOT, stdout=log, stderr=log, check=True)
    saved = json.loads((out / 'report.json').read_text())
    print(f"Native batch complete: {len(saved['outcomes']):,} matches in {saved['elapsedSeconds']:.2f}s. Analyzing recorded results.", flush=True)
    report = [sys.executable, str(Path(__file__).with_name('report-combat.py')), str(out / 'report.json'), '--out', str(out / 'analytics')]
    for baseline in args.baseline:
        report += ['--baseline', str(baseline.resolve())]
    subprocess.run(report, cwd=ROOT, check=True)


if __name__ == '__main__':
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.exit(error.returncode)
    except (ValueError, OSError) as error:
        sys.exit(f'Cannot run combat batch: {error}')
