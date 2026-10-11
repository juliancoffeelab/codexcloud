"""Pack authoritative pixel/art sources; never replay recipes or modify source art."""

import argparse
import hashlib
import json
import math
import re
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


def require_road_exits(rows, mask, surface=frozenset("RrC"), width=8, depth=4, label="Road"):
    """Check shared-edge position, width and tangent, independent of interior art."""
    require_canvas(rows, label)
    apothem = 16 * math.sqrt(3)
    for direction in range(6):
        angle = direction * math.pi / 3
        ux, uy = math.cos(angle), math.sin(angle)
        for y, row in enumerate(rows):
            for x, symbol in enumerate(row):
                nx, ny = x + .5 - 32, y + .5 - 32
                normal = nx * ux + ny * uy
                tangent = abs(-nx * uy + ny * ux)
                if apothem - depth <= normal <= apothem and tangent < width / 2 + 3:
                    expected = bool(mask & (1 << direction)) and tangent < width / 2
                    if (symbol in surface) != expected:
                        raise ValueError(f"{label} {mask}: misaligned exit {direction} at ({x}, {y}); "
                                         "road edges must share centered width and normal direction")
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


def require_shared_palette(palette, symbols, label):
    if not isinstance(palette, dict) or set(palette) != set(symbols) or any(
        not isinstance(rgba, list) or len(rgba) != 4 or rgba[3] != 255
        or any(type(channel) is not int or not 0 <= channel <= 255 for channel in rgba)
        for rgba in palette.values()
    ):
        raise ValueError(f'{label}: exactly two opaque shades {symbols} are required')
    return palette


def faction_palette(side):
    return require_shared_palette(json.loads((ART / "factions.json").read_text())[side],
                                  ('^', '*'), f'{side} faction')


def load_terrain_floors():
    floors = json.loads((ART / "terrain/floors.json").read_text())
    for name, palette in floors['palettes'].items():
        if not re.fullmatch(r'[a-z][a-z0-9-]*', name):
            raise ValueError(f'Invalid floor name: {name!r}')
        require_shared_palette(palette, ('G', 'g'), f'{name} floor')
    for kind, name in floors['defaults'].items():
        if name not in floors['palettes']:
            raise ValueError(f'{kind}: unknown default floor {name!r}')
    return floors


def terrain_floor_palette(kind, floor=None):
    floors = load_terrain_floors()
    name = floor if floor is not None else floors['defaults'].get(kind)
    if name not in floors['palettes']:
        raise ValueError(f'{kind}: unknown floor {name!r}')
    return floors['palettes'][name]


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


def terrain_material_kind(matrix):
    matrix = matrix.resolve()
    root = (ART / 'terrain').resolve()
    if matrix.parent.parent == root:
        return matrix.parent.name
    if matrix.parent.parent.parent == root / 'props':
        return matrix.parent.parent.name
    return None


def read_palette(matrix, floor=None):
    matrix = matrix.resolve()
    path = matrix.parent / "palette.json"
    palette = json.loads(path.read_text())
    flying = palette.pop("flying", False)
    if type(flying) is not bool:
        raise ValueError(f"{path}: 'flying' must be a boolean")
    kind = terrain_material_kind(matrix)
    if kind is not None:
        ground = terrain_floor_palette(kind, floor)
        if palette.keys() & ground.keys():
            raise ValueError(f"{path}: floor shades belong in terrain/floors.json")
        palette.update(ground)
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


def load_terrain_layouts():
    layouts = {}
    for path in sorted((ART / 'terrain').glob('*/layouts.json')):
        kind = path.parent.name
        frames = json.loads(path.read_text())
        if isinstance(frames, list):
            frames = {'tile': frames}
        if not isinstance(frames, dict) or not frames:
            raise ValueError(f'{path}: expected nonempty terrain layouts')
        for frame, variants in frames.items():
            if (not isinstance(frame, str) or not re.fullmatch(r'[a-z0-9][a-z0-9-]*', frame)
                    or not (path.parent / f'{frame}.txt').is_file()):
                raise ValueError(f'{path}: unknown terrain frame {frame!r}')
            ground = read_matrix(path.parent / f'{frame}.txt')
            if not isinstance(variants, list) or not variants:
                raise ValueError(f'{path}: expected nonempty terrain layouts')
            key = kind if frame == 'tile' else f'{kind}_{frame}'
            layouts[key] = []
            for variant in variants:
                if not isinstance(variant, list) or not variant:
                    raise ValueError(f'{path}: expected nonempty prop placements')
                placements = []
                for p in variant:
                    if (not isinstance(p, dict) or set(p) != {'prop', 'x', 'y'}
                            or not isinstance(p['prop'], str) or not re.fullmatch(r'[a-z][a-z0-9-]*', p['prop'])
                            or type(p['x']) is not int or type(p['y']) is not int):
                        raise ValueError(f'{path}: invalid prop placement')
                    matrix = ART / 'terrain/props' / kind / p['prop'] / 'tile.txt'
                    if not matrix.is_file():
                        raise ValueError(f'{path}: unknown prop {p["prop"]!r}')
                    for y, row in enumerate(read_matrix(matrix)):
                        for x, s in enumerate(row):
                            xx, yy = x + p['x'], y + p['y']
                            if s != '.' and not (0 <= xx < 64 and 0 <= yy < 64 and ground[yy][xx] != '.'):
                                raise ValueError(f'{path}: prop {p["prop"]!r} extends outside the hex')
                    placements.append({'name': f'prop_{kind}_{p["prop"]}', 'x': p['x'], 'y': p['y']})
                layouts[key].append(placements)
    return layouts


def require_grounded(rows, palette, flying=False, label="Character"):
    require_canvas(rows, label)
    if type(flying) is not bool:
        raise ValueError(f"{label}: 'flying' must be a boolean")
    if not flying and not any(palette[symbol][3] for symbol in rows[-1]):
        raise ValueError(f"{label}: grounded sprites must touch canvas bottom (row 63); "
                         "declare flying: true in palette.json only for suspended poses")


def rgba_pixels(matrix, side="player", floor=None):
    rows, palette = read_matrix(matrix), read_palette(matrix, floor)
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
    terrain = {}
    floors = load_terrain_floors()

    def add_terrain(key, matrix):
        terrain[key] = rgba_pixels(matrix)
        if not any(terrain[key][2][3::4]):
            raise ValueError(f'{matrix}: expected nonempty terrain artwork')
        for floor in floors['palettes']:
            if floor != floors['defaults'][terrain_material_kind(matrix)]:
                terrain[f'{key}__{floor}'] = rgba_pixels(matrix, floor=floor)

    for matrix in sorted((ART / "terrain").glob("*/tile.txt")):
        key = f"terrain_{FRAME_NAMES.get(matrix.parent.name, matrix.parent.name)}"
        add_terrain(key, matrix)
        kind = matrix.parent.name
        default_mask = {"road": 9, "village": 9}.get(kind)
        surface = frozenset("RrC" if kind == "road" else "RrH")
        if default_mask is not None:
            require_road_exits(read_matrix(matrix), default_mask, surface, label=kind)
        for variant in sorted(matrix.parent.glob("*.txt")):
            if variant == matrix:
                continue
            crossing = matrix.parent.name in ("bridge", "ford") and variant.stem in ("ne-sw", "nw-se")
            path = (default_mask is not None and variant.stem.isdigit() and
                    1 <= int(variant.stem) <= 63 and int(variant.stem) != default_mask and
                    (kind == "road" or (kind == "village" and int(variant.stem).bit_count() <= 3)))
            capped = re.fullmatch(r"(\d+)-end-(\d+)", variant.stem) if kind == "road" else None
            if capped:
                exits, ends = map(int, capped.groups())
                capped = 1 <= exits <= 63 and 1 <= ends <= 63 and not (exits & ends)
            if not (crossing or path or capped):
                raise ValueError(f"{variant}: unknown terrain orientation")
            if path:
                require_road_exits(read_matrix(variant), int(variant.stem), surface, label=kind)
            if capped:
                require_road_exits(read_matrix(variant), exits, surface, label=kind)
            add_terrain(f"{key}_{variant.stem}", variant)
    layouts = load_terrain_layouts()
    for matrix in sorted((ART / 'terrain/props').glob('*/*/tile.txt')):
        add_terrain(f'terrain_prop_{matrix.parent.parent.name}_{matrix.parent.name}', matrix)
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
    floor_fields = ''.join(
        f'  {property_name(name)}: {json.dumps("#" + "".join(f"{c:02x}" for c in palette["G"][:3]))},\n'
        for name, palette in sorted(floors['palettes'].items())
    )
    default_fields = ''.join(
        f'  {property_name(kind)}: {json.dumps(name)},\n'
        for kind, name in sorted(floors['defaults'].items())
    )
    layout_fields = ''
    for kind, variants in layouts.items():
        layout_fields += f'  {property_name(kind)}: [\n'
        for placements in variants:
            layout_fields += '    [\n'
            for p in placements:
                layout_fields += f'      {{ name: {json.dumps(p["name"])}, x: {p["x"]}, y: {p["y"]} }},\n'
            layout_fields += '    ],\n'
        layout_fields += '  ],\n'
    model_fields = "".join(
        f"  {property_name(key)}: {json.dumps(model)},\n"
        for key, model in sorted(gameplay_models(lineups).items())
    )
    metadata = (
        "// Generated by scripts/build-pixel-atlas.py; edit pixel/art matrices and palettes.\n"
        + "export const pixelSources = {\n" + source_fields + "} as const;\n\n"
        + "export const pixelFrames = {\n" + frame_fields + "} as const;\n"
        + "\nexport const pixelFloorColors: Readonly<Record<string, string>> = {\n" + floor_fields + "};\n"
        + "\nexport const pixelDefaultFloors = {\n" + default_fields + "} as const;\n"
        + "\nexport const pixelTerrainLayouts: Readonly<\n"
        + "  Record<string, readonly (readonly { name: string; x: number; y: number }[])[]>\n"
        + "> = {\n" + layout_fields + "};\n"
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
