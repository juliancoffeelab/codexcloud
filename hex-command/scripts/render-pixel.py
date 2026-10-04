"""Render native pixel matrices using only the Python standard library."""

import argparse
import json
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "art" / "pixel"


def chunk(kind, data):
    checksum = zlib.crc32(kind + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)


def render(matrix, palette):
    rows = matrix.read_text().splitlines()
    if not rows or not rows[0] or any(len(row) != len(rows[0]) for row in rows):
        raise ValueError(f"{matrix}: matrix must be a nonempty rectangle")
    width, height = len(rows[0]), len(rows)
    pixels = bytearray()
    for y, row in enumerate(rows):
        pixels.append(0)  # PNG filter: none.
        for x, symbol in enumerate(row):
            if symbol not in palette:
                raise ValueError(f"{matrix}:{y + 1}:{x + 1}: unknown symbol {symbol!r}")
            pixels.extend(palette[symbol])
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(pixels, level=9))
        + chunk(b"IEND", b"")
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("matrices", nargs="*", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    palette = json.loads((ART / "palette.json").read_text())
    for symbol, rgba in palette.items():
        if (
            len(symbol) != 1
            or not isinstance(rgba, list)
            or len(rgba) != 4
            or any(type(channel) is not int or not 0 <= channel <= 255 for channel in rgba)
            or rgba[3] not in (0, 255)
        ):
            raise ValueError(f"Invalid flat RGBA palette entry: {symbol!r}")
    if palette.get(".") != [0, 0, 0, 0]:
        raise ValueError("Palette '.' must be transparent black")
    matrices = [matrix.resolve() for matrix in args.matrices] or sorted(ART.rglob("*.txt"))
    if not matrices:
        raise ValueError("No pixel matrices found")
    for matrix in matrices:
        png = matrix.with_suffix(".png")
        data = render(matrix, palette)
        if args.check:
            if not png.exists() or png.read_bytes() != data:
                raise ValueError(f"{png}: regenerate from {matrix}")
            print(f"Verified {png.relative_to(ROOT)}")
        else:
            png.write_bytes(data)
            print(f"Rendered {png.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
