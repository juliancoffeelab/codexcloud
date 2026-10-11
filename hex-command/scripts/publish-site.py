"""Commit and package the game; GitHub receives the commit after deployment."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import termios

ROOT = Path(__file__).resolve().parents[1]


def command(args, cwd=ROOT, env=None):
    result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"{' '.join(args)} failed:\n{result.stdout}{result.stderr}")
    return result.stdout.strip()


def step(label):
    print(f"PUBLISH: {label}", flush=True)


def read_credential():
    if sys.stdin.isatty():
        settings = termios.tcgetattr(sys.stdin.fileno())
        settings[3] &= ~termios.ECHO
        termios.tcsetattr(sys.stdin.fileno(), termios.TCSANOW, settings)
    print("Ready for Site credential JSON on stdin (input is hidden).", flush=True)
    credential = json.loads(sys.stdin.readline())
    if credential.get("auth_mode") != "http_extra_header":
        raise ValueError("Unsupported Site credential authentication mode")
    return credential


def prepare(credential, message):
    hosting = json.loads((ROOT / ".openai/hosting.json").read_text())
    repo = Path(command(["git", "rev-parse", "--show-toplevel"]))
    branch = command(["git", "symbolic-ref", "--short", "HEAD"])
    prefix = ROOT.relative_to(repo).as_posix()
    # Keep unrelated projects' staged changes out of the publication commit.
    step("Render generated assets")
    command(["npm", "run", "art:build"])
    step("Style checks")
    command(["npm", "run", "style"])
    step("Commit project source")
    command(["git", "add", "-A", "--", "."])
    changed = subprocess.run(["git", "diff", "--cached", "--quiet", "--", "."], cwd=ROOT)
    if changed.returncode not in (0, 1):
        raise RuntimeError("Could not inspect staged project changes")
    if changed.returncode:
        command(["git", "commit", "--only", "-m", message, "--", "."])
    source_commit = command(["git", "rev-parse", "HEAD"])
    command(["git", "diff", "--quiet", "HEAD", "--", "."])

    # The credential never enters an argument, file or log.
    environment = os.environ.copy()
    environment.update({
        "GIT_TERMINAL_PROMPT": "0",
        "GIT_CONFIG_COUNT": "2",
        "GIT_CONFIG_KEY_0": "http.extraHeader",
        "GIT_CONFIG_VALUE_0": "Authorization: Bearer " + credential["token"],
        "GIT_CONFIG_KEY_1": "credential.helper",
        "GIT_CONFIG_VALUE_1": "",
    })
    checkout = ROOT / ".cache/sites" / hosting["project_id"]
    step("Synchronize Site source")
    if checkout.exists():
        remote = command(["git", "remote", "get-url", "origin"], cwd=checkout)
        if remote != credential["remote_url"]:
            raise ValueError("Cached checkout belongs to a different Site repository")
        command(["git", "fetch", "origin", credential["branch"]], cwd=checkout, env=environment)
        command(["git", "checkout", "-B", credential["branch"], "FETCH_HEAD"], cwd=checkout)
    else:
        checkout.parent.mkdir(parents=True, exist_ok=True)
        command(["git", "clone", "--single-branch", "--branch", credential["branch"],
                 credential["remote_url"], str(checkout)], env=environment)
    manifest_path = checkout / ".openai/hosting.json"
    previous = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    if previous.get("project_id", hosting["project_id"]) != hosting["project_id"]:
        raise ValueError("Cached source manifest belongs to a different Site")
    for path in checkout.iterdir():
        if path.name == ".git":
            continue
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path)
        else:
            path.unlink()
    tree = source_commit if prefix == "." else f"{source_commit}:{prefix}"
    # Export tracked SOURCE only; build its output inside this isolated checkout.
    with tempfile.TemporaryFile() as source_tar:
        subprocess.run(["git", "archive", "--format=tar", tree], cwd=repo,
                       stdout=source_tar, check=True)
        source_tar.seek(0)
        with tarfile.open(fileobj=source_tar) as archive:
            archive.extractall(checkout, filter="data")
    manifest_path.write_text(json.dumps({**previous, **hosting}, separators=(",", ":")) + "\n")
    command(["git", "config", "user.name", "Codex"], cwd=checkout)
    command(["git", "config", "user.email", "codex@openai.com"], cwd=checkout)
    # Local cache/build exclusions do not alter the published source.
    with (checkout / ".git/info/exclude").open("a") as excludes:
        excludes.write("\ndist/\nnode_modules/\nartifacts/\n")
    command(["git", "add", "-A"], cwd=checkout)
    changed = subprocess.run(["git", "diff", "--cached", "--quiet"], cwd=checkout)
    if changed.returncode not in (0, 1):
        raise RuntimeError("Could not inspect staged Site changes")
    if changed.returncode:
        command(["git", "commit", "-m", f"{message} ({source_commit[:7]})"], cwd=checkout)
    # Reuse installed dependencies; source and generated output stay isolated.
    (checkout / "node_modules").symlink_to(ROOT / "node_modules", target_is_directory=True)
    step("Build committed Site source")
    command(["npm", "run", "build"], cwd=checkout)
    command(["git", "diff", "--quiet", "HEAD", "--", "."], cwd=checkout)
    command(["git", "push", "origin", credential["branch"]], cwd=checkout, env=environment)
    step("Package complete build")
    archive_path = str(Path(tempfile.gettempdir()) / f"{hosting['project_id']}-publish.tar.gz")
    result = json.loads(command(["python3", "scripts/package-site.py", archive_path], cwd=checkout))
    result.update({"source_commit": source_commit, "source_branch": branch,
                   "checkout_path": str(checkout)})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--message", default="Publish Hex Command")
    args = parser.parse_args()
    result = prepare(read_credential(), args.message)
    print("SITE_PACKAGE: " + json.dumps(result), flush=True)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Publication preparation failed: {error}", file=sys.stderr)
        sys.exit(1)
