"""Render matrices and pack pixel assets using only the Python standard library."""

import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "art" / "pixel"
LEGACY = ART / "legacy"
FRAME_NAMES = {"bonearcher": "bone_archer"}


def chunk(kind, data):
    checksum = zlib.crc32(kind + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)


def read_matrix(matrix):
    rows = matrix.read_text().splitlines()
    if not rows or not rows[0] or any(len(row) != len(rows[0]) for row in rows):
        raise ValueError(f"{matrix}: matrix must be a nonempty rectangle")
    if matrix.parent.parent == ART / "units" and (len(rows[0]), len(rows)) != (48, 48):
        raise ValueError(f"{matrix}: unit matrices must be 48×48")
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


def read_png(path):
    """Decode the original non-interlaced 8-bit RGB/RGBA sheets without resampling."""
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError(f"{path}: not a PNG")
    offset, compressed, transparent = 8, bytearray(), None
    while offset < len(data):
        length = struct.unpack(">I", data[offset : offset + 4])[0]
        kind = data[offset + 4 : offset + 8]
        payload = data[offset + 8 : offset + 8 + length]
        if kind == b"IHDR":
            width, height, depth, color, compression, filtering, interlace = struct.unpack(
                ">IIBBBBB", payload
            )
        elif kind == b"IDAT":
            compressed.extend(payload)
        elif kind == b"tRNS":
            transparent = struct.unpack(">HHH", payload)
        offset += length + 12
    if depth != 8 or color not in (2, 6) or compression or filtering or interlace:
        raise ValueError(f"{path}: expected non-interlaced 8-bit RGB/RGBA")
    channels = 3 if color == 2 else 4
    stride = width * channels
    raw = zlib.decompress(compressed)
    if len(raw) != (stride + 1) * height:
        raise ValueError(f"{path}: invalid scanline length")
    previous = bytearray(stride)
    rgba = bytearray()
    for y in range(height):
        start = y * (stride + 1)
        method = raw[start]
        row = bytearray(raw[start + 1 : start + 1 + stride])
        if method not in range(5):
            raise ValueError(f"{path}: invalid PNG filter")
        if method:
            for i in range(stride):
                left = row[i - channels] if i >= channels else 0
                up = previous[i]
                upper_left = previous[i - channels] if i >= channels else 0
                if method == 1:
                    predictor = left
                elif method == 2:
                    predictor = up
                elif method == 3:
                    predictor = (left + up) // 2
                else:
                    p = left + up - upper_left
                    a, b, c = abs(p - left), abs(p - up), abs(p - upper_left)
                    predictor = left if a <= b and a <= c else up if b <= c else upper_left
                row[i] = (row[i] + predictor) & 255
        if channels == 4:
            rgba.extend(row)
        else:
            for x in range(width):
                rgb = row[x * 3 : x * 3 + 3]
                rgba.extend(rgb)
                rgba.append(0 if transparent == tuple(rgb) else 255)
        previous = row
    return width, height, rgba


def crop_pixels(image, crop):
    """Copy whole source pixels; retain fractional crop edges in frame metadata."""
    width, height, pixels = image
    x, y, crop_width, crop_height = crop
    left, top = math.floor(x), math.floor(y)
    right, bottom = math.ceil(x + crop_width), math.ceil(y + crop_height)
    tile_width, tile_height = right - left, bottom - top
    if tile_width <= 0 or tile_height <= 0:
        raise ValueError(f"Invalid crop: {crop}")
    tile = bytearray(tile_width * tile_height * 4)
    # Out-of-sheet portions stay transparent, matching browser image cropping.
    for source_y in range(max(0, top), min(height, bottom)):
        start, end = max(0, left), min(width, right)
        if start < end:
            source = (source_y * width + start) * 4
            target = ((source_y - top) * tile_width + start - left) * 4
            tile[target : target + (end - start) * 4] = pixels[source : source + (end - start) * 4]
    return tile_width, tile_height, tile, [x - left, y - top, crop_width, crop_height]


def pack_atlas(tiles):
    placements, frames = [], {}
    x, y, row_height, width = 1, 1, 0, 0
    for key, (tile_width, tile_height, pixels, crop) in sorted(tiles.items()):
        if x + tile_width + 1 > 2048:
            x, y, row_height = 1, y + row_height + 1, 0
        placements.append((x, y, tile_width, tile_height, pixels))
        frames[key] = [x + crop[0], y + crop[1], crop[2], crop[3]]
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
    legacy = json.loads((LEGACY / "frames.json").read_text())
    native = {}
    for directory in sorted((ART / "units").iterdir()):
        if not directory.is_dir():
            continue
        alpha = None
        for side in ("player", "enemy"):
            matrix = directory / f"{side}.txt"
            if not matrix.exists():
                continue
            width, height, pixels = rgba_pixels(matrix)
            if (width, height) != (48, 48) or not any(pixels[3::4]):
                raise ValueError(f"{matrix}: expected a nonempty 48×48 sprite")
            if alpha is not None and alpha != pixels[3::4]:
                raise ValueError(f"{directory.name}: factions must share identical silhouettes")
            alpha = pixels[3::4]
            key = f"{side}_{FRAME_NAMES.get(directory.name, directory.name)}"
            if key in native:
                raise ValueError(f"Duplicate native frame: {key}")
            native[key] = (width, height, pixels, [0, 0, width, height])
    if not native:
        raise ValueError("No native unit matrices found")
    images, outputs, sources, frames = {}, {}, {}, {}
    for kind in ("units", "terrain"):
        tiles = dict(native) if kind == "units" else {}
        for key, entry in legacy[kind].items():
            if key in tiles:
                continue  # Existing matrices replace only their matching old artwork.
            path = LEGACY / entry["source"]
            if path not in images:
                images[path] = read_png(path)
            tiles[key] = crop_pixels(images[path], entry["crop"])
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

    def number(value):
        return int(value) if int(value) == value else value

    source_fields = ""
    for kind, source in sources.items():
        fields = "".join(f"    {key}: {json.dumps(value)},\n" for key, value in source.items())
        source_fields += f"  {kind}: {{\n{fields}  }},\n"
    frame_fields = "".join(
        f"  {property_name(key)}: {json.dumps([number(v) for v in crop])},\n"
        for key, crop in sorted(frames.items())
    )
    metadata = (
        "// Generated by scripts/render-pixel.py --atlas; edit matrices or legacy crop inputs.\n"
        + "export const pixelSources = {\n" + source_fields + "} as const;\n\n"
        + "export const pixelFrames = {\n" + frame_fields + "} as const;\n"
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
