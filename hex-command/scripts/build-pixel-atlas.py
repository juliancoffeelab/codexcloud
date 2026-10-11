"""Pack authoritative pixel/art sources; never replay recipes or modify source art."""

import argparse
import hashlib
import json
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "pixel" / "art"
FRAME_NAMES = {"bonearcher": "bone_archer", "grove": "sacred_grove"}


def require_canvas(rows, label="Artwork"):
    if len(rows) != 64 or any(len(row) != 64 for row in rows):
        raise ValueError(f"{label}: artwork matrices must be exactly 64×64")
    return rows


def unit_directories(lineups=None):
    return {unit["id"]: ART / "units" / race / unit["source"]
            for race, lineup in (lineups or load_lineups()).items()
            for unit in entries(lineup)}


def unit_directory(id):
    try:
        return unit_directories()[id]
    except KeyError:
        raise ValueError(f"Unknown unit: {id}; register it in its race tree") from None


def gameplay_models(lineups):
    models = {}
    for lineup in lineups.values():
        for unit in entries(lineup):
            if "gameplayId" not in unit:
                continue
            id = unit["gameplayId"]
            if id in models:
                raise ValueError(f"Duplicate gameplay artwork assignment: {id}")
            definition = json.loads((ROOT / "rust/data/game/units" / f"{id}.json").read_text())
            if definition["level"] != unit["level"]:
                raise ValueError(f"{id}: artwork tier must match its gameplay tier")
            models[id] = unit["id"]
    return models


def entries(lineup):
    for tree in lineup["trees"]:
        yield tree["base"]
        for branch in tree["branches"]:
            yield from branch["units"]
    if "commander" in lineup:
        yield from lineup["commander"]["units"]


def rows(lineup):
    if "commander" in lineup:
        yield lineup["commander"]["units"]
    for tree in lineup["trees"]:
        for branch in tree["branches"]:
            yield [tree["base"], *branch["units"]]


def load_lineups(directory=None):
    directory = directory or ART / "units/lineups"
    lineups, owners, folders = {}, {}, set()
    if any(directory.parent.rglob("lineup.json")) or any(p.is_dir() for p in directory.iterdir()):
        raise ValueError("Race trees must be flat lineups/<race>.json files")
    paths = sorted(directory.glob("*.json"))
    if not paths:
        raise ValueError("No race tree JSON files found")
    for path in paths:
        lineup = json.loads(path.read_text())
        if "references" in lineup:
            raise ValueError(f"{path}: race trees contain selected units only, not archived poses")
        race = lineup.get("race")
        if race != path.stem or not isinstance(lineup.get("trees"), list) or not lineup["trees"]:
            raise ValueError(f"{path}: expected the race's named tree file")
        for tree in lineup["trees"]:
            if tree.get("base", {}).get("level") != 1 or not tree.get("branches"):
                raise ValueError(f"{path}: every starter needs L1 and at least one branch")
            for branch in tree["branches"]:
                if not branch.get("role") or [u.get("level") for u in branch.get("units", [])] != [2, 3]:
                    raise ValueError(f"{path}: branches must contain L2 followed by L3")
        if "commander" in lineup and [u.get("level") for u in lineup["commander"].get("units", [])] != [1, 2, 3]:
            raise ValueError(f"{path}: commander must contain L1, L2 and L3")
        for unit in entries(lineup):
            id = unit.get("id")
            if not isinstance(id, str) or not id or Path(id).name != id or not unit.get("name"):
                raise ValueError(f"{path}: invalid unit identity")
            if id in owners:
                raise ValueError(f"{path}: {id} is already assigned in {owners[id]}")
            source = unit.get("source")
            if not isinstance(source, str):
                raise ValueError(f"{path}: {id} needs its own unit folder")
            race_root = (directory.parent / race).resolve()
            folder = (race_root / source).resolve()
            if not folder.is_relative_to(race_root) or folder == race_root:
                raise ValueError(f"{path}: {id} source must belong inside its race folder")
            if source != id:
                raise ValueError(f"{path}: {id} source must be its own unit folder")
            if folder in folders:
                raise ValueError(f"{path}: duplicate variant folder for {id}")
            if not (folder / "sprite.txt").is_file():
                raise ValueError(f"{path}: missing matrix for {id}")
            require_canvas((folder / "sprite.txt").read_text().splitlines(), id)
            owners[id] = race
            folders.add(folder)
        lineups[race] = lineup
    unexpected = {p.parent.resolve() for p in directory.parent.rglob("sprite.txt")} - folders
    if unexpected:
        raise ValueError(f"Unregistered unit folders: {sorted(map(str, unexpected))}")
    return lineups


def faction_palette(side):
    palette = json.loads((ART / "factions.json").read_text())[side]
    if set(palette) != {'^', '*'} or any(
        not isinstance(rgba, list) or len(rgba) != 4 or rgba[3] != 255
        or any(type(channel) is not int or not 0 <= channel <= 255 for channel in rgba)
        for rgba in palette.values()
    ):
        raise ValueError(f'{side}: exactly two opaque faction shades are required')
    return palette


def chunk(kind, data):
    checksum = zlib.crc32(kind + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)


def read_matrix(matrix):
    matrix = matrix.resolve()
    rows = matrix.read_text().splitlines()
    if not rows or not rows[0] or any(len(row) != len(rows[0]) for row in rows):
        raise ValueError(f"{matrix}: matrix must be a nonempty rectangle")
    require_canvas(rows, matrix)
    return rows


def read_palette(matrix):
    path = matrix.parent / "palette.json"
    palette = json.loads(path.read_text())
    flying = palette.pop("flying", False)
    if type(flying) is not bool:
        raise ValueError(f"{path}: 'flying' must be a boolean")
    for symbol, rgba in palette.items():
        if (
            len(symbol) != 1
            or not isinstance(rgba, list)
            or len(rgba) != 4
            or any(type(channel) is not int or not 0 <= channel <= 255 for channel in rgba)
            or rgba[3] not in (0, 255)
        ):
            raise ValueError(f"Invalid flat RGBA palette entry: {symbol!r}")
    if matrix.stem == "sprite" and ("^" in palette or "*" in palette):
        raise ValueError(f"{matrix}: faction shades belong in factions.json")
    if palette.get(".") != [0, 0, 0, 0]:
        raise ValueError("Palette '.' must be transparent black")
    return palette


def require_grounded(rows, palette, flying=False, label="Character"):
    require_canvas(rows, label)
    if type(flying) is not bool:
        raise ValueError(f"{label}: 'flying' must be a boolean")
    if not flying and not any(palette[symbol][3] for symbol in rows[-1]):
        raise ValueError(f"{label}: grounded sprites must touch canvas bottom (row 63); "
                         "declare flying: true in palette.json only for suspended poses")


def rgba_pixels(matrix, side="player"):
    rows, palette = read_matrix(matrix), read_palette(matrix)
    if matrix.stem == "sprite":
        palette.update(faction_palette(side))
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
    lineups = load_lineups()
    native = {}
    for id, directory in sorted(unit_directories(lineups).items()):
        matrix = directory / "sprite.txt"
        materials = read_palette(matrix)
        require_grounded(read_matrix(matrix), {**materials, **faction_palette("player")},
                         json.loads((directory / "palette.json").read_text()).get("flying", False), id)
        alpha = None
        for side in ("player", "enemy"):
            matrix = directory / "sprite.txt"
            if not matrix.exists():
                raise ValueError(f"{matrix}: a shared sprite matrix is required")
            width, height, pixels = rgba_pixels(matrix, side)
            require_canvas(read_matrix(matrix), id)
            if not any(pixels[3::4]):
                raise ValueError(f"{matrix}: expected a nonempty unit sprite")
            if alpha is not None and alpha != pixels[3::4]:
                raise ValueError(f"{id}: factions must share identical silhouettes")
            alpha = pixels[3::4]
            key = f"{side}_{FRAME_NAMES.get(id, id)}"
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
    model_fields = "".join(
        f"  {property_name(key)}: {json.dumps(model)},\n"
        for key, model in sorted(gameplay_models(lineups).items())
    )
    metadata = (
        "// Generated by scripts/build-pixel-atlas.py; edit pixel/art matrices and palettes.\n"
        + "export const pixelSources = {\n" + source_fields + "} as const;\n\n"
        + "export const pixelFrames = {\n" + frame_fields + "} as const;\n"
        + "\nexport const pixelTerrainColors = {\n" + color_fields + "} as const;\n"
        + "\nexport const pixelUnitModels: Readonly<Record<string, string>> = {\n"
        + model_fields + "};\n"
    )
    return outputs, metadata


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Verify generated assets without writing files")
    args = parser.parse_args()
    outputs, metadata = pixel_atlases()
    products = {ROOT / "public/assets" / name: data for name, data in outputs.items()}
    products[ROOT / "src/presentation/pixel-art.ts"] = metadata.encode()
    for path, data in products.items():
        if args.check:
            if not path.exists() or path.read_bytes() != data:
                raise ValueError(f"{path}: regenerate with npm run art:build")
        elif not path.exists() or path.read_bytes() != data:
            path.write_bytes(data)
    print(f"{'Verified' if args.check else 'Packed'} pixel unit/terrain atlases and metadata; sources are read-only")


if __name__ == "__main__":
    main()
