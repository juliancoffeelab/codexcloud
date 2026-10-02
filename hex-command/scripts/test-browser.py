"""Run the existing browser regression suite against the production build."""
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
for test in sorted((root / 'tests').glob('*.py')):
    if test.name == 'parity.py':
        continue
    print(f'Running {test.name}', flush=True)
    subprocess.run([sys.executable, str(test)], cwd=root, check=True)
