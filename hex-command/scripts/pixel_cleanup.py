"""Replay reviewed material corrections without changing sprite geometry."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UNITS = ROOT / "art" / "pixel" / "units"
RECIPES = ROOT / "art" / "pixel" / "reference" / "cleanup"


def apply_cleanup(matrices, check=False, units_root=UNITS, recipe_root=RECIPES):
    recipes = {}
    for file in sorted(recipe_root.glob("*.json")):
        for unit, recipe in json.loads(file.read_text())["units"].items():
            if unit in recipes:
                raise ValueError(f"Duplicate cleanup ownership: {unit}")
            recipes[unit] = recipe

    palettes = set()
    for matrix in matrices:
        if matrix.parent.parent != units_root or matrix.stem not in ("player", "enemy"):
            continue
        unit = matrix.parent.name
        recipe = recipes.get(unit)
        if recipe is None:
            continue
        rows = [list(row) for row in matrix.read_text().splitlines()]
        if "canvas" in recipe and recipe["canvas"] != [len(rows[0]), len(rows)]:
            raise ValueError(f"{unit}: cleanup requires the reviewed native canvas")
        changes = recipe["matrixChanges" if matrix.stem == "player" else "enemyMatrixChanges"]
        for change in changes:
            x, y = change["x"], change["y"]
            before, after = change["from"], change["to"]
            if (before == ".") != (after == "."):
                raise ValueError(f"{unit}: material cleanup must preserve transparency")
            if (before == "O") != (after == "O"):
                interior = (
                    change.get("removeInteriorOutline") is True
                    and before == "O"
                    and 0 < x < len(rows[0]) - 1
                    and 0 < y < len(rows) - 1
                    and all(
                        rows[yy][xx] != "."
                        for yy in range(y - 1, y + 2)
                        for xx in range(x - 1, x + 2)
                    )
                )
                boundary = (
                    change.get("repairMaterialBoundary") is True
                    and before == "O"
                    and 0 < x < len(rows[0]) - 1
                    and 0 < y < len(rows) - 1
                    and sum(
                        rows[yy][xx] != "."
                        for yy in range(y - 1, y + 2)
                        for xx in range(x - 1, x + 2)
                    ) >= 7
                    and sum(
                        rows[yy][xx] != "."
                        for xx, yy in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1))
                    ) >= 3
                )
                if not (interior or boundary):
                    raise ValueError(f"{unit}: material cleanup must preserve outlines")
            current = rows[y][x]
            if current == after:
                continue
            if current != before or check:
                raise ValueError(f"{unit}/{matrix.stem} ({x},{y}): stale cleanup source")
            rows[y][x] = after
        result = "\n".join("".join(row) for row in rows) + "\n"
        if not check and result != matrix.read_text():
            matrix.write_text(result)

        if unit in palettes:
            continue
        palettes.add(unit)
        file = matrix.parent / "palette.json"
        palette = json.loads(file.read_text())
        altered = False
        for symbol, change in recipe.get("paletteChanges", {}).items():
            before, after = change["from"], change["to"]
            current = palette.get(symbol)
            if current == after:
                continue
            if current != before or check:
                raise ValueError(f"{unit}/{symbol}: stale cleanup palette")
            if len(after) != 4 or after[3] != 255 or (before is not None and before[3] != 255):
                raise ValueError(f"{unit}/{symbol}: cleanup colors must remain opaque")
            palette[symbol] = after
            altered = True
        if altered:
            file.write_text(json.dumps(palette, indent=2) + "\n")
