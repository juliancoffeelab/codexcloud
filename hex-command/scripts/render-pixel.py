"""Render matrices and pack pixel assets using only the Python standard library."""

import argparse
import hashlib
import json
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "art" / "pixel"
FRAME_NAMES = {"bonearcher": "bone_archer", "grove": "sacred_grove"}


def chunk(kind, data):
    checksum = zlib.crc32(kind + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)


def read_matrix(matrix):
    rows = matrix.read_text().splitlines()
    if not rows or not rows[0] or any(len(row) != len(rows[0]) for row in rows):
        raise ValueError(f"{matrix}: matrix must be a nonempty rectangle")
    if matrix.parent.parent in (ART / "units", ART / "terrain", ART / "portraits") and (len(rows[0]), len(rows)) != (48, 48):
        raise ValueError(f"{matrix}: artwork matrices must be 48×48")
    return rows


def read_palette(matrix):
    path = matrix.parent / "palette.json"
    palette = json.loads((path if path.exists() else ART / "palette.json").read_text())
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
    return palette


def rgba_pixels(matrix):
    rows, palette = read_matrix(matrix), read_palette(matrix)
    pixels = bytearray()
    for y, row in enumerate(rows):
        for x, symbol in enumerate(row):
            if symbol not in palette:
                raise ValueError(f"{matrix}:{y + 1}:{x + 1}: unknown symbol {symbol!r}")
            pixels.extend(palette[symbol])
    return len(rows[0]), len(rows), pixels


def png_bytes(width, height, rgba):
    pixels = bytearray()
    stride = width * 4
    for y in range(height):
        pixels.append(0)  # PNG filter: none.
        pixels.extend(rgba[y * stride : (y + 1) * stride])
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(pixels, level=9))
        + chunk(b"IEND", b"")
    )


def pack_atlas(tiles, max_width=393):
    placements, frames = [], {}
    x, y, row_height, width = 1, 1, 0, 0
    for key, (tile_width, tile_height, pixels) in sorted(tiles.items()):
        if x + tile_width + 1 > max_width and x > 1:
            x, y, row_height = 1, y + row_height + 1, 0
        placements.append((x, y, tile_width, tile_height, pixels))
        frames[key] = [x, y, tile_width, tile_height]
        width = max(width, x + tile_width + 1)
        row_height = max(row_height, tile_height)
        x += tile_width + 1
    height = y + row_height + 1
    atlas = bytearray(width * height * 4)
    for x, y, tile_width, tile_height, pixels in placements:
        for row in range(tile_height):
            target = ((y + row) * width + x) * 4
            atlas[target : target + tile_width * 4] = pixels[row * tile_width * 4 : (row + 1) * tile_width * 4]
    return png_bytes(width, height, atlas), frames


def pixel_atlases():
    native = {}
    for directory in sorted((ART / "units").iterdir()):
        if not directory.is_dir():
            continue
        alpha = None
        for side in ("player", "enemy"):
            matrix = directory / f"{side}.txt"
            if not matrix.exists():
                raise ValueError(f"{matrix}: both faction matrices are required")
            width, height, pixels = rgba_pixels(matrix)
            if (width, height) != (48, 48) or not any(pixels[3::4]):
                raise ValueError(f"{matrix}: expected a nonempty 48×48 sprite")
            if alpha is not None and alpha != pixels[3::4]:
                raise ValueError(f"{directory.name}: factions must share identical silhouettes")
            alpha = pixels[3::4]
            key = f"{side}_{FRAME_NAMES.get(directory.name, directory.name)}"
            if key in native:
                raise ValueError(f"Duplicate native frame: {key}")
            native[key] = (width, height, pixels)
    if not native:
        raise ValueError("No native unit matrices found")
    for matrix in sorted((ART / "portraits").glob("*/portrait.txt")):
        width, height, pixels = rgba_pixels(matrix)
        if not any(pixels[3::4]):
            raise ValueError(f"{matrix}: expected a nonempty portrait")
        native[f"portrait_{matrix.parent.name}"] = (width, height, pixels)
    terrain, terrain_colors = {}, {}
    for matrix in sorted((ART / "terrain").glob("*/tile.txt")):
        width, height, pixels = rgba_pixels(matrix)
        if not any(pixels[3::4]):
            raise ValueError(f"{matrix}: expected a nonempty terrain sprite")
        key = f"terrain_{FRAME_NAMES.get(matrix.parent.name, matrix.parent.name)}"
        terrain[key] = (width, height, pixels)
        # The vector hex fills the transparent staircase around the raster tile.
        # Ground palettes name their base G; river palettes name it W.
        palette = read_palette(matrix)
        background = palette["G" if "G" in palette else "W"]
        if background[3] != 255:
            raise ValueError(f"{matrix}: terrain ground must be opaque")
        terrain_colors[matrix.parent.name] = "#" + "".join(f"{channel:02x}" for channel in background[:3])
    if not terrain:
        raise ValueError("No native terrain matrices found")
    outputs, sources, frames = {}, {}, {}
    for kind, tiles in (("units", native), ("terrain", terrain)):
        data, packed = pack_atlas(tiles)
        outputs[f"{kind}-pixel.png"] = data
        width, height = struct.unpack(">II", data[16:24])
        sources[kind] = {
            "filename": f"{kind}-pixel.png",
            "width": width,
            "height": height,
            "revision": hashlib.sha256(data).hexdigest()[:12],
        }
        frames.update(packed)

    def property_name(key):
        return key if key.isidentifier() else json.dumps(key)

    source_fields = ""
    for kind, source in sources.items():
        fields = "".join(f"    {key}: {json.dumps(value)},\n" for key, value in source.items())
        source_fields += f"  {kind}: {{\n{fields}  }},\n"
    frame_fields = "".join(
        f"  {property_name(key)}: {json.dumps(crop)},\n"
        for key, crop in sorted(frames.items())
    )
    color_fields = "".join(
        f"  {property_name(key)}: {json.dumps(color)},\n"
        for key, color in sorted(terrain_colors.items())
    )
    metadata = (
        "// Generated by scripts/render-pixel.py --atlas; edit art/pixel matrices and palettes.\n"
        + "export const pixelSources = {\n" + source_fields + "} as const;\n\n"
        + "export const pixelFrames = {\n" + frame_fields + "} as const;\n"
        + "\nexport const pixelTerrainColors = {\n" + color_fields + "} as const;\n"
    )
    return outputs, metadata


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("matrices", nargs="*", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--atlas", action="store_true", help="Build the public unit and terrain atlases")
    args = parser.parse_args()
    matrices = [matrix.resolve() for matrix in args.matrices] or sorted(ART.rglob("*.txt"))
    for matrix in matrices:
        png = matrix.with_suffix(".png")
        data = png_bytes(*rgba_pixels(matrix))
        if args.check:
            if not png.exists() or png.read_bytes() != data:
                raise ValueError(f"{png}: regenerate from {matrix}")
            print(f"Verified {png.relative_to(ROOT)}")
        else:
            png.write_bytes(data)
            print(f"Rendered {png.relative_to(ROOT)}")
    if args.atlas:
        outputs, metadata = pixel_atlases()
        for filename, data in outputs.items():
            path = ROOT / "public" / "assets" / filename
            if args.check:
                if not path.exists() or path.read_bytes() != data:
                    raise ValueError(f"{path}: regenerate with --atlas")
            else:
                path.write_bytes(data)
        manifest = ROOT / "src" / "presentation" / "pixel-art.ts"
        if args.check:
            if not manifest.exists() or manifest.read_text() != metadata:
                raise ValueError(f"{manifest}: regenerate with --atlas")
            print("Verified public pixel unit/terrain atlases and metadata")
        else:
            manifest.write_text(metadata)
            print("Packed public pixel unit/terrain atlases and metadata")


if __name__ == "__main__":
    main()
