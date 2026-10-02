"""Compare a production build to the last JavaScript release without network access."""
import argparse
from pathlib import Path
import subprocess
import sys
import tarfile
from tempfile import TemporaryDirectory

root = Path(__file__).resolve().parents[1]
repository = Path(subprocess.check_output(
    ['git', 'rev-parse', '--show-toplevel'], cwd=root, text=True).strip())
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--baseline-ref', default='2297aa1', help='Original JavaScript release commit')
parser.add_argument('--output', type=Path, default=Path('/tmp/hex-command-parity'))
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
with TemporaryDirectory(prefix='hex-command-original-') as temporary:
    original = Path(temporary)
    archive = original / 'original.tar'
    with archive.open('wb') as target:
        subprocess.run(['git', 'archive', f'{args.baseline_ref}:hex-command/dist'],
                       cwd=repository, stdout=target, check=True)
    with tarfile.open(archive) as source:
        source.extractall(original, filter='data')
    subprocess.run([sys.executable, 'tests/parity.py', '--capture', str(original),
                    '--output', str(args.output / 'baseline')], cwd=root, check=True)
    subprocess.run([sys.executable, 'tests/parity.py', '--capture', 'dist',
                    '--output', str(args.output / 'current'),
                    '--compare', str(args.output / 'baseline')], cwd=root, check=True)
