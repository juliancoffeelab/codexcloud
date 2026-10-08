"""Rebuild the nine restored dwarf bases and check their visible fill regions."""

import argparse
from collections import deque
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parents[3]
UNITS = PROJECT / "art/pixel/units"
# A shade is part of its main colour, never a separate edge/fill region.
MATERIALS = {"brown": "Bb", "skin": "Hh", "cream": "Ii", "cloth": "TtRr", "steel": "MaA"}
OWNER = {symbol: material for material, symbols in MATERIALS.items() for symbol in symbols}


def surfaces(rows):
    """Detect material edges from actual artwork, then flood-fill between them."""
    height, width = len(rows), len(rows[0])
    seen, result = set(), []
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            if symbol not in OWNER and symbol not in ".O":
                raise ValueError(f"Unexpected colour symbol at ({x},{y}): {symbol}")
            if symbol in ".O" or (x, y) in seen:
                continue
            material = OWNER[symbol]
            points, queue = [], deque([(x, y)])
            seen.add((x, y))
            while queue:
                a, b = queue.popleft()
                points.append((a, b))
                for xx, yy in ((a - 1, b), (a + 1, b), (a, b - 1), (a, b + 1)):
                    if (0 <= xx < width and 0 <= yy < height and (xx, yy) not in seen
                            and OWNER.get(rows[yy][xx]) == material):
                        seen.add((xx, yy))
                        queue.append((xx, yy))
            result.append((material, points))
    return result


def describe(rows):
    parts, mask = [], [[".." if c == "." else "##" for c in row] for row in rows]
    for number, (material, points) in enumerate(surfaces(rows)):
        token = f"{number:02x}"
        if number >= 256:
            raise ValueError("Too many fragments in a base")
        for x, y in points:
            mask[y][x] = token
        parts.append({"id": token, "material": material, "seed": list(points[0]),
                      "pixels": len(points),
                      "bounds": [min(x for x, _ in points), min(y for _, y in points),
                                 max(x for x, _ in points), max(y for _, y in points)]})
    return parts, "\n".join(" ".join(row) for row in mask) + "\n"


def verify_parts(rows, expected, expected_mask):
    parts, mask = describe(rows)
    if parts != expected or mask != expected_mask:
        raise ValueError("A visible material edge changed or a fill region leaked")
    return parts


def verify_anchors(rows, anchors):
    """Named physical parts must be reachable from their own seeds independently."""
    _, mask = describe(rows)
    grid = [row.split() for row in mask.splitlines()]
    occupied = {}
    for anchor in anchors:
        x, y = anchor["seed"]
        if OWNER.get(rows[y][x]) != anchor["material"]:
            raise ValueError(f"Lost {anchor['name']} at its reviewed fill seed")
        region = grid[y][x]
        if region in occupied:
            raise ValueError(f"Fill leaks between {occupied[region]} and {anchor['name']}")
        if region != anchor["region"]:
            raise ValueError(f"Fill boundary changed around {anchor['name']}")
        occupied[region] = anchor["name"]


def verify_original_details(original, rows, reviewed):
    eyes = set()
    for edit in reviewed:
        if edit.get("feature") != "eye":
            continue
        x, y = edit["x"], edit["y"]
        if (not (0 < x < 63 and 0 < y < 63)
                or edit["from"] not in "Hh" or edit["to"] != "O"
                or original[y][x] not in "Hh" or rows[y][x] != "O"
                or rows[y - 1][x] not in "MaA"
                or any(rows[yy][xx] not in "Hh" for xx, yy in
                       ((x - 1, y), (x + 1, y), (x, y + 1)))):
            raise ValueError("Reviewed eye must be a single facial pixel below the helmet")
        eyes.add((x, y))
    removals = {(p["x"], p["y"]) for p in reviewed if p["from"] == "O"}
    for x, y in removals:
        if original[y][x] != "O" or not (0 < x < 63 and 0 < y < 63):
            raise ValueError("Invalid reviewed interior speck")
        ring = [original[yy][xx] for yy in range(y - 1, y + 2)
                for xx in range(x - 1, x + 2) if (xx, yy) != (x, y)]
        if any(c not in "BbVvWw" for c in ring):
            raise ValueError("Removing this dark pixel would erase a contour or material edge")
    for y, row in enumerate(original):
        for x, symbol in enumerate(row):
            if symbol == "O" and rows[y][x] != "O" and (x, y) not in removals:
                raise ValueError(f"Original contour/facial feature erased at ({x},{y})")
    # Connected original skin fields include hands and exposed forearms, not armour.
    normal = [row.translate(str.maketrans({c: "B" for c in "VvWwGE"})) for row in original]
    for material, points in surfaces(normal):
        if material == "skin" and len(points) >= 3:
            if any(OWNER.get(rows[y][x]) != "skin" and (x, y) not in eyes for x, y in points):
                raise ValueError("Cleanup swallowed an original face, hand or bare arm")


def run(ids, destination, check=False):
    poses = json.loads((HERE / "sources.json").read_text())["poses"]
    chosen = [pose for pose in poses if not ids or "dwarf-" + pose in ids]
    if ids and len(chosen) != len(set(ids)):
        raise ValueError("Unknown base; advanced tiers and the rune line are excluded")
    matrices = []
    for pose in chosen:
        unit = "dwarf-" + pose
        source, out = HERE / "bases" / unit, destination / unit
        rows = (source / "player.mask").read_text().splitlines()
        original = (HERE.parent / "dwarf-progression/bases" / unit / "player.txt").read_text().splitlines()
        if len(rows) != 64 or any(len(row) != 64 for row in rows):
            raise ValueError("Base must retain its 64×64 native canvas")
        if any((a == ".") != (b == ".") for ra, rb in zip(rows, original) for a, b in zip(ra, rb)):
            raise ValueError("Cleanup changes the approved pose silhouette")
        palette = json.loads((source / "palette.json").read_text())
        parts = json.loads((source / "parts.json").read_text())
        verify_parts(rows, parts, (source / "parts.mask").read_text())
        verify_anchors(rows, json.loads((source / "anchors.json").read_text()))
        if "rider" not in pose:
            verify_original_details(original, rows, json.loads((source / "reviewed-specks.json").read_text()))
        elif any(c == "O" and rows[y][x] != "O"
                 for y, row in enumerate(original) for x, c in enumerate(row)):
            raise ValueError("Mounted base erases an original contour/facial feature")
        if set(palette) != set("".join(rows) + "Rr"):
            raise ValueError("Palette must contain only the used colours and enemy cloth")
        player = "\n".join(rows) + "\n"
        enemy = player.translate(str.maketrans("Tt", "Rr"))
        edits = [{"x": x, "y": y, "from": original[y][x], "to": c}
                 for y, row in enumerate(rows) for x, c in enumerate(row) if c != original[y][x]]
        files = {"player.txt": player, "enemy.txt": enemy,
                 "palette.json": json.dumps(palette, indent=2) + "\n",
                 "changes.json": json.dumps({"base": unit, "source": "dwarf-base-cleanup/bases/" + unit,
                                               "level": 1, "changes": edits}, indent=2) + "\n",
                 "components.md": f"# {unit}\n\nRestored pre-progression equipment and pose, with flat material colours, "
                 "simple shading and reviewed surface boundaries. Face and hands share the skin ramp; "
                 "leather, beard and wood share the brown ramp. Cream cloth, faction cloth and steel "
                 "have separate ramps.\n\nThe native source and detected fill regions live in "
                 f"`art/pixel/reference/dwarf-base-cleanup/bases/{unit}/`. "
                 "Run that folder's `build.py` from the project root. Advanced tiers are deferred; "
                 "the rune line is excluded.\n"}
        if not check:
            out.mkdir(parents=True, exist_ok=True)
        for name, contents in files.items():
            path = out / name
            if check:
                current = path.read_text()
                equal = json.loads(current) == json.loads(contents) if name.endswith(".json") else current == contents
                if not equal:
                    raise ValueError(f"{unit}/{name}: rebuild the base")
            else:
                path.write_text(contents)
        matrices.extend(out / f"{side}.txt" for side in ("player", "enemy"))
    subprocess.run([sys.executable, str(PROJECT / "scripts/render-pixel.py"),
                    *(str(path) for path in matrices), *(["--check"] if check else [])],
                   cwd=PROJECT, check=True, stdout=subprocess.DEVNULL)
    print(f"{'Verified' if check else 'Rebuilt'} {len(chosen)} bases, both factions; visible edge/fill regions verified")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ids", nargs="*")
    parser.add_argument("--output", type=Path, default=UNITS)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--record-parts", action="store_true", help="Record fill regions after a reviewed native source edit")
    args = parser.parse_args()
    if args.record_parts:
        if args.check:
            parser.error("--record-parts cannot be used with --check")
        for directory in sorted((HERE / "bases").iterdir()):
            if args.ids and directory.name not in args.ids:
                continue
            parts, mask = describe((directory / "player.mask").read_text().splitlines())
            grid = [row.split() for row in mask.splitlines()]
            anchors = json.loads((directory / "anchors.json").read_text())
            for anchor in anchors:
                x, y = anchor["seed"]
                anchor["region"] = grid[y][x]
            verify_anchors((directory / "player.mask").read_text().splitlines(), anchors)
            (directory / "parts.json").write_text(json.dumps(parts, indent=2) + "\n")
            (directory / "parts.mask").write_text(mask)
            (directory / "anchors.json").write_text(json.dumps(anchors, indent=2) + "\n")
    run(args.ids, args.output.resolve(), args.check)
