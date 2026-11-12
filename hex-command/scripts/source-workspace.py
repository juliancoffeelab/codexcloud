"""Copy source inventories into a stable workspace without touching native outputs."""

import hashlib
import json
import os
from pathlib import Path
import shutil


def write_json(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, sort_keys=True))
    temporary.replace(path)


def sync_files(source, destination, names, previous):
    for name in previous - names:
        path = destination / name
        if path.is_file() or path.is_symlink():
            path.unlink()
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
            # Cargo uses input mtimes. Changed sources need fresh timestamps,
            # not archive timestamps that can predate the last build.
            shutil.copyfile(incoming, target)
            shutil.copymode(incoming, target)
        elif incoming.stat().st_mode != target.stat().st_mode:
            shutil.copymode(incoming, target)


def update_workspace(cache, sources):
    """Write ahead so an interrupted copy cannot leave unaccounted source files."""
    manifest = cache / "sources.json"
    previous = json.loads(manifest.read_text()) if manifest.exists() else {}
    inventories = {name: sorted(path.relative_to(source).as_posix()
                               for path in source.rglob("*")
                               if path.is_file() or path.is_symlink())
                   for name, source in sources.items()}
    write_json(manifest, {name: sorted(set(files) | set(previous.get(name, [])))
                          for name, files in inventories.items()})
    for name, source in sources.items():
        destination = cache / name
        destination.mkdir(exist_ok=True)
        sync_files(source, destination, set(inventories[name]), set(previous.get(name, [])))
    write_json(manifest, inventories)


def install_dependencies(checkout):
    """npm owns installed dependencies; only its lockfile changes require npm ci."""
    import subprocess
    key = hashlib.sha256((checkout / "package.json").read_bytes()
                         + (checkout / "package-lock.json").read_bytes()).hexdigest()
    installed = checkout / "node_modules/.source-lock.sha256"
    if not installed.exists() or installed.read_text() != key:
        subprocess.run(["npm", "ci", "--include=dev", "--ignore-scripts", "--no-audit", "--no-fund"],
                       cwd=checkout, check=True)
        installed.write_text(key)
