"""Flag opaque pixels with no equal RGBA colour at left/right/up/down."""

import argparse
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "scripts"))
from pixel_sources import faction_palette


def isolated(rows, palette, ignore_exterior_black=True, allowed_shades=None, unapproved_shades=()):
    height, width = len(rows), len(rows[0])
    found = []
    material_groups = [set(symbols) for symbols in (allowed_shades or {}).values()]
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            color = palette[symbol]
            if not color[3]:
                continue
            if symbol in unapproved_shades:
                found.append({"x": x, "y": y, "symbol": symbol, "rgba": color,
                              "kind": "unapproved-shade"})
                continue
            neighbors = [(xx, yy) for xx, yy in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1))
                         if 0 <= xx < width and 0 <= yy < height]
            if (ignore_exterior_black and max(color[:3]) <= 40
                    and any(palette[rows[yy][xx]][3] == 0 for xx, yy in neighbors)):
                continue
            matching_material = any(
                symbol in group and any(rows[yy][xx] in group for xx, yy in neighbors)
                for group in material_groups)
            if not matching_material and not any(palette[rows[yy][xx]] == color for xx, yy in neighbors):
                found.append({"x": x, "y": y, "symbol": symbol, "rgba": color})
    return found


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("matrices", nargs="+", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--images-dir", type=Path, help="Default: beside each matrix")
    parser.add_argument("--include-exterior-black", action="store_true",
                        help="Also flag black silhouette pixels beside transparency")
    parser.add_argument("--allowed-shades", nargs="?", type=Path,
                        const=Path(__file__).resolve().parents[1] / "references/elf-shades.json",
                        help="Use the elf material policy, or supply a policy JSON; default is strict exact colour")
    args = parser.parse_args()
    policy = json.loads(args.allowed_shades.read_text()) if args.allowed_shades else {}
    groups = policy.get("allowed_shades", {})
    unapproved = {symbol for material in policy.get("unshaded_materials", {}).values()
                  for symbol in material.get("extra_shades", [])}
    report = {}
    for path in args.matrices:
        palette_path = path.parent / "palette.json"
        if not palette_path.exists():
            palette_path = ROOT / "art/pixel/palette.json"
        palette = json.loads(palette_path.read_text())
        if path.stem == "sprite":
            palette.update(faction_palette("debug"))
        flags = isolated(path.read_text().splitlines(), palette, not args.include_exterior_black,
                         allowed_shades=groups, unapproved_shades=unapproved)
        report[str(path)] = flags
        from pixel_review_image import write_review
        image_dir = args.images_dir or path.parent
        image_path = image_dir / ((path.parent.name + "-" if args.images_dir else "") + path.stem +
                                  ("-material-strays.png" if args.allowed_shades else "-strays.png"))
        write_review(path, flags, image_path)
        print(f"Image: {image_path} (left: original; right: ALL flags)")
        print(f"{path.parent.name}/{path.name}: {len(flags)} suspicious isolated pixels")
    if args.output:
        args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(f"Total: {sum(len(flags) for flags in report.values())}; eyes and intentional details require visual review")
