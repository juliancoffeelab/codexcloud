"""Ownership of generated CSV exports is recorded by tables.json."""

import json
from pathlib import Path


def remove_obsolete_csv(output, tables):
    output.mkdir(parents=True, exist_ok=True)
    manifest = output / "tables.json"
    if not manifest.exists():
        return
    previous = json.loads(manifest.read_text())
    if not isinstance(previous, dict) or any(
        not isinstance(name, str) or Path(name).name != name or not name.isidentifier()
        for name in previous
    ):
        raise ValueError("Invalid generated table manifest")
    for name in previous.keys() - tables.keys():
        (output / (name + ".csv")).unlink(missing_ok=True)
