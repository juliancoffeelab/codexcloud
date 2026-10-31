"""Prepare committed game source as an unpublished Site release; never deploy."""

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
    print(f"PREPARE: {label}", flush=True)


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
    # Changes are checked, committed and pushed by the implementation workflow.
    # Preparation never reformats, tests or commits the working tree.
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
    tree = source_commit if prefix == "." else f"{source_commit}:{prefix}"
    # Copy only changed source; retain ignored build caches and source mtimes.
    with tempfile.TemporaryDirectory() as snapshot:
        source = Path(snapshot)
        with tempfile.TemporaryFile() as source_tar:
            subprocess.run(["git", "archive", "--format=tar", tree], cwd=repo,
                           stdout=source_tar, check=True)
            source_tar.seek(0)
            with tarfile.open(fileobj=source_tar) as archive:
                archive.extractall(source, filter="data")
        current_files = {path.relative_to(source).as_posix()
                         for path in source.rglob("*") if path.is_file() or path.is_symlink()}
        tracked = set(command(["git", "ls-files", "-z"], cwd=checkout).split("\0")) - {""}
        for name in tracked - current_files:
            path = checkout / name
            if path.is_file() or path.is_symlink():
                path.unlink()
            parent = path.parent
            while parent != checkout:
                try:
                    parent.rmdir()
                except OSError:
                    break
                parent = parent.parent
        for name in current_files:
            incoming, destination = source / name, checkout / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            if incoming.is_symlink():
                target = os.readlink(incoming)
                if destination.is_symlink() and os.readlink(destination) == target:
                    continue
                destination.unlink(missing_ok=True)
                destination.symlink_to(target)
            elif destination.is_symlink() or not destination.exists() or destination.read_bytes() != incoming.read_bytes():
                if destination.is_symlink():
                    destination.unlink()
                shutil.copy2(incoming, destination)
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
    dependencies = checkout / "node_modules"
    if not dependencies.exists():
        dependencies.symlink_to(ROOT / "node_modules", target_is_directory=True)
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
    parser.add_argument("--message", default="Prepare Hex Command release")
    args = parser.parse_args()
    result = prepare(read_credential(), args.message)
    print("SITE_PACKAGE: " + json.dumps(result), flush=True)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Release preparation failed: {error}", file=sys.stderr)
        sys.exit(1)
