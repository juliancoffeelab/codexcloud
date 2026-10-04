"""Package the complete generated Site build, including Git-ignored client bundles."""

from html.parser import HTMLParser
import json
from pathlib import Path
import subprocess
import sys
import tarfile
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]


class ClientAssets(HTMLParser):
    def __init__(self):
        super().__init__()
        self.urls = []

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if tag == "script" and attrs.get("src"):
            self.urls.append(attrs["src"])
        if tag == "link" and attrs.get("rel") == "stylesheet":
            self.urls.append(attrs["href"])


def main():
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python3 scripts/package-site.py /absolute/output.tar.gz")
    output = Path(sys.argv[1]).resolve()
    subprocess.run(["git", "diff", "--quiet", "HEAD", "--", "."], cwd=ROOT, check=True)
    manifest = ROOT / ".openai" / "hosting.json"
    hosting = json.loads(manifest.read_text())
    build = ROOT / hosting["static"]["directory"]
    html = ClientAssets()
    html.feed((build / "index.html").read_text())
    for url in html.urls:
        path = urlsplit(url)
        if path.scheme or path.netloc:
            continue
        asset = (build / path.path.removeprefix("/")).resolve()
        if not asset.is_relative_to(build.resolve()) or not asset.is_file():
            raise ValueError(f"Missing client asset referenced by index.html: {url}")
    expected = {str(manifest.relative_to(ROOT))} | {
        str(path.relative_to(ROOT)) for path in build.rglob("*") if path.is_file()
    }
    with tarfile.open(output, "w:gz") as archive:
        archive.add(manifest, arcname=str(manifest.relative_to(ROOT)))
        archive.add(build, arcname=str(build.relative_to(ROOT)))
    with tarfile.open(output, "r:gz") as archive:
        actual = {member.name for member in archive.getmembers() if member.isfile()}
        if actual != expected:
            raise ValueError("Deployment archive does not contain the complete build")
        for name in expected:
            if archive.extractfile(name).read() != (ROOT / name).read_bytes():
                raise ValueError(f"Deployment archive differs from the build: {name}")
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    print(json.dumps({"project_id": hosting["project_id"], "commit_sha": commit,
                      "archive": str(output), "file_count": len(expected)}))


if __name__ == "__main__":
    main()
