"""Run an authorized native simulation and generate its complete data report."""
import argparse
import io
from pathlib import Path
import subprocess
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[4]


def validate_data(data_dir):
    prefix = ROOT.name + '/rust/data/'
    archive = subprocess.run(['git', 'archive', 'HEAD', '--', prefix.rstrip('/')],
                             cwd=ROOT.parent, capture_output=True, check=True)
    with tarfile.open(fileobj=io.BytesIO(archive.stdout)) as files:
        expected = {m.name.removeprefix(prefix): files.extractfile(m).read()
                    for m in files if m.isfile() and m.name.startswith(prefix)}
    if not expected:
        raise ValueError('No committed simulation data found')
    for name, content in expected.items():
        path = data_dir / name
        if not path.is_file() or path.read_bytes() != content:
            raise ValueError(f'Simulation data differs from HEAD: {path}. '
                             'Commit the data and run ai:combat from that checkout before reporting it.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=Path, help='Directory for raw results and complete report')
    parser.add_argument('--baseline', action='append', default=[], type=Path)
    parser.add_argument('--data-dir', type=Path, default=ROOT / 'rust/data',
                        help='Catalog directory; must match this checkout\'s committed data')
    args, simulation_options = parser.parse_known_args()
    data_dir = args.data_dir.resolve()
    validate_data(data_dir)
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    simulation_options += ['--data-dir', str(data_dir)]
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
    except (ValueError, OSError) as error:
        sys.exit(f'Cannot run combat batch: {error}')
