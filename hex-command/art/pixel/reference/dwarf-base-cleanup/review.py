"""Render the base board, independent region fills and cardinal-neighbour flags."""

import colorsys
import importlib.util
import json
from pathlib import Path
import subprocess
from PIL import Image, ImageDraw, ImageFont

from build import HERE, PROJECT, source_directory, surfaces, verify_anchors

spec = importlib.util.spec_from_file_location("isolated_pixels", PROJECT / "scripts/check_isolated_pixels.py")
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)

meta = json.loads((HERE / "sources.json").read_text())
scale, cell, step, row_height = 4, 256, 272, 308
size = (832, 940)
fill_board = Image.new("RGB", size, (20, 31, 26))
flag_board = fill_board.copy()
font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 15)
report = {}
for i, pose in enumerate(meta["poses"]):
    unit = "dwarf-" + pose
    directory = source_directory(unit)
    rows = (directory / "player.txt").read_text().splitlines()
    palette = json.loads((directory / "palette.json").read_text())
    verify_anchors(rows, json.loads((HERE / "bases" / unit / "anchors.json").read_text()))
    filled = Image.new("RGBA", (64, 64))
    filled.putdata([tuple(palette[c]) for row in rows for c in row])
    # Fill every detected field, with no inference from an armour rectangle.
    for number, (_, pixels) in enumerate(surfaces(rows)):
        rgb = tuple(round(c * 255) for c in colorsys.hsv_to_rgb((number * 0.381966) % 1, 0.57, 0.96))
        for x, y in pixels:
            filled.putpixel((x, y), (*rgb, 255))
    actual = Image.open(directory / "player.png").convert("RGBA")
    bx, by = 16 + i % 3 * step, 44 + i // 3 * row_height
    flags = checker.isolated(rows, palette)
    report[str((directory / "player.txt").relative_to(PROJECT))] = flags
    for board, frame in ((fill_board, filled), (flag_board, actual)):
        draw = ImageDraw.Draw(board)
        for y in range(0, cell, 32):
            for x in range(0, cell, 32):
                color = (58, 64, 60) if (x + y) // 32 % 2 else (43, 50, 46)
                draw.rectangle((bx + x, by + y, bx + x + 31, by + y + 31), fill=color)
        large = frame.resize((cell, cell), Image.Resampling.NEAREST)
        board.paste(large, (bx, by), large)
        title = pose.replace("-", " ")
        if board is flag_board:
            title += f" · {len(flags)} flags"
        draw.text((bx, by - 29), title, font=font, fill=(225, 221, 200))
    draw = ImageDraw.Draw(flag_board)
    for flag in flags:
        x, y = bx + flag["x"] * scale, by + flag["y"] * scale
        draw.rectangle((x, y, x + scale - 1, y + scale - 1), outline=(255, 55, 82), width=1)
fill_board.save(HERE / "fill-regions.png")
flag_board.save(HERE / "isolated-pixels.png")
(HERE / "isolated-pixels.json").write_text(json.dumps(report, indent=2) + "\n")
subprocess.run(["node", "art/pixel/reference/human-anatomies/preview.cjs",
                str(HERE / "lineup.json"), str(HERE / "lineup-player.png"), "3", "4"], cwd=PROJECT, check=True)
print(f"Rendered independent fills and {sum(len(p) for p in report.values())} remaining suspect pixels; exterior black is excluded")
