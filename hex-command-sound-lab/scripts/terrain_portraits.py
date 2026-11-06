#!/usr/bin/env python3
"""Copy exact complete terrain portraits selected by the game's terrain-preview.ts.

Run after the game's artwork build. Changes only terrain snapshot entries/artwork;
unit snapshots and all game sources stay untouched. No runtime game dependency.
"""
import argparse
import hashlib
import json
import re
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def snapshot(game):
    preview = (game / "src/presentation/terrain-preview.ts").read_bytes()
    choices = {}
    for quoted, bare, frame, floor in re.findall(r'^\s*(?:"([\w-]+)"|([\w-]+)):\s*\["([\w-]+)",\s*"([\w-]+)"\]', preview.decode(), re.M):
        choices[quoted or bare] = (frame, floor)
    frame_source = (game / "src/presentation/pixel-art.ts").read_text()
    frames = dict(re.findall(r'"?([\w-]+)"?: "(assets/sprites/[^"]+)"', frame_source))
    path = ROOT / "dist/workbench.json"
    workbench = json.loads(path.read_text())
    target = ROOT / "dist/workbench/terrain-portraits"
    target.mkdir(parents=True, exist_ok=True)
    provenance = {}
    for terrain in workbench["terrains"]:
        frame, floor = choices[terrain["id"]]
        name = "sacred_grove" if frame == "grove" else frame
        key = f"terrain_{name}__{floor}"
        if key not in frames:
            raise ValueError(f"Build game artwork first: missing exact portrait {key}")
        source = game / "public" / frames[key]
        raw = source.read_bytes()
        if raw[:8] != b"\x89PNG\r\n\x1a\n":
            raise ValueError(f"Invalid terrain PNG: {source}")
        width, height = struct.unpack(">II", raw[16:24])
        dest = target / f"{terrain['id']}.png"
        dest.write_bytes(raw)
        terrain["artwork"] = dest.relative_to(ROOT / "dist").as_posix()
        terrain["portraitFrame"] = key
        terrain["portraitFloor"] = floor
        provenance[terrain["id"]] = {"frame": key, "sourcePath": "public/" + frames[key],
                                      "sha256": hashlib.sha256(raw).hexdigest(), "width": width, "height": height}
    info = {"source": "Hexcommand src/presentation/terrain-preview.ts canonical portraits",
            "previewDefinitionSha256": hashlib.sha256(preview).hexdigest(),
            "method": "Exact copies of complete floor-colored portrait frames. No compositing, redraw or resizing.",
            "portraits": provenance}
    (target / "provenance.json").write_text(json.dumps(info, indent=2) + "\n")
    workbench["terrainPortraits"] = {"provenance": "workbench/terrain-portraits/provenance.json",
                                     "previewDefinitionSha256": info["previewDefinitionSha256"]}
    path.write_text(json.dumps(workbench, indent=2) + "\n")
    print(f"Copied {len(provenance)} exact complete game terrain portraits")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--game-dir", type=Path, required=True)
    snapshot(parser.parse_args().game_dir.resolve())
