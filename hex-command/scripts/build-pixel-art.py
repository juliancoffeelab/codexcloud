"""Build standalone native sprites from pixel/art; never modify source artwork."""

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
# Finite native canvas policy, not an artwork-controlled size override. New
# families stay 64x64 unless this reviewed contract explicitly admits them.
PADDED_TERRAIN_FAMILIES = frozenset({'castle-wall', 'castle-gate', 'castle-bell'})
CASTLE_CANVAS = {'width': 80, 'height': 96, 'anchorX': 40, 'anchorY': 48}


def terrain_canvas(path):
    geometry = json.loads(path.read_text())
    if (path.parent.name not in PADDED_TERRAIN_FAMILIES or
            not isinstance(geometry, dict) or
            set(geometry) != {*CASTLE_CANVAS, 'z'} or
            any(type(v) is not int for v in geometry.values()) or
            any(geometry[key] != value for key, value in CASTLE_CANVAS.items())):
        raise ValueError(f'{path}: only approved castle terrain may use the '
                         '80×96 canvas anchored at (40,48); all other artwork is 64×64')
    return geometry


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


def load_road_dead_ends():
    pairs = json.loads((ART / 'terrain/road/dead-ends.json').read_text())
    if not isinstance(pairs, list) or any(
        not isinstance(pair, list) or len(pair) != 2 or
        any(type(mask) is not int or not 1 <= mask <= 63 for mask in pair) or
        pair[0] & pair[1] for pair in pairs
    ) or len({tuple(pair) for pair in pairs}) != len(pairs):
        raise ValueError('Invalid registered road caps')
    return {tuple(pair) for pair in pairs}


def require_terrain_variants(kind, stems, road_caps):
    """Every supported connection must have finished artwork, without frozen pixels."""
    if kind == 'road':
        expected = {str(mask) for mask in range(1, 64)}
        expected |= {f'{exits}-end-{ends}' for exits, ends in road_caps}
    elif kind == 'signpost':
        expected = {'9'}
    elif kind == 'village':
        expected = {'0'} | {str(mask) for mask in range(1, 64)
                            if mask.bit_count() <= 3}
    elif kind in ('castle-wall', 'castle-gate', 'castle-bell'):
        catalog = json.loads((ART / 'terrain/castle-library.json').read_text())
        if catalog.get('version') != 1:
            raise ValueError('Unsupported castle library definition version')
        definitions = catalog['families'][kind]
        registered = json.loads((ART / 'terrain' / kind / 'variants.json').read_text())
        if sorted(definitions, key=lambda v: v['id']) != sorted(registered, key=lambda v: v['id']):
            raise ValueError(f'{kind}: finished registry differs from the reusable tile definitions')
        expected = {v['id'] for v in definitions}
    elif kind in ('bridge', 'ford'):
        expected = {'9', '18', '36'}
    elif kind in ('landing', 'ferry'):
        expected = {'1', '8'}
    else:
        return
    missing, unsupported = expected - set(stems), set(stems) - expected
    if missing or unsupported:
        raise ValueError(f'{kind}: missing supported variants {sorted(missing)}; '
                         f'unsupported variants {sorted(unsupported)}')


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
        if not tree["branches"]:
            yield [tree["base"]]
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
            if tree.get("base", {}).get("level") != 1 or not isinstance(tree.get("branches"), list):
                raise ValueError(f"{path}: every starter needs L1 and a branches list (which may be empty)")
            for branch in tree["branches"]:
                if not branch.get("role") or [u.get("level") for u in branch.get("units", [])] != [2, 3]:
                    raise ValueError(f"{path}: branches must contain L2 followed by L3")
        if "commander" in lineup and [u.get("level") for u in lineup["commander"].get("units", [])] != [1, 2, 3]:
            raise ValueError(f"{path}: commander must contain L1, L2 and L3")
        for unit in entries(lineup):
            factions = unit.get("factions", ["player", "enemy"])
            available = set(json.loads((ART / "factions.json").read_text())) - {"debug"}
            if not isinstance(factions, list) or any(not isinstance(faction, str) for faction in factions) or not {"player", "enemy"}.issubset(factions) or len(set(factions)) != len(factions) or not set(factions).issubset(available):
                raise ValueError(f'{path}: invalid unit faction variants')
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
    return floors


def terrain_floor_palette(kind, floor=None):
    floors = load_terrain_floors()
    if floor is None:
        return {'G': [0, 0, 0, 0], 'g': [0, 0, 0, 0]}
    if floor not in floors['palettes']:
        raise ValueError(f'{kind}: unknown floor {floor!r}')
    return floors['palettes'][floor]


def chunk(kind, data):
    checksum = zlib.crc32(kind + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)


def read_matrix(matrix):
    matrix = matrix.resolve()
    rows = matrix.read_text().splitlines()
    if not rows or not rows[0] or any(len(row) != len(rows[0]) for row in rows):
        raise ValueError(f"{matrix}: matrix must be a nonempty rectangle")
    canvas = matrix.parent / 'canvas.json'
    if canvas.is_file() and matrix.parent.parent == ART / 'terrain':
        geometry = terrain_canvas(canvas)
        if (len(rows) != geometry['height'] or len(rows[0]) != geometry['width']):
            raise ValueError(f'{matrix}: invalid padded terrain canvas')
    else:
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
        ):
            raise ValueError(f"Invalid RGBA palette entry: {symbol!r}")
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


def composite(base, layers):
    """Composite straight-alpha layers at native integer offsets, in depth order."""
    width, height, pixels = base
    result = bytearray(pixels)
    for (layer_width, layer_height, layer), dx, dy in layers:
        if type(dx) is not int or type(dy) is not int:
            raise ValueError('Native placements must be integer offsets')
        for y in range(layer_height):
            for x in range(layer_width):
                if not (0 <= x + dx < width and 0 <= y + dy < height):
                    continue
                source = (y * layer_width + x) * 4
                alpha = layer[source + 3]
                if alpha:
                    target = ((y + dy) * width + x + dx) * 4
                    if alpha == 255:
                        result[target:target + 4] = layer[source:source + 4]
                    else:
                        background_alpha = result[target + 3]
                        combined = alpha * 255 + background_alpha * (255 - alpha)
                        for channel in range(3):
                            numerator = (layer[source + channel] * alpha * 255
                                         + result[target + channel] * background_alpha * (255 - alpha))
                            result[target + channel] = (numerator + combined // 2) // combined
                        result[target + 3] = (combined + 127) // 255
    return width, height, result


def pixel_sprites():
    lineups = load_lineups()
    native = {}
    factions = {u["id"]: u.get("factions", ["player", "enemy"]) for lineup in lineups.values() for u in entries(lineup)}
    for id, directory in sorted(unit_directories(lineups).items()):
        matrix = directory / "sprite.txt"
        materials = read_palette(matrix)
        require_grounded(read_matrix(matrix), {**materials, **faction_palette("player")},
                         json.loads((directory / "palette.json").read_text()).get("flying", False), id)
        alpha = None
        for side in factions[id]:
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
    road_caps = load_road_dead_ends()

    def add_terrain(key, matrix):
        terrain[key] = rgba_pixels(matrix)
        for floor in floors['palettes']:
            terrain[f'{key}__{floor}'] = rgba_pixels(matrix, floor=floor)
            if not any(terrain[f'{key}__{floor}'][2][3::4]):
                raise ValueError(f'{matrix}: expected nonempty terrain artwork')

    for palette in sorted((ART / "terrain").glob("*/palette.json")):
        kind = palette.parent.name
        matrices = sorted(palette.parent.glob("*.txt"))
        require_terrain_variants(kind, [path.stem for path in matrices], road_caps)
        connected = kind in ('signpost', 'road', 'village', 'bridge', 'ford', 'landing', 'ferry', 'castle-wall', 'castle-gate', 'castle-bell')
        if not connected and [p.stem for p in matrices] != ['tile']:
            raise ValueError(f'{kind}: unconnected terrain requires only tile.txt')
        key = f"terrain_{FRAME_NAMES.get(kind, kind)}"
        for matrix in matrices:
            if kind in ('road', 'village'):
                mask = int(matrix.stem.split('-')[0])
                require_road_exits(read_matrix(matrix), mask,
                                   frozenset("RrC" if kind == "road" else "RrH"), label=kind)
            add_terrain(f"{key}_{matrix.stem}" if connected else key, matrix)
    layouts = load_terrain_layouts()
    for matrix in sorted((ART / 'terrain/props').glob('*/*/tile.txt')):
        add_terrain(f'terrain_prop_{matrix.parent.parent.name}_{matrix.parent.name}', matrix)
    if not terrain:
        raise ValueError("No native terrain matrices found")
    for name, variants in layouts.items():
        for index, placements in enumerate(variants):
            for floor in [None, *floors['palettes']]:
                suffix = f'__{floor}' if floor else ''
                base = terrain[f'terrain_{name}{suffix}']
                layers = [(terrain[f'terrain_{piece["name"]}{suffix}'], piece['x'], piece['y'])
                          for piece in placements]
                terrain[f'terrain_{name}_layout_{index}{suffix}'] = composite(base, layers)
    outputs, sprites = {}, {}
    for key, (width, height, pixels) in sorted({**native, **terrain}.items()):
        data = png_bytes(width, height, pixels)
        name = hashlib.sha256(data).hexdigest()[:24] + '.png'
        if name in outputs and outputs[name] != data:
            raise ValueError(f'Sprite hash collision: {key}')
        outputs[name] = data
        sprites[key] = f'assets/sprites/{name}'

    def property_name(key):
        return key if key.isidentifier() else json.dumps(key)

    sprite_fields = ''.join(f'  {property_name(key)}: {json.dumps(url)},\n'
                            for key, url in sorted(sprites.items()))
    floor_fields = ''.join(
        f'  {property_name(name)}: {json.dumps("#" + "".join(f"{c:02x}" for c in palette["G"][:3]))},\n'
        for name, palette in sorted(floors['palettes'].items())
    )
    layout_fields = ''
    for kind, variants in layouts.items():
        def placement_field(p):
            return f'{{ name: {json.dumps(p["name"])}, x: {p["x"]}, y: {p["y"]} }}'
        if len(variants) == 1 and len(variants[0]) == 1:
            layout_fields += f'  {property_name(kind)}: [[{placement_field(variants[0][0])}]],\n'
            continue
        layout_fields += f'  {property_name(kind)}: [\n'
        for placements in variants:
            if len(placements) == 1:
                layout_fields += f'    [{placement_field(placements[0])}],\n'
                continue
            layout_fields += '    [\n'
            for p in placements:
                layout_fields += f'      {placement_field(p)},\n'
            layout_fields += '    ],\n'
        layout_fields += '  ],\n'
    model_fields = "".join(
        f"  {property_name(key)}: {json.dumps(model)},\n"
        for key, model in sorted(gameplay_models(lineups).items())
    )
    geometry = {path.parent.name: terrain_canvas(path)
                for path in sorted((ART / 'terrain').glob('*/canvas.json'))}
    metadata = (
        "// Generated by scripts/build-pixel-art.py; edit pixel/art matrices and palettes.\n"
        + "export const pixelSprites: Readonly<Record<string, string>> = {\n" + sprite_fields + "};\n"
        + "\nexport const pixelFloorColors: Readonly<Record<string, string>> = {\n" + floor_fields + "};\n"
        + "\nexport const pixelTerrainLayouts: Readonly<\n"
        + "  Record<string, readonly (readonly { name: string; x: number; y: number }[])[]>\n"
        + "> = {\n" + layout_fields + "};\n"
        + "\nexport const pixelTerrainGeometry: Readonly<Record<string, { width: number; height: number; anchorX: number; anchorY: number; z: number }>> = " + json.dumps(geometry) + ";\n"
        + "\nexport const pixelUnitModels: Readonly<Record<string, string>> = {\n"
        + model_fields + "};\n"
    )
    return outputs, metadata


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Verify generated assets without writing files")
    args = parser.parse_args()
    outputs, metadata = pixel_sprites()
    products = {ROOT / "public/assets/sprites" / name: data for name, data in outputs.items()}
    products[ROOT / "src/presentation/pixel-art.ts"] = metadata.encode()
    directory = ROOT / 'public/assets/sprites'
    expected = {directory / name for name in outputs}
    obsolete = set(directory.glob('*.png')) - expected
    if args.check and obsolete:
        raise ValueError('Obsolete generated sprites; regenerate with npm run art:build')
    if not args.check:
        directory.mkdir(parents=True, exist_ok=True)
        for path in obsolete:
            path.unlink()
    for path, data in products.items():
        if args.check:
            if not path.exists() or path.read_bytes() != data:
                raise ValueError(f"{path}: regenerate with npm run art:build")
        elif not path.exists() or path.read_bytes() != data:
            path.write_bytes(data)
    print(f"{'Verified' if args.check else 'Built'} standalone pixel sprites and metadata; sources are read-only")


if __name__ == "__main__":
    main()
