"""Read-only L1/L2/L3 race grids with one shared native pixel scale.

Usage: python3 .agents/skills/pixel-lineup/scripts/preview.py --race humans
       python3 .agents/skills/pixel-lineup/scripts/preview.py --race beasts --no-names --no-boxes
       python3 .agents/skills/pixel-lineup/scripts/preview.py --list
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
    render_grid(args.race, units, args.output, args.scale, args.rows, args.side, args.names, args.boxes)


if __name__ == "__main__":
    main()
