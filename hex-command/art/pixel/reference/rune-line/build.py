"""Build rune tiers from guarded material regions and approved Stonespell art."""

import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parents[3]
UNITS = PROJECT / "art/pixel/units"
SOURCE = HERE / "bases"
spec = importlib.util.spec_from_file_location("rune_parts", HERE.parent / "dwarf-base-cleanup/build.py")
parts_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(parts_module)
parts_module.OWNER.update({"E": "magic", "z": "stone-border"})
spec = importlib.util.spec_from_file_location("rune_trim", HERE.parent / "dwarf-upgrades/build.py")
trim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(trim)
CRYSTAL = {"E": [122, 198, 227, 255], "e": [67, 147, 175, 255], "J": [181, 229, 244, 255]}
BLUE_HAMMER = {"C": CRYSTAL["E"], "c": CRYSTAL["e"], "K": CRYSTAL["J"]}
GOLD_CRYSTAL = {"E": [239, 179, 57, 255], "e": [175, 108, 29, 255], "J": [255, 234, 151, 255], "z": [126, 76, 25, 255]}
L2_ARMOUR = {"helmet", "left-cuirass", "right-cuirass", "right-bracer", "upper-fastener", "belt-buckle"}


def armoured(part, level):
    return part["role"] in {"garment", "glove", "boot", "fastener"} and (level == 3 or part["name"] in L2_ARMOUR)


def stonespell(level):
    # Approved imagegen design, quantized to native editable pixels. These
    # independent hood/stone shapes cannot be derived by recolouring a helmet.
    rows = (HERE / "designs" / f"stonespell-l{level}.mask").read_text().splitlines()
    palette = json.loads((HERE / "designs/palette.json").read_text())
    if len(rows) != 64 or any(len(row) != 64 for row in rows):
        raise ValueError("Stonespell must retain its native 64x64 frame")
    used = set("".join(rows) + "Rr")
    gold = {(x, y) for y, row in enumerate(rows) for x, c in enumerate(row) if c in "YG"}
    return rows, {k: v for k, v in palette.items() if k in used}, gold


def load():
    rows = (SOURCE / "player.mask").read_text().splitlines()
    original = (SOURCE / "original.mask").read_text().splitlines()
    palette = json.loads((SOURCE / "palette.json").read_text())
    parts = json.loads((SOURCE / "parts.json").read_text())
    parts_module.verify_parts(rows, parts, (SOURCE / "parts.mask").read_text())
    parts_module.verify_anchors(rows, json.loads((SOURCE / "anchors.json").read_text()))
    seams = {(p["x"], p["y"]) for p in json.loads((SOURCE / "seams.json").read_text())}
    flecks = {(p["x"], p["y"]) for p in json.loads((SOURCE / "reviewed-flecks.json").read_text())}
    if len(rows) != 64 or any(len(r) != 64 for r in rows):
        raise ValueError("Rune base must retain its native 64×64 canvas")
    for y, row in enumerate(original):
        for x, before in enumerate(row):
            after = rows[y][x]
            if (before == ".") != (after == ".") or (before == "O" and after != "O"):
                raise ValueError("Cleanup erases original silhouette, eyes or contours")
            if (before != "O" and after == "O") != ((x, y) in seams):
                raise ValueError("Unreviewed internal contour change")
            if before in "Hh" and (x, y) not in flecks and after not in "Hh":
                raise ValueError("Cleanup swallows a face or grip")
            if before in "Ez" and after != before:
                raise ValueError("Cleanup changes the crystal shape")
    roles = json.loads((HERE / "roles.json").read_text())
    if [r["id"] for r in roles] != [p["id"] for p in parts]:
        raise ValueError("Role assignment no longer matches the actual fill regions")
    pixels = {p["id"]: points for p, (_, points) in zip(parts, parts_module.surfaces(rows))}
    return rows, palette, roles, pixels


def derive(source, palette, roles, pixels, branch, level):
    if branch == "stonespell":
        return stonespell(level)
    output = [list(row) for row in source]
    ramp = trim.SILVER if level == 3 else trim.IRON
    result_palette = {**palette, **ramp, **GOLD_CRYSTAL, **BLUE_HAMMER, **(trim.GOLD if level == 3 else {})}
    gold = set()
    for part in roles:
        role, points = part["role"], pixels[part["id"]]
        armour = branch == "thundersmith" and armoured(part, level)
        for x, y in points:
            before = source[y][x]
            if armour:
                output[y][x] = "S" if before == "A" else "p" if before in "bhita" else "P"
            elif branch == "thundersmith" and role == "hammer":
                output[y][x] = "K" if before == "A" else "c" if before == "a" else "C"
        if level == 3 and role in {"garment", "glove", "boot", "faction-cloth", "fastener"}:
            stroke = points if role == "fastener" else trim.band(points, limit=min(len(points) // 4, 8), minimum=4)
            if role != "fastener" and (len(points) < 12 or len(stroke) < 2):
                stroke = []
            for index, (x, y) in enumerate(stroke):
                output[y][x] = "Y" if index == 0 and len(stroke) > 1 else "G"
                gold.add((x, y))
    rows = ["".join(r) for r in output]
    verify(source, palette, rows, result_palette, roles, pixels, branch, level, gold)
    used = set("".join(rows) + "Rr")
    return rows, {k: v for k, v in result_palette.items() if k in used}, gold


def verify(source, source_palette, rows, palette, roles, pixels, branch, level, gold):
    if branch == "stonespell":
        expected_rows, expected_palette, expected_gold = stonespell(level)
        if rows != expected_rows or palette != expected_palette or gold != expected_gold:
            raise ValueError("Stonespell departs from the approved hood, robe or stone design")
        return
    for y, row in enumerate(source):
        for x, before in enumerate(row):
            after = rows[y][x]
            if (before == ".") != (after == ".") or (before == "O") != (after == "O"):
                raise ValueError("Branch changes eyes, contours or silhouette")
    for part in roles:
        role = part["role"]
        for x, y in pixels[part["id"]]:
            before = source[y][x]
            c = rows[y][x]
            if (x, y) in gold:
                if level != 3 or c not in "YG" or palette[c] != trim.GOLD[c]:
                    raise ValueError("Gold trim uses the wrong material")
            elif branch == "thundersmith" and armoured(part, level):
                if c not in "PpS" or palette[c] != (trim.SILVER if level == 3 else trim.IRON)[c]:
                    raise ValueError(f"Incomplete armour on {part['name']}")
            elif branch == "thundersmith" and role == "hammer":
                if c not in "CcK" or palette[c] != BLUE_HAMMER[c]:
                    raise ValueError("Hammer is not completely blue crystal")
            elif before in "Ez":
                if c != before or palette[c] != GOLD_CRYSTAL[c]:
                    raise ValueError(f"Branch changes protected crystal shape or golden material on {part['name']}")
            elif c != source[y][x] or palette[c] != source_palette[c]:
                raise ValueError(f"Branch recolours protected {part['name']}")


def write_unit(unit, source, rows, palette, check, branch=None, level=1):
    output = UNITS / unit
    player = "\n".join(rows) + "\n"
    enemy = player.translate(str.maketrans("Tt", "Rr"))
    edits = [{"x": x, "y": y, "from": source[y][x], "to": c}
             for y, row in enumerate(rows) for x, c in enumerate(row) if c != source[y][x]]
    files = {"player.txt": player, "enemy.txt": enemy,
             "palette.json": json.dumps(palette, indent=2) + "\n",
             "changes.json": json.dumps({"base": "dwarf-runesmith", "level": level, "branch": branch,
                                          "source": f"rune-line/designs/stonespell-l{level}.mask" if branch == "stonespell" else "rune-line/bases/player.mask", "changes": edits}, indent=2) + "\n",
             "components.md": f"# {unit}\n\nNative rune-line artwork from `art/pixel/reference/rune-line/`. "
             "Face, gripping hand, beard and wooden shafts keep their own materials. "
             "The pointed apron, torso, overskirt, trousers and boots have independent fill seeds. "
             "Thundersmith uses a sun-gold hand crystal and crystal-blue hammer, partial steel L2 armour with leather lower garments, "
             "and full silver-and-gold L3 armour. Stonespell uses a heavy slate hood and robes; "
             "L3 adds stone shoulders, restrained edging and a crystal-blue rune tablet. "
             "Faction cloth swaps teal to red; stone and crystal colours remain shared.\n\n"
             "Run `python3 art/pixel/reference/rune-line/build.py` to rebuild, or add `--check` to verify.\n"}
    for name, contents in files.items():
        path = output / name
        if check:
            current = path.read_text()
            if not (json.loads(current) == json.loads(contents) if name.endswith(".json") else current == contents):
                raise ValueError(f"Rebuild {unit}/{name}")
        else:
            output.mkdir(parents=True, exist_ok=True)
            path.write_text(contents)
    return [output / f"{side}.txt" for side in ("player", "enemy")]


def run(check=False):
    source, palette, roles, pixels = load()
    paths = write_unit("dwarf-runesmith", (SOURCE / "original.mask").read_text().splitlines(), source, palette, check)
    coverage = {}
    for branch in ("thundersmith", "stonespell"):
        for level in (2, 3):
            unit = f"dwarf-{branch}-l{level}"
            rows, result_palette, gold = derive(source, palette, roles, pixels, branch, level)
            paths += write_unit(unit, source, rows, result_palette, check, branch, level)
            coverage[unit] = {"goldPixels": len(gold), "source": "approved native design" if branch == "stonespell" else "guarded apprentice parts",
                              "partialArmour": branch == "thundersmith" and level == 2}
    path = HERE / "coverage.json"
    if check:
        if json.loads(path.read_text()) != coverage:
            raise ValueError("Rebuild rune coverage report")
    else:
        path.write_text(json.dumps(coverage, indent=2) + "\n")
    subprocess.run([sys.executable, str(PROJECT / "scripts/render-pixel.py"), *(str(p) for p in paths),
                    *(["--check"] if check else [])], cwd=PROJECT, check=True, stdout=subprocess.DEVNULL)
    print(f"{'Verified' if check else 'Rebuilt'} rune apprentice and four branch tiers, both factions; fill coverage and protected features checked")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    run(parser.parse_args().check)
