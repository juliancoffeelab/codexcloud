"""Render a matrix beside all its detector findings, without filtering findings."""
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location("build_pixel_art", ROOT / "scripts/build-pixel-art.py")
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)
faction_palette = renderer.faction_palette



def write_review(path, findings, output, scale=8):
    output = output.resolve()
    if not output.is_relative_to(ROOT / 'tmp'):
        raise ValueError('Review images belong under project tmp/')
    rows = renderer.read_matrix(path)
    palette = renderer.read_palette(path)
    palette.update(faction_palette('enemy'))
    height, width = len(rows), len(rows[0])
    gap = 16
    panel_width = width * scale
    total_width = panel_width * 2 + gap
    total_height = height * scale
    rgba = bytearray(total_width * total_height * 4)

    def put(x, y, color):
        if 0 <= x < total_width and 0 <= y < total_height:
            i = (y * total_width + x) * 4
            rgba[i:i+4] = bytes(color)

    for y in range(total_height):
        for x in range(total_width):
            color = (22, 32, 28, 255)
            for offset in (0, panel_width + gap):
                local = x - offset
                if 0 <= local < panel_width:
                    checker = (48, 56, 51, 255) if (local // (scale*4) + y // (scale*4)) % 2 else (57, 65, 59, 255)
                    value = palette[rows[y//scale][local//scale]]
                    color = value if value[3] else checker
            put(x, y, color)
    colors = {'unapproved-shade': (255, 125, 0, 255), 'outline-fragment': (0, 220, 255, 255), 'missing-border': (255, 220, 0, 255), 'enclosed-hole': (65, 135, 255, 255),
              'material-leak': (255, 125, 0, 255)}
    for finding in findings:
        color = colors.get(finding.get('kind'), (255, 0, 225, 255))
        x = panel_width + gap + finding['x'] * scale
        y = finding['y'] * scale
        for k in range(scale):
            for thickness in (0, 1):
                put(x+k, y+thickness, color)
                put(x+k, y+scale-1-thickness, color)
                put(x+thickness, y+k, color)
                put(x+scale-1-thickness, y+k, color)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(renderer.png_bytes(total_width, total_height, rgba))
