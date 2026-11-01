"""Package the complete generated Site build, including Git-ignored client bundles."""

import json
from pathlib import Path
import subprocess
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python3 scripts/package-site.py /absolute/output.tar.gz")
    output = Path(sys.argv[1]).resolve()
    manifest = ROOT / ".openai" / "hosting.json"
    hosting = json.loads(manifest.read_text())
    build = ROOT / hosting["static"]["directory"]
    with tarfile.open(output, "w:gz") as archive:
        archive.add(manifest, arcname=str(manifest.relative_to(ROOT)))
        archive.add(build, arcname=str(build.relative_to(ROOT)))
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    print(json.dumps({"project_id": hosting["project_id"], "commit_sha": commit,
                      "archive": str(output)}))


if __name__ == "__main__":
    main()
