"""Native nearest-neighbour rune branches, for both factions."""

from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parents[3]
font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 14)
for side in ("player", "enemy"):
    board = Image.new("RGB", (768, 468), (20, 31, 26))
    draw = ImageDraw.Draw(board)
    for row, branch in enumerate(("thundersmith", "stonespell")):
        for column, level in enumerate((1, 2, 3)):
            unit = "dwarf-runesmith" if level == 1 else f"dwarf-{branch}-l{level}"
            x, y = column * 256, row * 234
            draw.text((x + 16, y + 12), "L1 Rune apprentice" if level == 1 else f"L{level} {branch.capitalize()}",
                      fill=(238, 230, 206), font=font)
            sprite = Image.open(PROJECT / f"art/pixel/units/{unit}/{side}.png").convert("RGBA")
            sprite = sprite.resize((256, 256), Image.Resampling.NEAREST)
            board.paste(sprite, (x, y + 7), sprite)
    board.save(HERE / f"lineup-{side}.png")
    print(f"Rendered rune branches for {side}")
