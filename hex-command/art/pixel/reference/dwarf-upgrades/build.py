"""Fill complete cleaned dwarf surfaces for L2/L3, preserving L1 and both factions."""

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parents[3]
sys.path.insert(0, str(PROJECT / "scripts"))
from pixel_sources import legacy_text, recipe_matches
UNITS = PROJECT / "art/pixel/units"
BASES = HERE.parent / "dwarf-base-cleanup"
spec = importlib.util.spec_from_file_location("dwarf_bases", BASES / "build.py")
bases = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bases)
IRON = {"P": [151, 173, 188, 255], "p": [84, 111, 128, 255], "S": [210, 226, 236, 255]}
SILVER = {"P": [218, 231, 235, 255], "p": [129, 151, 160, 255], "S": [249, 251, 245, 255]}
GOLD = {"G": [218, 169, 62, 255], "g": [133, 86, 30, 255], "Y": [252, 222, 118, 255]}
LEATHER = {"L": [111, 72, 43, 255], "l": [64, 40, 28, 255]}
ARMOUR = {"helmet", "upper", "lower", "shield"}


def branches():
    lineup = json.loads((PROJECT / "art/pixel/lineups/dwarves.json").read_text())
    return {branch["units"][0]["id"].removesuffix("-l2"): tree["base"]["id"]
            for tree in lineup["trees"] if tree.get("status") != "postponed" and tree["base"]["id"] != "dwarf-runesmith"
            for branch in tree["branches"]}


def load(unit):
    directory = BASES / "bases" / unit
    rows = (directory / "player.mask").read_text().splitlines()
    palette = json.loads(legacy_text((directory / "palette.json")))
    starter = BASES / "bases" / branches()[unit]
    starter_rows = (starter / "player.mask").read_text().splitlines()
    starter_palette = json.loads(legacy_text((starter / "palette.json")))
    if (len(rows), len(rows[0])) != (len(starter_rows), len(starter_rows[0])):
        raise ValueError("Branch reference changes its starter's native scale")
    if any(palette[c] != starter_palette[c] for c in palette):
        raise ValueError("Branch clothing and materials depart from their cleaned starter")
    parts = json.loads((directory / "parts.json").read_text())
    bases.verify_parts(rows, parts, (directory / "parts.mask").read_text())
    anchors = json.loads((directory / "anchors.json").read_text())
    bases.verify_anchors(rows, anchors)
    record = json.loads((HERE / "regions.json").read_text())[unit]
    assigned = record["surfaces"]
    if (record["baseMatrixSha256"] != hashlib.sha256((directory / "player.mask").read_bytes()).hexdigest()
            or [p["id"] for p in assigned] != [p["id"] for p in parts]):
        raise ValueError(f"{unit}: surface assignment no longer matches the cleaned base")
    roles = {p["id"]: p["role"] for p in assigned}
    for anchor in anchors:
        name, role = anchor["name"], roles[anchor["region"]]
        if (name == "helmet" or any(word in name for word in ("cuirass", "bracer", "wrist", "boot", "leg", "trouser", "apron", "shirt", "cuff", "shoulder"))) and role not in ARMOUR:
            raise ValueError(f"Uncovered wearer part: {unit}/{name}")
        if name in {"face", "beard", "shaft", "stock", "horn", "mane", "fur", "saddle", "barrel", "bow-steel"} or "hand" in name or "head" in name:
            if role != "protected":
                raise ValueError(f"Armour assignment invades {unit}/{name}")
    pixels = {p["id"]: points for p, (_, points) in zip(parts, bases.surfaces(rows))}
    return rows, palette, assigned, pixels


def band(points, top=False, limit=None, minimum=5):
    """One contiguous inset band inside the actual surface, never across a seam."""
    by_row = {}
    for x, y in points:
        by_row.setdefault(y, []).append(x)
    for y in sorted(by_row, reverse=not top):
        runs, run = [], []
        for x in sorted(by_row[y]):
            if run and x != run[-1] + 1:
                runs.append(run)
                run = []
            run.append(x)
        runs.append(run)
        run = max(runs, key=len)
        if len(run) >= minimum:
            run = run[1:-1]
            if limit:
                run = run[:limit]
            return [(x, y) for x in run]
    return []


def derive(source, palette, parts, pixels, level, inlays=()):
    rows = [list(row) for row in source]
    ramp = SILVER if level == 3 else IRON
    output_palette = {**palette, **ramp, **LEATHER, **(GOLD if level == 3 else {})}
    ornaments = set()
    for part in parts:
        role, points = part["role"], pixels[part["id"]]
        metal = role in {"helmet", "upper"} or (level == 3 and role in ARMOUR)
        leather = role in {"lower", "shield"} and level == 2
        for x, y in points:
            before = source[y][x]
            if metal:
                rows[y][x] = "S" if before == "A" else "p" if before in "bhita" else "P"
            elif leather:
                rows[y][x] = "l" if before in "bhita" else "L"
        if level == 3 and (role in ARMOUR or role == "cloth") and len(points) >= (12 if role == "cloth" else 18):
            # Gold hems and embroidery stay wholly within the selected fill.
            budget = min(len(points) // 4, 5) if role == "cloth" else len(points) // 4
            stroke = band(points, top=role == "shield", limit=budget, minimum=4 if role == "cloth" else 5)
            if role == "cloth" and not stroke:
                stroke = [(y, x) for x, y in band([(y, x) for x, y in points], top=True, limit=budget, minimum=4)]
            # Keep each piece mostly silver/faction colour.
            if len(stroke) <= len(points) // 4:
                for number, (x, y) in enumerate(stroke):
                    rows[y][x] = "Y" if number == 0 else "G"
                    ornaments.add((x, y))
    if level == 3:
        for x, y in inlays:
            if source[y][x] not in "BbIi":
                raise ValueError("Gold inlay no longer matches the repaired garment")
            rows[y][x] = "G"
            ornaments.add((x, y))
    rows = ["".join(row) for row in rows]
    verify(source, palette, rows, output_palette, parts, pixels, level, ornaments, inlays)
    used = set("".join(rows) + "Rr")
    output_palette = {k: v for k, v in output_palette.items() if k in used}
    return rows, output_palette, ornaments


def verify(source, source_palette, rows, palette, parts, pixels, level, ornaments, inlays=()):
    if len(rows) != len(source) or any(len(a) != len(b) for a, b in zip(rows, source)):
        raise ValueError("Upgrade changes the native canvas")
    for y, row in enumerate(source):
        for x, before in enumerate(row):
            after = rows[y][x]
            if (before == ".") != (after == ".") or (before == "O") != (after == "O"):
                raise ValueError("Upgrade erases a contour, eye or silhouette")
    for part in parts:
        role = part["role"]
        for x, y in pixels[part["id"]]:
            symbol = rows[y][x]
            if level == 3 and (x, y) in inlays:
                if symbol != "G" or palette[symbol] != GOLD[symbol]:
                    raise ValueError("Repaired skin fleck is missing its L3 gold inlay")
                continue
            if role == "protected" or (role == "cloth" and (x, y) not in ornaments):
                if symbol != source[y][x] or palette[symbol] != source_palette[symbol]:
                    raise ValueError(f"Upgrade recolours protected {part['name']}")
            elif role in ARMOUR:
                expected = "PpSYG" if level == 3 else "PpS" if role in {"helmet", "upper"} else "Ll"
                if symbol not in expected:
                    raise ValueError(f"Incomplete {part['name']}: ({x},{y}) {symbol}")
                ramp = {**(SILVER if level == 3 else IRON), **LEATHER, **GOLD}
                if palette[symbol] != ramp[symbol]:
                    raise ValueError("Armour or gold uses the wrong material colour")
            elif role == "cloth":
                if symbol not in "YG":
                    raise ValueError("Embroidery leaves the gold ramp")
            else:
                raise ValueError(f"Unknown role {role}")
    if level == 2 and ornaments:
        raise ValueError("Gold belongs to L3")
    for x, y in ornaments:
        if rows[y][x] not in "YG" or palette[rows[y][x]] != GOLD[rows[y][x]]:
            raise ValueError("Gold ornament uses the wrong material colour")
        if (x, y) not in inlays and not any((xx, yy) in ornaments for xx, yy in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1))):
            raise ValueError("Gold trim contains an isolated pixel")


def run(ids, check=False):
    mapping = json.loads((HERE / "regions.json").read_text())
    parents = branches()
    if set(mapping) != set(parents):
        raise ValueError("Surface recipes must match the eight active L2 branches")
    if set(ids) - (set(mapping) | set(parents.values())):
        raise ValueError("Unknown dwarf pose; rune family is excluded")
    chosen = [unit for unit in mapping if not ids or unit in ids or parents[unit] in ids]
    paths, coverage = [], {}
    for unit in chosen:
        source, source_palette, parts, pixels = load(unit)
        records = json.loads((HERE / "gold-inlays.json").read_text()).get(unit, [])
        inlays = {(p["x"], p["y"]) for p in records}
        reviewed = json.loads((BASES / "bases" / unit / "reviewed-specks.json").read_text())
        if inlays != {(p["x"], p["y"]) for p in reviewed if p.get("feature") == "skin-fleck"}:
            raise ValueError("Gold inlays must match the explicitly repaired skin flecks")
        for level in (2, 3):
            rows, palette, ornaments = derive(source, source_palette, parts, pixels, level, inlays)
            name, out = f"{unit}-l{level}", UNITS / f"{unit}-l{level}"
            player = "\n".join(rows) + "\n"
            enemy = player.translate(str.maketrans("Tt", "Rr"))
            changes = [{"x": x, "y": y, "from": source[y][x], "to": c,
                        "part": p["name"], "region": p["id"]}
                       for p in parts for x, y in pixels[p["id"]] if (c := rows[y][x]) != source[y][x]]
            report = {"base": parents[unit], "poseReference": unit, "level": level, "source": "dwarf-upgrades/regions.json",
                      "changes": changes}
            files = {"player.txt": player, "enemy.txt": enemy,
                     "palette.json": json.dumps(palette, indent=2) + "\n",
                     "changes.json": json.dumps(report, indent=2) + "\n",
                     "components.md": f"# {name}\n\nL{level} branch of `{parents[unit]}`, using the cleaned `{unit}` pose reference "
                     "at the starter's native scale and material palette. Complete detected surfaces are filled. "
                     "L2 has steel upper armour over leather lower armour; L3 has silver armour with gold hems "
                     "and faction-cloth embroidery. Faces, hands, beard, weapon wood/steel, mount and tack "
                     "retain their base materials. Teal cloth becomes red for the enemy; gold is shared. "
                     "L1 and the rune family are untouched.\n\nRebuild with "
                     "`python3 art/pixel/reference/dwarf-upgrades/build.py`; assignments are in that folder's "
                     "`regions.json` and must match the exact detected base surfaces.\n"}
            for filename, value in files.items():
                path = out / filename
                if check:
                    if not recipe_matches(path, value):
                        raise ValueError(f"Rebuild {name}/{filename}")
                else:
                    out.mkdir(parents=True, exist_ok=True)
                    path.write_text(value)
            paths.extend(out / f"{side}.txt" for side in ("player", "enemy"))
            coverage[name] = {"pixelsByRole": {role: sum(len(pixels[p['id']]) for p in parts if p['role'] == role)
                                              for role in sorted({p['role'] for p in parts})},
                              "goldPixels": len(ornaments), "uncoveredArmourPixels": 0}
    subprocess.run([sys.executable, str(PROJECT / "scripts/render-pixel.py"), *(str(p) for p in paths),
                    *(["--check"] if check else [])], cwd=PROJECT, check=True, stdout=subprocess.DEVNULL)
    if not ids:
        path = HERE / "coverage.json"
        if check:
            if json.loads(path.read_text()) != coverage:
                raise ValueError("Rebuild tier coverage")
        else:
            path.write_text(json.dumps(coverage, indent=2) + "\n")
    print(f"{'Verified' if check else 'Rebuilt'} {len(paths) // 2} tiers, both factions; complete surface coverage and protected materials checked")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ids", nargs="*")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    run(args.ids, args.check)
