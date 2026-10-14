"""Run an authorized native simulation and generate its complete data report."""
import argparse
import json
import os
from datetime import datetime, timezone
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]

def simulation_binary():
    target = Path(os.environ.get('CARGO_TARGET_DIR', 'rust/target'))
    name = 'hex-command-sim.exe' if os.name == 'nt' else 'hex-command-sim'
    return ROOT / target / 'release' / name


def main():
    parser = argparse.ArgumentParser(description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter, epilog='''Simulation options (forwarded to the Rust engine):
  --mode arena|campaign|full   Workload (default full)
  --seeds N --seed N           Seed count (default 10) and first seed (default 1)
  --policies LIST              Comma-separated arena player policies (default all five)
  --armies LIST --landscapes LIST --campaigns LIST
                              Select factions, landscapes or campaigns (default all)
  --campaign-players LIST      Campaign players (default selected Mixed/Priority)
  --campaign-enemies LIST      Campaign opponents (default Classic)
  --diplomacy LIST             Campaign diplomacy settings
  --rounds N --recruits N      Override round cap and recruitment limit
  --workers N --profile-turns  Worker count and optional policy CPU measurements
  --self-play                 Matching-policy comparisons (requires --mode arena)
  --details arrow|jsonl        Raw gameplay format (default jsonl)
  --schedule-only             Print planned matches without running or reporting
  --replay PATH               Verify an existing run without generating a report''')
    parser.add_argument('--out', type=Path, help='Output directory (default a fresh artifacts/simulation-* directory); a .json path selects the raw report file')
    parser.add_argument('--no-report', action='store_true', help='Run simulations and retain raw results without analyzing them')
    parser.add_argument('--baseline', action='append', default=[], type=Path)
    parser.add_argument('--all-pairs', action='store_true',
                        help='Run the full arena policy matrix instead of players versus Classic')
    parser.add_argument('--data-dir', type=Path, default=ROOT / 'rust/data',
                        help='Authored catalog directory for the native simulation')
    args, simulation_options = parser.parse_known_args()
    if args.no_report and args.baseline:
        parser.error('--baseline requires report generation; remove --no-report')
    if args.all_pairs and '--vs-classic' in simulation_options:
        parser.error('--all-pairs cannot be combined with --vs-classic')
    data_dir = args.data_dir.resolve()
    binary = str(simulation_binary())
    if any(flag in simulation_options for flag in ['--stdio', '--phase-stdio', '--fingerprint', '--replay']):
        subprocess.run([binary, *simulation_options, '--data-dir', str(data_dir)], cwd=ROOT, check=True)
        return
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
    if '--schedule-only' in simulation_options:
        subprocess.run([binary, *simulation_options], cwd=ROOT, check=True)
        return
    if args.no_report:
        simulation_options += ['--no-report']
    out = (args.out or ROOT / 'artifacts' / ('simulation-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))).resolve()
    report_path = out if out.suffix == '.json' else out / 'report.json'
    out = report_path.parent
    out.mkdir(parents=True, exist_ok=True)
    print('Running complete native batch; progress and raw output: ' + str(out / 'native.log'), flush=True)
    with (out / 'native.log').open('w') as log:
        subprocess.run([binary, *simulation_options, '--out', str(report_path)],
                       cwd=ROOT, stdout=log, stderr=log, check=True)
    saved = json.loads(report_path.read_text())
    print(f"Simulation complete: {len(saved['outcomes']):,} matches in {saved['elapsedSeconds']:.2f}s.", flush=True)
    if args.no_report:
        print('Raw results: ' + str(report_path), flush=True)
        return
    report = [sys.executable, str(Path(__file__).with_name('report-combat.py')), str(report_path), '--out', str(out / 'analytics')]
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
