"""Run an authorized native simulation and generate its complete data report."""
import argparse
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=Path, help='Directory for raw results and complete report')
    parser.add_argument('--baseline', action='append', default=[], type=Path)
    args, simulation_options = parser.parse_known_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    subprocess.run(['npm', 'run', 'ai:simulate', '--', *simulation_options, '--out', str(out / 'report.json')], cwd=ROOT, check=True)
    report = [sys.executable, str(Path(__file__).with_name('report-combat.py')), str(out / 'report.json'), '--out', str(out / 'analytics')]
    for baseline in args.baseline:
        report += ['--baseline', str(baseline.resolve())]
    subprocess.run(report, cwd=ROOT, check=True)


if __name__ == '__main__':
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.exit(error.returncode)
