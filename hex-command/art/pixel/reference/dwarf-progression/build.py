"""Derive dwarf tiers from reviewed native bases and full material ownership masks."""

import argparse
from collections import defaultdict
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parents[3]
UNITS = PROJECT / "art/pixel/units"
ARMOUR = "KCPABLF"
PROTECTED = "hbwmcn"
LEATHER = {"L": [150, 103, 59, 255], "l": [92, 58, 33, 255], "k": [190, 140, 88, 255]}
IRON = {"P": [151, 173, 188, 255], "p": [84, 111, 128, 255], "S": [210, 226, 236, 255]}
SILVER = {"P": [218, 231, 235, 255], "p": [129, 151, 160, 255], "S": [249, 251, 245, 255]}
METAL_L2 = "KCPA"


def read_pose(directory):
    rows = (directory / "player.txt").read_text().splitlines()
    mask = (directory / "materials.mask").read_text().splitlines()
    palette = json.loads((directory / "palette.json").read_text())
    if len(rows) != len(mask) or any(len(a) != len(b) for a, b in zip(rows, mask)):
        raise ValueError("Material mask requires its exact source canvas")
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            owner = mask[y][x]
            if owner not in ARMOUR + PROTECTED + "Do.":
                raise ValueError(f"Unclassified source pixel ({x},{y})")
            if (symbol == ".") != (owner == ".") or (symbol == "O") != (owner == "o"):
                raise ValueError(f"Material mask changes silhouette/outline ({x},{y})")
            if symbol not in palette:
                raise ValueError(f"Missing source palette color: {symbol}")
            if owner == "m" and symbol not in "AaM":
                raise ValueError(f"Nonmetal pixel in weapon head ({x},{y})")
            if owner == "w" and symbol not in "BbWwVv":
                raise ValueError(f"Nonwood pixel in shaft ({x},{y})")
    for part in "KCPALF":
        if not any(part in row for row in mask):
            raise ValueError(f"Missing armour part: {part}")
    return rows, mask, palette


def derive(rows, mask, palette, level):
    output = [list(row) for row in rows]
    result_palette = {**palette, **LEATHER, **(SILVER if level == 3 else IRON)}
    groups = defaultdict(list)
    for y, row in enumerate(mask):
        for x, part in enumerate(row):
            if part in ARMOUR or (part == "D" and level == 3):
                # Separate opposing limbs; shading never crosses the body gap.
                side = (x >= 31) if part in "PALF" else None
                groups[part, side].append((x, y))
    for (part, _), pixels in groups.items():
        metal = level == 3 or (level == 2 and part in METAL_L2)
        light, mid, dark = ("S", "P", "p") if metal else ("k", "L", "l")
        top = min(y for _, y in pixels)
        bottom = max(y for _, y in pixels)
        row_edges = {}
        for y in {y for _, y in pixels}:
            xs = [x for x, yy in pixels if yy == y]
            row_edges[y] = min(xs), max(xs)
        for x, y in pixels:
            left, right = row_edges[y]
            if part == "K":
                # Keep the cleaned cap's connected highlight and brim geometry.
                value = light if rows[y][x] == "A" else dark if rows[y][x] == "a" else mid
            elif part == "D":
                # Keep the round shield's boss and rim distinct from its face.
                value = light if rows[y][x] == "A" else dark if rows[y][x] == "a" else mid
                if rows[y][x] in "Ww" and x >= right - 1:
                    value = dark
            else:
                value = mid
                if x >= right - 1 or y == bottom:
                    value = dark
                elif x == left and y <= top + 2:
                    value = light
            output[y][x] = value
    result = ["".join(row) for row in output]
    verify(rows, mask, palette, result, result_palette, level)
    return result, result_palette


def verify(source, mask, original_palette, rows, palette, level):
    if len(source) != len(rows) or any(len(a) != len(b) for a, b in zip(source, rows)):
        raise ValueError("Tier changes source canvas")
    counts = defaultdict(int)
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            before, owner = source[y][x], mask[y][x]
            if (before == ".") != (symbol == ".") or (before == "O") != (symbol == "O"):
                raise ValueError(f"Tier changes silhouette/outline ({x},{y})")
            if owner in PROTECTED or owner in ".o" or (owner == "D" and level < 3):
                if symbol != before or palette[symbol] != original_palette[symbol]:
                    raise ValueError(f"Tier recolors protected {owner} pixel ({x},{y})")
            else:
                metal = level == 3 or (level == 2 and owner in METAL_L2)
                expected = "PpS" if metal else "Llk"
                if symbol not in expected:
                    raise ValueError(f"Incomplete {owner} piece: {symbol} at ({x},{y})")
                counts[owner] += 1
            if symbol not in palette:
                raise ValueError(f"Missing tier color: {symbol}")
    return dict(counts)


def run(ids, destination, check=False, render=True):
    poses = json.loads((HERE / "sources.json").read_text())["poses"]
    chosen = [pose for pose in poses if not ids or "dwarf-" + pose in ids]
    if ids and len(chosen) != len(set(ids)):
        raise ValueError("Unknown dwarf progression pose; rune line is excluded")
    matrices = []
    coverage = {}
    for pose in chosen:
        base_id = "dwarf-" + pose
        source, mask, original_palette = read_pose(HERE / "bases" / base_id)
        for level in (1, 2, 3):
            unit = base_id + (f"-l{level}" if level > 1 else "")
            output = destination / unit
            rows, palette = derive(source, mask, original_palette, level)
            player = "\n".join(rows) + "\n"
            enemy = player.translate(str.maketrans("Tt", "Rr"))
            coverage[unit] = verify(source, mask, original_palette, rows, palette, level)
            if check:
                for side, expected in (("player", player), ("enemy", enemy)):
                    if (output / f"{side}.txt").read_text() != expected:
                        raise ValueError(f"{unit}/{side}: regenerate progression")
                if json.loads((output / "palette.json").read_text()) != palette:
                    raise ValueError(f"{unit}: regenerate tier palette")
            else:
                output.mkdir(parents=True, exist_ok=True)
                (output / "player.txt").write_text(player)
                (output / "enemy.txt").write_text(enemy)
                (output / "palette.json").write_text(json.dumps(palette, indent=2) + "\n")
                edits = [{"x": x, "y": y, "from": source[y][x], "to": c, "part": mask[y][x]}
                         for y, row in enumerate(rows) for x, c in enumerate(row) if c != source[y][x]]
                (output / "changes.json").write_text(json.dumps({
                    "base": base_id,
                    "source": "dwarf-progression/bases/" + base_id,
                    "level": level,
                    "changes": edits,
                    "paletteChanges": {s: c for s, c in palette.items() if original_palette.get(s) != c},
                }, indent=2) + "\n")
                (output / "components.md").write_text(
                    f"# {unit}\n\n"
                    f"L{level} artwork derived from the cleaned `{base_id}` pose. "
                    "The branch assignment is in `art/pixel/lineups/dwarves.json`.\n\n"
                    "Armour: L1 leather; L2 metal helmet, shoulders, cuirass and arm guards "
                    "over leather leggings, boots and apron; L3 complete silver coverage. "
                    "Face, hands, beard, wooden shafts, weapon steel, faction fabric "
                    "and mount/tack remain their own materials.\n\n"
                    "The exact native inputs and ownership masks are saved under "
                    f"`art/pixel/reference/dwarf-progression/bases/{base_id}/`. "
                    "Edit that recipe and run `python3 art/pixel/reference/dwarf-progression/build.py`; "
                    "do not patch PNGs or reuse the older armour rectangles.\n"
                )
            matrices.extend(output / f"{side}.txt" for side in ("player", "enemy"))
    if render:
        subprocess.run([sys.executable, str(PROJECT / "scripts/render-pixel.py"),
                        *(str(p) for p in matrices), *( ["--check"] if check else [])],
                       cwd=PROJECT, check=True, stdout=subprocess.DEVNULL)
    if not ids and destination == UNITS:
        report = {
            "armourPixelsByUnitAndPart": coverage,
            "l1": "Every wearer armour part is leather",
            "l2": "Helmet, cuirass, shoulders and arm guards are metal; apron, leggings and boots are leather",
            "l3": "Every wearer armour part and the shield is silver",
            "protected": "Skin, beard, shafts, weapon steel, faction cloth, goat coat and mount tack",
            "runeLine": "Excluded",
        }
        file = HERE / "coverage.json"
        if check:
            if json.loads(file.read_text()) != report:
                raise ValueError("Regenerate the saved armour coverage report")
        else:
            file.write_text(json.dumps(report, indent=2) + "\n")
    print(f"{'Verified' if check else 'Derived'} {len(coverage)} dwarf tiers: every armour piece has complete material coverage; protected pixels unchanged")
    return coverage


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ids", nargs="*")
    parser.add_argument("--output", type=Path, default=UNITS)
    parser.add_argument("--check", action="store_true")
    options = parser.parse_args()
    run(options.ids, options.output.resolve(), check=options.check)
