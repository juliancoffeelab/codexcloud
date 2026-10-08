"""Flag opaque pixels with no equal RGBA colour at left/right/up/down."""

import argparse
import json
from pathlib import Path


def isolated(rows, palette, ignore_exterior_black=True):
    height, width = len(rows), len(rows[0])
    found = []
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            color = palette[symbol]
            if not color[3]:
                continue
            neighbors = [(xx, yy) for xx, yy in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1))
                         if 0 <= xx < width and 0 <= yy < height]
            if (ignore_exterior_black and max(color[:3]) <= 40
                    and any(palette[rows[yy][xx]][3] == 0 for xx, yy in neighbors)):
                continue
            if not any(palette[rows[yy][xx]] == color for xx, yy in neighbors):
                found.append({"x": x, "y": y, "symbol": symbol, "rgba": color})
    return found


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("matrices", nargs="+", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--include-exterior-black", action="store_true",
                        help="Also flag black silhouette pixels beside transparency")
    args = parser.parse_args()
    report = {}
    for path in args.matrices:
        palette_path = path.parent / "palette.json"
        if not palette_path.exists():
            palette_path = Path(__file__).resolve().parents[1] / "art/pixel/palette.json"
        palette = json.loads(palette_path.read_text())
        flags = isolated(path.read_text().splitlines(), palette, not args.include_exterior_black)
        report[str(path)] = flags
        print(f"{path.parent.name}/{path.name}: {len(flags)} suspicious isolated pixels")
    if args.output:
        args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(f"Total: {sum(len(flags) for flags in report.values())}; eyes and intentional details require visual review")
