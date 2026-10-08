"""Render the actual five dwarf trees, including the postponed rune tree."""

import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parents[3]
lineup = json.loads((PROJECT / "art/pixel/lineups/dwarves.json").read_text())
font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 15)
small = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 12)
title = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 20)

for side in ("player", "enemy"):
    board = Image.new("RGB", (792, 2144), (20, 31, 26))
    draw = ImageDraw.Draw(board)
    draw.text((24, 14), f"Dwarves · {'teal' if side == 'player' else 'red'} faction", font=title, fill=(238, 230, 206))
    for x, text in zip((24, 288, 552), ("L1 starter", "L2 · steel + leather", "L3 · silver + gold")):
        draw.text((x, 47), text, font=font, fill=(202, 205, 190))
    cursor = 84
    for tree in lineup["trees"]:
        postponed = tree.get("status") == "postponed"
        height = 212 if postponed else 460
        draw.line((16, cursor, 776, cursor), fill=(59, 72, 61))
        base = tree["base"]
        base_y = cursor + (24 if postponed else 140)
        image = Image.open(PROJECT / f"art/pixel/units/{base['id']}/{side}.png").convert("RGBA")
        image = image.resize((192, 192), Image.Resampling.NEAREST)
        board.paste(image, (24, base_y), image)
        draw.text((24, cursor + 7), base["name"], font=font, fill=(231, 221, 199))
        if postponed:
            draw.text((288, cursor + 65), "Runesmith · postponed", font=font, fill=(143, 153, 145))
            draw.text((288, cursor + 88), "Magic and support; split later", font=small, fill=(143, 153, 145))
            draw.text((552, cursor + 65), "L3 · postponed", font=font, fill=(143, 153, 145))
        else:
            mid = base_y + 96
            draw.line((216, mid, 252, mid), fill=(126, 142, 126), width=2)
            for index, branch in enumerate(tree["branches"]):
                y = cursor + 34 + index * 212
                center = y + 96
                draw.line((252, mid, 252, center, 278, center), fill=(126, 142, 126), width=2)
                draw.polygon(((278, center), (272, center - 4), (272, center + 4)), fill=(126, 142, 126))
                draw.line((480, center, 542, center), fill=(126, 142, 126), width=2)
                draw.polygon(((542, center), (536, center - 4), (536, center + 4)), fill=(126, 142, 126))
                for x, unit in zip((288, 552), branch["units"]):
                    image = Image.open(PROJECT / f"art/pixel/units/{unit['id']}/{side}.png").convert("RGBA")
                    image = image.resize((192, 192), Image.Resampling.NEAREST)
                    board.paste(image, (x, y), image)
                    draw.text((x, y - 17), unit["name"].replace(" veteran", " · veteran"), font=small, fill=(224, 217, 192))
                draw.text((288, y + 174), branch["purpose"], font=small, fill=(167, 182, 169))
        cursor += height
    board.save(HERE / f"tree-{side}.png")
    print(f"Rendered five starting trees for {side}; rune development postponed")
