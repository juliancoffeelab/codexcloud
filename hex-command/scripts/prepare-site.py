"""Synchronize current game source and prepare an unpublished Site release; never deploy."""

import argparse
import hashlib
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
    if credential.get("auth_mode", "work_proxy") not in ("http_extra_header", "work_proxy"):
        raise ValueError("Unsupported Site credential authentication mode")
    return credential


def prepare(credential, message, source_commit=None, preview=None, preview_project_id=None):
    repo = Path(command(["git", "rev-parse", "--show-toplevel"]))
    prefix = ROOT.relative_to(repo).as_posix()
    # Implementation owns committing/pushing required changes. Publish HEAD;
    # unrelated staged, unstaged and untracked work is neither a blocker nor
    # input to this release. Never stage or commit it as a deployment side effect.
    source_commit = source_commit or command(["git", "rev-parse", "HEAD"])
    hosting = json.loads(command(["git", "show", f"{source_commit}:./.openai/hosting.json"]))
    if preview is not None:
        if preview < 1 or not preview_project_id or preview_project_id == hosting["project_id"]:
            raise ValueError("Preview requires a positive PR number and an independent resolved Site")
        hosting["project_id"] = preview_project_id
    elif preview_project_id is not None:
        raise ValueError("A preview Site cannot override a main deployment")

    # The credential never enters an argument, file or log.
    environment = os.environ.copy()
    environment["GIT_TERMINAL_PROMPT"] = "0"
    if credential.get("auth_mode", "work_proxy") == "http_extra_header":
        environment.update({
            "GIT_CONFIG_COUNT": "2",
            "GIT_CONFIG_KEY_0": "http.extraHeader",
            "GIT_CONFIG_VALUE_0": "Authorization: Bearer " + credential["token"],
            "GIT_CONFIG_KEY_1": "credential.helper",
            "GIT_CONFIG_VALUE_1": "",
        })
    checkout = ROOT / ".cache/sites" / hosting["project_id"] / "hex-command"
    step("Synchronize Site source")
    # Newly registered Sites have an empty repository, without a branch to clone.
    empty_remote = not command(["git", "ls-remote", "--heads", credential["remote_url"]],
                               env=environment)
    if checkout.exists():
        command(["git", "remote", "set-url", "origin", credential["remote_url"]], cwd=checkout)
        if empty_remote:
            command(["git", "symbolic-ref", "HEAD", "refs/heads/" + credential["branch"]], cwd=checkout)
        else:
            command(["git", "fetch", "origin", credential["branch"]], cwd=checkout, env=environment)
            command(["git", "-c", "core.hooksPath=/dev/null", "checkout", "--force", "-B",
                     credential["branch"], "FETCH_HEAD"], cwd=checkout)
        command(["git", "clean", "-fd"], cwd=checkout)
    else:
        checkout.parent.mkdir(parents=True, exist_ok=True)
        if empty_remote:
            command(["git", "init", "--initial-branch", credential["branch"], str(checkout)])
            command(["git", "remote", "add", "origin", credential["remote_url"]], cwd=checkout)
        else:
            command(["git", "-c", "core.hooksPath=/dev/null", "clone", "--single-branch", "--branch", credential["branch"],
                     credential["remote_url"], str(checkout)], env=environment)
    manifest_path = checkout / ".openai/hosting.json"
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
    manifest_path.write_text(json.dumps(hosting, separators=(",", ":")) + "\n")
    command(["git", "config", "user.name", "Codex"], cwd=checkout)
    command(["git", "config", "user.email", "codex@openai.com"], cwd=checkout)
    # Install only when the committed dependency inputs change. Never borrow a
    # mutable workspace's node_modules for a supposedly pinned release build.
    dependencies = checkout / "node_modules"
    dependency_key = hashlib.sha256((checkout / 'package-lock.json').read_bytes()
                                    + (checkout / 'package.json').read_bytes()).hexdigest()
    if dependencies.is_symlink():
        dependencies.unlink()
    installed = dependencies / '.release-lock.sha256'
    if not installed.exists() or installed.read_text() != dependency_key:
        command(["npm", "ci", "--include=dev", "--ignore-scripts", "--no-audit", "--no-fund"], cwd=checkout)
        installed.write_text(dependency_key)
    build_environment = os.environ.copy()
    build_environment["HEX_COMMAND_SOURCE_COMMIT"] = source_commit
    build_environment['PATH'] = str(dependencies / '.bin') + os.pathsep + build_environment.get('PATH', '')
    step("Build committed Site source")
    # Keep the sibling SDK at the same pinned repository revision. It is build
    # input only, outside the Site repository and publication archive.
    sdk = checkout.parent / "ocaml-js-bindgen"
    sdk.mkdir(parents=True, exist_ok=True)
    sdk_manifest = sdk / ".release-source.json"
    old_files = set(json.loads(sdk_manifest.read_text())["files"]) if sdk_manifest.exists() else set()
    with tempfile.TemporaryDirectory() as snapshot:
        with tempfile.TemporaryFile() as source_tar:
            subprocess.run(["git", "archive", "--format=tar", f"{source_commit}:ocaml-js-bindgen"],
                           cwd=repo, stdout=source_tar, check=True)
            source_tar.seek(0)
            with tarfile.open(fileobj=source_tar) as archive:
                archive.extractall(snapshot, filter="data")
        source = Path(snapshot)
        files = {path.relative_to(source).as_posix() for path in source.rglob("*")
                 if path.is_file() or path.is_symlink()}
        for name in old_files - files:
            (sdk / name).unlink(missing_ok=True)
        for name in files:
            incoming, destination = source / name, sdk / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            if incoming.is_symlink():
                if not destination.is_symlink() or os.readlink(destination) != os.readlink(incoming):
                    destination.unlink(missing_ok=True)
                    destination.symlink_to(os.readlink(incoming))
            elif destination.is_symlink() or not destination.exists() or destination.read_bytes() != incoming.read_bytes():
                destination.unlink(missing_ok=True)
                shutil.copy2(incoming, destination)
        sdk_manifest.write_text(json.dumps({"source_commit": source_commit, "files": sorted(files)}))
    # One build graph owns generation, checks, compilation, artwork and bundling.
    command(["node", "scripts/toolchain/with-toolchain.mjs", "make", "build",
             "PROFILE=wasm-release"], cwd=checkout, env=build_environment)
    # Generated source is part of the built Site snapshot, not a clean-tree gate.
    command(["git", "add", "-A"], cwd=checkout)
    command(["git", "-c", "core.hooksPath=/dev/null", "commit", "--allow-empty",
             "-m", f"{message} ({source_commit[:7]})"], cwd=checkout)
    command(["git", "-c", "core.hooksPath=/dev/null", "push", "origin", credential["branch"]], cwd=checkout, env=environment)
    step("Package complete build")
    archive_fd, archive_path = tempfile.mkstemp(prefix=f"{hosting['project_id']}-", suffix='.tar.gz')
    os.close(archive_fd)
    result = json.loads(command(["python3", "scripts/package-site.py", archive_path], cwd=checkout))
    result.update({"source_commit": source_commit, "checkout_path": str(checkout)})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--message", default="Prepare Hex Command release")
    parser.add_argument("--source-commit", help="Exact implementation commit pinned by the operation")
    parser.add_argument("--preview", type=int, help="Pull request preview number")
    parser.add_argument("--preview-project-id", help="Exact Site identity resolved by the operation")
    args = parser.parse_args()
    result = prepare(read_credential(), args.message, args.source_commit, args.preview, args.preview_project_id)
    print("SITE_PACKAGE: " + json.dumps(result), flush=True)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Release preparation failed: {error}", file=sys.stderr)
        sys.exit(1)
