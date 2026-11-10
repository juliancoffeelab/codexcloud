"""Synchronize current game source and prepare an unpublished Site release; never deploy."""

import argparse
import fcntl
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
import threading
import time

ROOT = Path(__file__).resolve().parents[1]


def command(args, cwd=ROOT, env=None):
    result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"{' '.join(args)} failed:\n{result.stdout}{result.stderr}")
    return result.stdout.strip()


def step(label):
    print(f"PREPARE: {label}", flush=True)


def run_stage_action(label, action, interval=30):
    started = time.monotonic()
    finished = threading.Event()

    def heartbeat():
        while not finished.wait(interval):
            step(f"{label} still running ({time.monotonic() - started:.0f}s)")

    worker = threading.Thread(target=heartbeat, daemon=True)
    worker.start()
    try:
        return action()
    finally:
        finished.set()
        worker.join()


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


def write_json(path, value):
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(json.dumps(value))
    temporary.replace(path)


def compile_frontend(checkout, wrapper, environment, diagnostics):
    diagnostics.mkdir(parents=True, exist_ok=True)
    report = {
        "source_commit": environment.get("HEX_COMMAND_SOURCE_COMMIT"),
        "checkout": str(checkout),
        "existing_build": (checkout / "_build").exists(),
        "cache_environment": {key: environment[key] for key in
                              ("XDG_CACHE_HOME", "DUNE_CACHE", "DUNE_CACHE_ROOT", "DUNE_JOBS")
                              if key in environment},
        "commands": [],
    }
    step(f"Frontend diagnostics: {diagnostics}")
    commands = [
        ("Generate resource visitors", "resource-visitors.log",
         ["node", wrapper, "node", "scripts/generate-resource-visitors.mjs"]),
        ("Generate fixture JSON", "fixture-json.log", ["node", "scripts/fixture-json.mjs"]),
        ("Compile Dune/Melange", "dune-output.log",
         ["node", wrapper, "dune", "build", "--root", str(checkout), "@melange",
          "--display=verbose", "--debug-package-logs",
          f"--trace-file={diagnostics / 'dune-trace.csexp'}"]),
    ]
    for label, filename, args in commands:
        step(label)
        log = diagnostics / filename
        entry = {"label": label, "args": args, "log": filename, "status": "running"}
        report["commands"].append(entry)
        write_json(diagnostics / "timings.json", report)
        started = time.monotonic()
        try:
            with log.open("w") as output:
                result = subprocess.run(args, cwd=checkout, env=environment,
                                        stdout=output, stderr=subprocess.STDOUT, text=True)
            entry["returncode"] = result.returncode
            if result.returncode:
                raise RuntimeError(f"{label} failed; log: {log}\n{log.read_text()}")
            entry["status"] = "succeeded"
        except Exception:
            entry["status"] = "failed"
            raise
        finally:
            entry["seconds"] = round(time.monotonic() - started, 3)
            write_json(diagnostics / "timings.json", report)
            step(f"{label} {entry['status']} in {entry['seconds']:.1f}s; log: {log}")


def sync_files(source, destination, names, previous):
    for name in previous - names:
        path = destination / name
        if path.is_file() or path.is_symlink():
            path.unlink()
        parent = path.parent
        while parent != destination:
            try:
                parent.rmdir()
            except OSError:
                break
            parent = parent.parent
    for name in names:
        incoming, target = source / name, destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        if incoming.is_symlink():
            link = os.readlink(incoming)
            if target.is_symlink() and os.readlink(target) == link:
                continue
            target.unlink(missing_ok=True)
            target.symlink_to(link)
        elif target.is_symlink() or not target.exists() or target.read_bytes() != incoming.read_bytes():
            target.unlink(missing_ok=True)
            shutil.copy2(incoming, target)


def extract(repo, tree, destination):
    with tempfile.TemporaryFile() as source_tar:
        subprocess.run(["git", "archive", "--format=tar", tree], cwd=repo,
                       stdout=source_tar, check=True)
        source_tar.seek(0)
        with tarfile.open(fileobj=source_tar) as archive:
            archive.extractall(destination, filter="data")
    return {path.relative_to(destination).as_posix() for path in destination.rglob("*")
            if path.is_file() or path.is_symlink()}


def outputs_digest(root, paths):
    digest = hashlib.sha256()
    for name in paths:
        path = root / name
        if not path.exists():
            return None
        files = sorted(p for p in path.rglob("*") if p.is_file() or p.is_symlink()) if path.is_dir() else [path]
        if not files:
            return None
        for item in files:
            digest.update(item.relative_to(root).as_posix().encode() + b"\0")
            data = os.readlink(item).encode() if item.is_symlink() else item.read_bytes()
            digest.update(hashlib.sha256(data).digest())
    return digest.hexdigest()


def prepare(credential, message, source_commit=None, preview=None, preview_project_id=None):
    # All destinations share one pinned-source build workspace. Its lock also
    # covers packaging, so another target cannot change the build being copied.
    cache = ROOT / ".cache/site-build"
    cache.mkdir(parents=True, exist_ok=True)
    with (cache / "lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        return prepare_locked(cache, credential, message, source_commit, preview, preview_project_id)


def prepare_locked(cache, credential, message, source_commit, preview, preview_project_id):
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
    manifest_path = cache / "source.json"
    try:
        if not manifest_path.exists() and (checkout.exists() or sdk.exists()):
            raise ValueError("Missing source inventory")
        previous = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
        if not isinstance(previous, dict) or any(not isinstance(previous.get(k, []), list) for k in ("game", "sdk")):
            raise ValueError("Invalid source cache")
    except (ValueError, OSError):
        # A damaged source inventory cannot safely describe stale input files.
        # Discard only these ignored build trees and reconstruct the pinned source.
        for directory in (checkout, sdk):
            if directory.exists():
                shutil.rmtree(directory)
        previous = {}
    checkout.mkdir(exist_ok=True)
    sdk.mkdir(exist_ok=True)
    step("Synchronize pinned build inputs")
    with tempfile.TemporaryDirectory() as snapshot:
        source = Path(snapshot) / "game"
        source.mkdir()
        names = extract(repo, source_commit if prefix == "." else f"{source_commit}:{prefix}", source)
        sdk_source = Path(snapshot) / "sdk"
        sdk_source.mkdir()
        sdk_names = extract(repo, f"{source_commit}:ocaml-js-bindgen", sdk_source)
        # Write ahead: an interrupted copy must not leave unaccounted source
        # files that can affect a later branch's build.
        write_json(manifest_path, {"game": sorted(names | set(previous.get("game", []))),
                                   "sdk": sorted(sdk_names | set(previous.get("sdk", [])))})
        sync_files(source, checkout, names, set(previous.get("game", [])))
        sync_files(sdk_source, sdk, sdk_names, set(previous.get("sdk", [])))
        write_json(manifest_path, {"game": sorted(names), "sdk": sorted(sdk_names)})

    def source_key(*paths, extra=""):
        entries = command(["git", "ls-tree", source_commit, "--", *[f"{prefix}/{p}" for p in paths]], cwd=repo)
        return hashlib.sha256((entries + extra).encode()).hexdigest()

    sdk_key = command(["git", "rev-parse", f"{source_commit}:ocaml-js-bindgen"], cwd=repo)
    pipeline_key = source_key("scripts/prepare-site.py", "scripts/toolchain", "scripts/install-dune.sh",
                              "rust-toolchain.toml", "rust-toolchain", ".cargo")
    stages_path = cache / "stages.json"
    try:
        stages = json.loads(stages_path.read_text()) if stages_path.exists() else {}
        if not isinstance(stages, dict):
            stages = {}
    except (ValueError, OSError):
        stages = {}

    def stage(label, key, root, outputs, action):
        key = hashlib.sha256((key + pipeline_key).encode()).hexdigest()
        previous = stages.get(label, {})
        if not isinstance(previous, dict):
            previous = {}
        digest = outputs_digest(root, outputs)
        if previous.get("key") == key and digest is not None and previous.get("outputs") == digest:
            step(label + " (cached)")
            return
        step(label)
        started = time.monotonic()
        run_stage_action(label, action)
        digest = outputs_digest(root, outputs)
        if digest is None:
            raise RuntimeError(label + " produced incomplete output")
        stages[label] = {"key": key, "outputs": digest}
        write_json(stages_path, stages)
        step(f"{label} finished in {time.monotonic() - started:.1f}s")

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
    wrapper = str(checkout / "scripts/toolchain/with-toolchain.mjs")
    def build(args, cwd=checkout):
        return command(["node", wrapper, *args], cwd=cwd, env=build_environment)
    # Explicit toolchain overrides must not reuse artifacts from another compiler.
    toolchain = "\n".join([
        build(["rustc", "-vV"]),
        build(["dune", "--version"]),
        command(["node", "--version"]),
    ])
    pipeline_key = hashlib.sha256((pipeline_key + toolchain).encode()).hexdigest()
    # Stage keys come from committed inputs, not mutable workspace files.
    stage("Build binding tool", sdk_key, sdk, ["target/debug/ocamljs_pack"],
          lambda: build(["cargo", "build", "--locked", "--manifest-path", str(sdk / "Cargo.toml"),
                         "--package", "ocamljs_pack", "--target-dir", str(sdk / "target")]))
    wasm_key = source_key("rust", "package.json", "package-lock.json", "scripts/optimize-wasm.mjs", extra=sdk_key)
    def wasm():
        build([str(sdk / "target/debug/ocamljs_pack"), "build", "--features", "wasm",
               "--name", "hex_command", "--out-dir", "pkg", "--profile", "wasm-release"], cwd=checkout / "rust")
        command(["node", "scripts/optimize-wasm.mjs", "rust/pkg/backend.wasm"], cwd=checkout, env=build_environment)
    stage("Build WASM", wasm_key, checkout,
          ["rust/pkg", "rust/pkg/backend.wasm", "rust/pkg/Bindings.ml", "rust/pkg/Bindings.mli"], wasm)
    for suffix in ("ml", "mli"):
        incoming, target = checkout / f"rust/pkg/Bindings.{suffix}", checkout / f"frontend/engine/Domain.{suffix}"
        if target.read_bytes() != incoming.read_bytes():
            shutil.copy2(incoming, target)
    frontend_key = source_key("frontend", "packages/save-migrations", "dune", "dune-project", "dune-workspace",
                              "dune.lock", ".dune-version", "package.json", "package-lock.json",
                              "scripts", extra=wasm_key)
    def frontend():
        directory = cache / "diagnostics"
        directory.mkdir(exist_ok=True)
        diagnostics = Path(tempfile.mkdtemp(prefix=f"{source_commit[:12]}-", dir=directory))
        compile_frontend(checkout, wrapper, build_environment, diagnostics)
    stage("Compile frontend", frontend_key, checkout,
          ["_build/default/frontend/output", "_build/default/frontend/output/frontend/Boot.js",
           "frontend/engine/Domain_resource_walk.ml", "frontend/Fixture_json.ml"], frontend)
    art_key = source_key("pixel", "rust/data", "scripts/build-pixel-art.py", "src/presentation/pixel-art.mjs")
    stage("Build artwork", art_key, checkout, ["public/assets/sprites", "src/presentation/pixel-art.mjs"],
          lambda: command(["python3", "scripts/build-pixel-art.py"], cwd=checkout, env=build_environment))
    # The bundle embeds the exact source commit; a new revision needs a bundle
    # even when its compiler and native inputs are unchanged.
    stage("Bundle client", source_commit, checkout, ["dist", "dist/index.html"],
          lambda: command(["node", "node_modules/vite/bin/vite.js", "build"], cwd=checkout, env=build_environment))

    # Source Git history belongs to the destination Site, independent of the
    # reusable build workspace. Copy only this pinned source and its built output.
    built = checkout
    checkout = ROOT / ".cache/sites" / hosting["project_id"] / "hex-command"
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
    result = prepare(read_credential(), args.message, args.source_commit, args.preview, args.preview_project_id)
    print("SITE_PACKAGE: " + json.dumps(result), flush=True)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Release preparation failed: {error}", file=sys.stderr)
        sys.exit(1)
