#!/usr/bin/env python3
"""Render authored audition edits from verified originals. Called by build-soundboard.py."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "dist"
recipes = json.loads((ROOT / "audio-edits.json").read_text())
catalog = json.loads((OUT / "catalog.json").read_text())
originals = {item["id"]: item for item in catalog if not item.get("derivedFrom")}
result = list(originals.values())

for recipe in recipes["edits"]:
    original = originals[recipe["sourceId"]]
    source = ROOT / "vendor/sfx" / original["sourcePath"]
    digest = hashlib.sha256(source.read_bytes()).hexdigest()
    if digest != recipe["sourceHash"] or digest != original["sourceHash"]:
        raise ValueError(f"Original hash mismatch: {source}")
    identity = json.dumps({"recipe": recipe, "format": "mp3-mono-44100-96k"}, sort_keys=True)
    name = hashlib.sha256(identity.encode()).hexdigest()[:20] + ".mp3"
    path = OUT / "audio/listening" / name
    subprocess.run([
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-nostdin", "-y",
        "-i", str(source), "-map_metadata", "-1", "-af", ",".join(recipe["filters"]),
        "-ac", "1", "-ar", "44100", "-codec:a", "libmp3lame", "-b:a", "96k", str(path),
    ], check=True)
    decoded = subprocess.check_output([
        "ffmpeg", "-v", "error", "-nostdin", "-i", str(path), "-ac", "1", "-ar", "44100", "-f", "f32le", "-",
    ])
    duration = len(decoded) / (4 * 44100)
    if duration <= 0:
        raise ValueError(f"Empty rendered edit: {recipe['id']}")
    item = {**original,
        "id": recipe["id"], "name": recipe["name"], "path": path.relative_to(OUT).as_posix(),
        "pack": "audition-edits", "packLabel": "Audition edits", "kind": "processed-foley" if original["kind"] in ["recorded-foley", "recorded-weapons"] else "processed-effect",
        "originalKind": original["kind"], "takeMethod": "authored-edit",
        "derivedFrom": original["id"], "processingRecipe": recipe, "duration": round(duration, 4),
        "gainDb": 0, "categories": original.get("categories", ["movement"]),
        "takes": [{"start": 0, "duration": round(duration, 4)}],
        "notes": recipe["intent"] + " Level matching does not boost this deliberately quiet edit.",
    }
    result.append(item)
    print(f"Rendered {recipe['id']} from {original['id']}")

(OUT / "catalog.json").write_text(json.dumps(result, indent=2) + "\n")
(OUT / "audio-edits.json").write_text(json.dumps(recipes, indent=2) + "\n")
