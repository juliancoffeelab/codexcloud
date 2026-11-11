"""Synchronize current game source and prepare an unpublished Site release; never deploy."""

import argparse
import fcntl
import json
import importlib.util
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[1]

spec = importlib.util.spec_from_file_location("source_workspace", ROOT / "scripts/source-workspace.py")
workspace = importlib.util.module_from_spec(spec)
spec.loader.exec_module(workspace)
sync_files = workspace.sync_files


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
    print("SITE_CREDENTIAL_REQUIRED", flush=True)
    credential = json.loads(sys.stdin.readline())
    if credential.get("auth_mode", "work_proxy") not in ("http_extra_header", "work_proxy"):
        raise ValueError("Unsupported Site credential authentication mode")
    return credential


def extract(repo, tree, destination):
    with tempfile.TemporaryFile() as source_tar:
        subprocess.run(["git", "archive", "--format=tar", tree], cwd=repo,
                       stdout=source_tar, check=True)
        source_tar.seek(0)
        with tarfile.open(fileobj=source_tar) as archive:
            archive.extractall(destination, filter="data")
    return {path.relative_to(destination).as_posix() for path in destination.rglob("*")
            if path.is_file() or path.is_symlink()}


def prepare(message, source_commit=None, preview=None, preview_project_id=None):
    # All destinations share one pinned-source build workspace. Its lock also
    # covers packaging, so another target cannot change the build being copied.
    cache = ROOT / ".cache/site-build"
    cache.mkdir(parents=True, exist_ok=True)
    with (cache / "lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        return prepare_locked(cache, message, source_commit, preview, preview_project_id)


def prepare_locked(cache, message, source_commit, preview, preview_project_id):
    repo = Path(command(["git", "rev-parse", "--show-toplevel"]))
    prefix = ROOT.relative_to(repo).as_posix()
    source_commit = source_commit or command(["git", "rev-parse", "HEAD"])
    hosting = json.loads(command(["git", "show", f"{source_commit}:./.openai/hosting.json"]))
    if preview is not None:
        if not str(preview).strip() or not preview_project_id or preview_project_id == hosting["project_id"]:
            raise ValueError("Preview requires a tag and an independent resolved Site")
        hosting["project_id"] = preview_project_id
    elif preview_project_id is not None:
        raise ValueError("A preview Site cannot override a main deployment")

    checkout = cache / "hex-command"
    sdk = cache / "ocaml-js-bindgen"
    step("Synchronize pinned build inputs")
    with tempfile.TemporaryDirectory() as snapshot:
        source = Path(snapshot) / "game"
        source.mkdir()
        names = extract(repo, source_commit if prefix == "." else f"{source_commit}:{prefix}", source)
        sdk_source = Path(snapshot) / "sdk"
        sdk_source.mkdir()
        extract(repo, f"{source_commit}:ocaml-js-bindgen", sdk_source)
        workspace.update_workspace(cache, {"hex-command": source, "ocaml-js-bindgen": sdk_source})

    # Cargo, Dune and Vite retain their own incremental state at stable paths.
    step("Prepare locked JavaScript dependencies")
    workspace.install_dependencies(checkout)
    started = time.monotonic()
    step("Build pinned production source")
    build_environment = os.environ.copy()
    build_environment["DUNE_CACHE"] = "enabled"
    wrapper = str(checkout / "scripts/toolchain/with-toolchain.mjs")
    # Build logs stream directly so progress and failures are visible immediately.
    subprocess.run(["node", wrapper, "bash", "scripts/build-wasm.sh", "release"],
                   cwd=checkout, env=build_environment, check=True)
    subprocess.run(["node", wrapper, "dune", "build", "--root=.", "--cache=enabled", "@melange", "@wasm-release", "@artwork"],
                   cwd=checkout, env=build_environment, check=True)
    subprocess.run(["node", "node_modules/vite/bin/vite.js", "build"], cwd=checkout,
                   env={**build_environment, "HEX_COMMAND_PROFILE": "release", "HEX_COMMAND_SOURCE_COMMIT": source_commit}, check=True)
    step(f"Production build finished in {time.monotonic() - started:.3f}s")

    built = checkout
    destination = ROOT / ".cache/sites" / hosting["project_id"] / "hex-command"

    def synchronize(credential):
        environment = build_environment.copy()
        environment["GIT_TERMINAL_PROMPT"] = "0"
        if credential.get("auth_mode", "work_proxy") == "http_extra_header":
            environment.update({
                "GIT_CONFIG_COUNT": "2",
                "GIT_CONFIG_KEY_0": "http.extraHeader",
                "GIT_CONFIG_VALUE_0": "Authorization: Bearer " + credential["token"],
                "GIT_CONFIG_KEY_1": "credential.helper",
                "GIT_CONFIG_VALUE_1": "",
            })

        # Source Git history belongs to the destination Site, independent of the
        # reusable build workspace. Copy only this pinned source and its built output.
        checkout = destination
        step("Synchronize Site source")
        empty_remote = not command(["git", "ls-remote", "--heads", credential["remote_url"]], env=environment)
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
                command(["git", "-c", "core.hooksPath=/dev/null", "clone", "--single-branch", "--branch",
                         credential["branch"], credential["remote_url"], str(checkout)], env=environment)
        tracked = set(command(["git", "ls-files", "-z"], cwd=checkout).split("\0")) - {""}
        sync_files(built, checkout, names, tracked)
        (checkout / ".openai/hosting.json").write_text(json.dumps(hosting, separators=(",", ":")) + "\n")
        static = Path(hosting["static"]["directory"])
        if static.is_absolute() or not static.parts or any(p in ("..", ".git") for p in static.parts):
            raise ValueError("Static build directory must stay inside the Site checkout")
        if not (built / static).is_dir():
            raise ValueError("Configured static build directory was not built")
        if (checkout / static).exists():
            shutil.rmtree(checkout / static)
        shutil.copytree(built / static, checkout / static)
        command(["git", "config", "user.name", "Codex"], cwd=checkout)
        command(["git", "config", "user.email", "codex@openai.com"], cwd=checkout)
        # Generated source is part of the built Site snapshot, not a clean-tree gate.
        command(["git", "add", "-A"], cwd=checkout)
        command(["git", "-c", "core.hooksPath=/dev/null", "commit", "--allow-empty",
                 "-m", f"{message} ({source_commit[:7]})"], cwd=checkout)
        command(["git", "-c", "core.hooksPath=/dev/null", "push", "origin", credential["branch"]], cwd=checkout, env=environment)

    # An authorization retry only synchronizes this exact completed build.
    # Compilation and artifact selection never restart or fall back to another pin.
    for attempt in range(2):
        credential = read_credential()
        try:
            synchronize(credential)
            break
        except RuntimeError as error:
            if attempt or not re.search(r"\b(?:401|403)\b|expired token|authentication failed", str(error), re.I):
                raise
            step("Source authorization rejected; refreshing for the same completed build")
    checkout = destination
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
    parser.add_argument("--preview", help="Retained first-branch preview tag")
    parser.add_argument("--preview-project-id", help="Exact Site identity resolved by the operation")
    args = parser.parse_args()
    result = prepare(args.message, args.source_commit, args.preview, args.preview_project_id)
    print("SITE_PACKAGE: " + json.dumps(result), flush=True)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Release preparation failed: {error}", file=sys.stderr)
        sys.exit(1)
