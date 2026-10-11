"""Read-only L1/L2/L3 race grids or trees with one shared native pixel scale.

Usage: python3 .agents/skills/pixel-lineup/scripts/preview.py --race humans
       python3 .agents/skills/pixel-lineup/scripts/preview.py --race beasts --no-names --no-boxes
       python3 .agents/skills/pixel-lineup/scripts/preview.py --list
       python3 .agents/skills/pixel-lineup/scripts/preview.py --race humans --layout tree
No artwork builds, cleanup recipes or source writes are performed.
"""

import argparse
import base64
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location("build_pixel_atlas", ROOT / "scripts/build-pixel-atlas.py")
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)
LINEUPS = renderer.load_lineups()
GROUPS = {race: race.title() for race in LINEUPS}


def inventory(side):
    units = []
    directories = renderer.unit_directories(LINEUPS)
    for race, lineup in LINEUPS.items():
        for entry in renderer.entries(lineup):
            matrix = directories[entry["id"]] / "sprite.txt"
            width, height, pixels = renderer.rgba_pixels(matrix, side)
            units.append({**entry, "race": race, "width": width, "height": height,
                          "sha256": hashlib.sha256(matrix.read_bytes()).hexdigest(),
                          "image": "data:image/png;base64," + base64.b64encode(renderer.png_bytes(width, height, pixels)).decode()})
    return units


def lineup_rows(race, units):
    records = {unit["id"]: unit for unit in units}
    return [[{**records[entry["id"]], **entry} for entry in row] for row in renderer.rows(LINEUPS[race])]


def tree_groups(lineup, units):
    """One shared starter per family; edges follow the declared branches."""
    records = {unit["id"]: unit for unit in units}
    groups = []

    def node(entry, column, row):
        return {"unit": {**records[entry["id"]], **entry}, "column": column, "row": row}

    if "commander" in lineup:
        groups.append({"rows": 1,
                       "nodes": [node(u, i, 0) for i, u in enumerate(lineup["commander"]["units"])],
                       "edges": [(0, 1), (1, 2)]})
    for tree in lineup["trees"]:
        count = len(tree["branches"])
        nodes = [node(tree["base"], 0, (count - 1) / 2)]
        edges = []
        for row, branch in enumerate(tree["branches"]):
            parent = 0
            for entry in branch["units"]:
                child = len(nodes)
                nodes.append(node(entry, entry["level"] - 1, row))
                edges.append((parent, child))
                parent = child
        groups.append({"rows": count, "nodes": nodes, "edges": edges})
    return groups


def tree_pages(groups, row_limit):
    """Keep an entire family together, even when it exceeds the page limit."""
    pages, page, rows = [], [], 0
    for group in groups:
        if page and row_limit and rows + group["rows"] > row_limit:
            pages.append(page)
            page, rows = [], 0
        page.append(group)
        rows += group["rows"]
    if page:
        pages.append(page)
    return pages


def render_tree(race, units, output, scale, rows_per_page, side, names=True, boxes=True):
    from io import BytesIO
    from PIL import Image, ImageDraw, ImageFont

    output = output.resolve()
    if not output.is_relative_to(ROOT / "tmp"):
        raise ValueError("Lineup previews belong under project tmp/")
    pages = tree_pages(tree_groups(LINEUPS[race], units), rows_per_page)
    cell, border = 64 * scale, scale if boxes else 0
    box = cell + 2 * border
    gap, margin = 14 * scale, 2 * scale
    caption = 8 * scale if names else 0
    header = 10 * scale if names else 0
    step = box + caption + 6 * scale
    font = ImageFont.truetype("DejaVuSans.ttf", 5 * scale) if names else None
    checker = Image.new("RGBA", (64, 64))
    checker.putdata([(58, 64, 60, 255) if (x // 8 + y // 8) % 2 else (43, 50, 46, 255)
                     for y in range(64) for x in range(64)])
    report = {"race": race, "side": side, "frame": [64, 64], "pixelScale": scale,
              "layout": "Shared L1 roots with L2 / L3 branch arrows", "names": names,
              "boxes": boxes, "sourceFilesChanged": False, "pages": []}
    output.mkdir(parents=True, exist_ok=True)
    for number, page in enumerate(pages, 1):
        sheet = Image.new("RGB", (3 * box + 2 * gap + 2 * margin,
                                  sum(g["rows"] for g in page) * step - 6 * scale + header + 2 * margin),
                          (29, 36, 30))
        draw = ImageDraw.Draw(sheet)
        if names:
            for col in range(3):
                draw.text((margin + col * (box + gap) + box / 2, margin + header / 2),
                          f"L{col + 1}", font=font, fill=(220, 225, 215), anchor="mm")
        offset, manifest = 0, []
        for group in page:
            positions = [(margin + n["column"] * (box + gap),
                          margin + header + int((offset + n["row"]) * step)) for n in group["nodes"]]
            for parent, child in group["edges"]:
                px, py = positions[parent]; cx, cy = positions[child]
                start, end = px + box + scale, cx - 2 * scale
                middle = (start + end) // 2
                a, b = py + box // 2, cy + box // 2
                color = (139, 182, 155)
                draw.line([(start, a), (middle, a), (middle, b), (end, b)], fill=color, width=scale)
                draw.polygon([(end, b), (end - 3 * scale, b - 2 * scale),
                              (end - 3 * scale, b + 2 * scale)], fill=color)
            for node, (x, y) in zip(group["nodes"], positions):
                unit = node["unit"]
                frame = checker.copy()
                png = base64.b64decode(unit["image"].split(",", 1)[1])
                frame.alpha_composite(Image.open(BytesIO(png)).convert("RGBA"))
                sheet.paste(frame.resize((cell, cell), Image.Resampling.NEAREST), (x + border, y + border))
                if boxes:
                    draw.rectangle((x, y, x + box - 1, y + box - 1), outline=(89, 102, 91), width=border)
                if names:
                    label = re.sub(r"\s+L[123]$", "", unit["name"])
                    label_font = font
                    while draw.textlength(label, font=label_font) > box and label_font.size > 3 * scale:
                        label_font = ImageFont.truetype("DejaVuSans.ttf", label_font.size - 1)
                    draw.text((x + box / 2, y + box + caption / 2), label,
                              font=label_font, fill=(220, 225, 215), anchor="mm")
            manifest.append({"nodes": [{k: n["unit"][k] for k in ("id", "width", "height", "sha256")}
                                       for n in group["nodes"]], "edges": group["edges"]})
            offset += group["rows"]
        path = output / (f"{race}.png" if len(pages) == 1 else f"{race}-{number:02}.png")
        sheet.save(path)
        report["pages"].append({"image": path.name, "trees": manifest})
        print(f"Tree: {path}")
    (output / f"{race}.json").write_text(json.dumps(report, indent=2) + "\n")


def render_grid(race, units, output, scale, rows_per_page, side, names=True, boxes=True):
    from io import BytesIO
    from PIL import Image, ImageDraw, ImageFont

    output = output.resolve()
    if not output.is_relative_to(ROOT / "tmp"):
        raise ValueError("Lineup previews belong under project tmp/")
    rows = lineup_rows(race, units)
    cell = 64 * scale
    border = scale if boxes else 0
    box = cell + 2 * border
    gap = 4 * scale if names or boxes else 0
    margin = 2 * scale if names or boxes else 0
    caption_height = 8 * scale if names else 0
    font = ImageFont.truetype("DejaVuSans.ttf", 5 * scale) if names else None
    checker = Image.new("RGBA", (64, 64))
    checker.putdata([
        (58, 64, 60, 255) if (x // 8 + y // 8) % 2 else (43, 50, 46, 255)
        for y in range(64) for x in range(64)
    ])
    rows_per_page = rows_per_page or len(rows)
    pages = [rows[i:i + rows_per_page] for i in range(0, len(rows), rows_per_page)]
    output.mkdir(parents=True, exist_ok=True)
    report = {
        "race": race, "side": side, "frame": [64, 64], "pixelScale": scale,
        "layout": "One branch per row; L1 / L2 / L3 columns",
        "names": names, "boxes": boxes,
        "sourceFilesChanged": False, "pages": [],
    }
    for number, page in enumerate(pages, 1):
        sheet = Image.new("RGB", (
            3 * box + 2 * gap + 2 * margin,
            len(page) * (box + caption_height + gap) - gap + 2 * margin,
        ), (29, 36, 30))
        draw = ImageDraw.Draw(sheet)
        for y, row in enumerate(page):
            for x, unit in enumerate(row):
                outer_x = margin + x * (box + gap)
                outer_y = margin + y * (box + caption_height + gap)
                bx, by = outer_x + border, outer_y + border
                frame = checker.copy()
                if unit:
                    png = base64.b64decode(unit["image"].split(",", 1)[1])
                    frame.alpha_composite(Image.open(BytesIO(png)).convert("RGBA"))
                sheet.paste(frame.resize((cell, cell), Image.Resampling.NEAREST), (bx, by))
                if boxes:
                    draw.rectangle(
                        (outer_x, outer_y, outer_x + box - 1, outer_y + box - 1),
                        outline=(89, 102, 91), width=border,
                    )
                if names and unit:
                    name = re.sub(r"\s+L[123]$", "", unit["name"])
                    caption = f"{name} L{unit['level']}"
                    label_font = font
                    while draw.textlength(caption, font=label_font) > box and label_font.size > 3 * scale:
                        label_font = ImageFont.truetype("DejaVuSans.ttf", label_font.size - 1)
                    draw.text(
                        (outer_x + box / 2, outer_y + box + caption_height / 2),
                        caption, font=label_font, fill=(220, 225, 215), anchor="mm",
                    )
        filename = f"{race}.png" if len(pages) == 1 else f"{race}-{number:02}.png"
        path = output / filename
        sheet.save(path)
        report["pages"].append({
            "image": path.name,
            "rows": [[{k: u[k] for k in ("id", "width", "height", "sha256")} if u else None for u in row] for row in page],
        })
        print(f"Grid: {path}")
    (output / f"{race}.json").write_text(json.dumps(report, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--race", help="One race to render; see --list")
    parser.add_argument("--side", choices=("player", "enemy"), default="enemy")
    parser.add_argument("--layout", choices=("grid", "tree"), default="grid",
                        help="Tree connects shared starters to their branches; grid repeats starters")
    parser.add_argument("--scale", type=int, default=4, help="Same integer pixel scale for every unit")
    parser.add_argument("--rows", type=int, default=0, help="Optional row limit per image; default is the entire race")
    parser.add_argument("--output", type=Path, default=ROOT / "tmp/unit-scale-grid")
    parser.add_argument("--list", action="store_true", help="List races and actual source sizes")
    parser.add_argument("--names", action=argparse.BooleanOptionalAction, default=True,
                        help="Show unit names and levels below tiles (default: enabled)")
    parser.add_argument("--boxes", action=argparse.BooleanOptionalAction, default=True,
                        help="Outline every tile without covering sprite pixels (default: enabled)")
    args = parser.parse_args()
    units = inventory(args.side)
    for group, name in GROUPS.items():
        if not args.list and group != args.race:
            continue
        sizes = Counter((u["width"], u["height"]) for u in units if u["race"] == group)
        if sizes:
            print(f"{group}: {name}: " + ", ".join(f"{n} × {w}×{h}" for (w, h), n in sorted(sizes.items())))
    if args.list:
        return
    if not args.race:
        parser.error("Choose one race with --race; use --list to see the choices")
    if args.race not in {u["race"] for u in units}:
        parser.error(f"Unknown race: {args.race}; use --list")
    if not 1 <= args.scale <= 16 or not 0 <= args.rows <= 100:
        parser.error("--scale must be 1..16 and --rows must be 0..100")
    render = render_tree if args.layout == "tree" else render_grid
    render(args.race, units, args.output, args.scale, args.rows, args.side, args.names, args.boxes)


if __name__ == "__main__":
    main()
