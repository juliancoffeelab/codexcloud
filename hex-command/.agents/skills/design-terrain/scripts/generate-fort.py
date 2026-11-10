"""Generate complete fort tiles from explicit walls, without map input."""
import argparse
import json
import math
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
APOTHEM = 16 * math.sqrt(3)
SPACING = 2 * APOTHEM
CENTER = 31.5
# Every wall is a complete inward-facing strip along its owning hex edge.
WALL_DEPTH = 9
# Complete tower footprint; body height and the approved canvas remain unchanged.
TOWER_RADIUS_X, TOWER_RADIUS_Y = 8, 4
# A low footing covers the wall toe without moving the crown or junction anchor.
# Its front reach is 4+7=11 pixels: ceil(9 / sin(60 degrees)) for the wall profile.
TOWER_FOOTING_RADIUS_X, TOWER_FOOTING_RADIUS_Y = 10, 7
TOWER_FOOTING_FORWARD, TOWER_FOOTING_HEIGHT = 4, 3


def model_x(x):
    # The regular-hex model is sampled on the renderer's integer 56x48 lattice.
    # Half-column translations are exactly 28 source pixels, never 27.71.
    return CENTER + (x - CENTER) * APOTHEM / 28


def paving_symbol(x, y):
    # The hex lattice translates by (SPACING, 0) or (SPACING/2, 48).
    # Eight bricks span SPACING; twelve courses span 48. Both translations
    # preserve the pattern, including its alternate-course half-brick offset.
    course = math.floor((y - CENTER) / 4)
    cross = (y - CENTER) % 4
    along = ((x - CENTER) / (SPACING / 8) + .5 * (course % 2)) % 1
    return 'g' if cross < 1 or along < .2 else 'G'


def edge_segment(direction):
    # The two fixed lattice corners belong to this edge. They never depend
    # on a guessed neighbor mask, an inset intersection or a halo volume.
    angle = direction * math.pi / 3
    return ((CENTER + 32 * math.cos(angle - math.pi / 6),
             CENTER + 32 * math.sin(angle - math.pi / 6)),
            (CENTER + 32 * math.cos(angle + math.pi / 6),
             CENTER + 32 * math.sin(angle + math.pi / 6)))


def wall_sample(x, y, edge):
    direction, p, q = edge
    angle = direction * math.pi / 3
    nx, ny = math.cos(angle), math.sin(angle)
    depth = APOTHEM - ((x - CENTER) * nx + (y - CENTER) * ny)
    vx, vy = q[0] - p[0], q[1] - p[1]
    t = ((x - p[0]) * vx + (y - p[1]) * vy) / (vx * vx + vy * vy)
    return depth, max(0, min(1, t))


def wall_material(depth, along):
    # The approved wall profile: outer cap, recessed walk, inner cap and
    # crenellated masonry face, entirely inside the owning tile.
    if depth < 1.5:
        return 'C'
    if depth < 2.5:
        return 'c'
    if depth < 4:
        return 'C'
    if depth < 6 and int(along * 8) % 2 == 0:
        return 'C'
    return 'k' if int(depth) % 3 == 0 else 'c'


def masonry(walls, base):
    edges = [(i, *edge_segment(i)) for i in sorted(walls)]
    rows = [['.'] * 64 for _ in range(64)]
    for y in range(64):
        for x in range(64):
            if base[y][x] == '.':
                continue
            hits = []
            for edge in edges:
                depth, along = wall_sample(model_x(x), y, edge)
                if 0 <= depth < WALL_DEPTH:
                    hits.append((depth, edge[0], along))
            if hits:
                depth, _, along = min(hits)
                rows[y][x] = wall_material(depth, along)
    return [''.join(row) for row in rows]


def validate_paving():
    shifts = [(SPACING, 0), (SPACING / 2, 48), (-SPACING / 2, 48)]
    for i in range(256):
        x, y = i * .317 + .13, i * .419 + .27
        value = paving_symbol(x, y)
        if any(paving_symbol(x + dx, y + dy) != value for dx, dy in shifts):
            raise ValueError('Paving changes phase across a hex neighbor offset')


# Fixed native canvas and lattice. All towers stand on these exact hex corners.
WIDTH, HEIGHT, PAD_X, PAD_Y = 80, 96, 8, 16
CORNERS = [(28, -16), (28, 16), (0, 32), (-28, 16), (-28, -16), (0, -32)]
CASTLES = ('castle-wall', 'castle-bell')
DIRECTIONS = ['e', 'se', 'sw', 'w', 'nw', 'ne']


def direction_mask(edges):
    if not isinstance(edges, list) or any(edge not in DIRECTIONS for edge in edges) or len(set(edges)) != len(edges):
        raise ValueError('Walls require unique explicit hex directions')
    return sum(1 << DIRECTIONS.index(edge) for edge in edges)


def validate_variants(kind, variants):
    if kind not in CASTLES or not isinstance(variants, list) or not variants:
        raise ValueError(f'Invalid castle family {kind}')
    accepted = {}
    for variant in variants:
        if not isinstance(variant, dict) or set(variant) != {'id', 'walls'}:
            raise ValueError(f'{kind}: explicit id and walls are required')
        walls = direction_mask(variant['walls'])
        if kind == 'castle-bell' and walls != 63:
            raise ValueError('Bell tiles have six enclosing walls')
        key = str(walls)
        if variant['id'] != key or key in accepted:
            raise ValueError(f'{kind}: invalid or duplicate id {variant["id"]}')
        accepted[key] = variant
    return accepted


def read_catalog(path):
    document = json.loads(Path(path).read_text())
    if set(document) != {'version', 'families'} or document['version'] != 3 or set(document['families']) != set(CASTLES):
        raise ValueError('Expected the explicit version-three castle tile catalog')
    return {kind: validate_variants(kind, document['families'][kind]) for kind in CASTLES}


def tower_body_pixel(dx, dy):
    value, height = '.', 0
    for z in range(10, -1, -1):
        radius = (dx / TOWER_RADIUS_X) ** 2 + ((dy + z) / TOWER_RADIUS_Y) ** 2
        if radius > 1:
            continue
        height = z
        value = ('C' if radius > .5 else 'c') if z == 10 else ('C' if dx < -TOWER_RADIUS_X / 2 else 'k' if dx > TOWER_RADIUS_X / 2 else 'c')
        if z in (3, 7) and dx >= -TOWER_RADIUS_X / 2:
            value = 'k'
        break
    for i in range(6):
        angle = i * math.pi / 3
        cx, cy = round((TOWER_RADIUS_X - 2) * math.cos(angle)), round(-10 + (TOWER_RADIUS_Y - 1) * math.sin(angle))
        if cx - 1 <= dx < cx + 1 and cy - 2 <= dy < cy + 1:
            value, height = 'C', 10
    return value, height


def tower_pixel(dx, dy):
    value, height = tower_body_pixel(dx, dy)
    if value != '.':
        return value, height
    for height in range(TOWER_FOOTING_HEIGHT, -1, -1):
        radius = (dx / TOWER_FOOTING_RADIUS_X) ** 2 + ((dy - TOWER_FOOTING_FORWARD + height) / TOWER_FOOTING_RADIUS_Y) ** 2
        if radius > 1:
            continue
        if height == TOWER_FOOTING_HEIGHT:
            value = 'C' if radius > .55 else 'c'
        elif height == 0:
            value = 'k' if dx > -TOWER_FOOTING_RADIUS_X / 2 else 'c'
        else:
            value = 'C' if dx < -TOWER_FOOTING_RADIUS_X * .65 else 'k' if dx > TOWER_FOOTING_RADIUS_X * .65 else 'c'
        return value, height
    return '.', 0


def stamp_structure(rows, structure, start, stop):
    glyphs, offset_x, offset_y, _ = structure
    for y in range(start, stop):
        for x, symbol in enumerate(glyphs[y]):
            if symbol != '.':
                rows[y + PAD_Y + offset_y][x + PAD_X + offset_x] = symbol


def complete_tile(walls, structure):
    """Bake paving, lower structure, boundary stonework, then upper structure."""
    base = [''.join('G' if abs(x - CENTER) <= 28 and abs(y - CENTER) <= 32 - abs(x - CENTER) * 4 / 7 else '.'
                    for x in range(64)) for y in range(64)]
    masonry_rows = masonry(walls, base)
    rows = [['.'] * WIDTH for _ in range(HEIGHT)]
    for y in range(64):
        for x in range(64):
            if base[y][x] != '.':
                rows[y + PAD_Y][x + PAD_X] = 'P' if paving_symbol(model_x(x), y) == 'G' else 'p'
    if structure is not None:
        glyphs, _, _, split_row = structure
        stamp_structure(rows, structure, split_row, len(glyphs))
    # Boundary stonework owns its overlap with structures baked into the courtyard.
    for y, row in enumerate(masonry_rows):
        for x, symbol in enumerate(row):
            if symbol != '.':
                rows[y + PAD_Y][x + PAD_X] = symbol
    # Edge i has endpoints i and i+1. No neighboring geometry moves or splits a tower.
    corners = {corner for edge in walls for corner in (edge, (edge + 1) % 6)}
    for corner in sorted(corners):
        vx, vy = CORNERS[corner]
        for y in range(HEIGHT):
            for x in range(WIDTH):
                symbol, _ = tower_pixel(x - PAD_X - CENTER - vx, y - PAD_Y - CENTER - vy)
                if symbol != '.':
                    rows[y][x] = symbol
    if structure is not None:
        stamp_structure(rows, structure, 0, split_row)
    return [''.join(row) for row in rows]


def generate(output, materials, paving_colors, catalog):
    validate_paving()
    families = read_catalog(catalog)
    variants = {(kind, key): variant
                for kind, definitions in families.items() for key, variant in definitions.items()}
    palette = {'.': [0, 0, 0, 0], 'P': paving_colors['G'], 'p': paving_colors['g'],
               **{key: materials[key] for key in ('O', 'C', 'c', 'k')}}
    for kind in CASTLES:
        folder = output / kind
        folder.mkdir(parents=True)
        registry = []
        for (terrain, id), variant in sorted(variants.items()):
            if terrain != kind:
                continue
            structure = None
            if kind == 'castle-bell':
                bell = (REPO / 'pixel/art/terrain/props/bell/structure/tile.txt').read_text().splitlines()
                structure = (bell, 6, 0, 32)
            rows = complete_tile([DIRECTIONS.index(d) for d in variant['walls']], structure)
            (folder / f'{id}.txt').write_text('\n'.join(rows) + '\n')
            registry.append(variant)
        kind_palette = {**palette, **json.loads((REPO / 'pixel/art/terrain/props/bell/structure/palette.json').read_text())} if kind == 'castle-bell' else palette
        (folder / 'palette.json').write_text(json.dumps(kind_palette, indent=2) + '\n')
        (folder / 'canvas.json').write_text(json.dumps(dict(width=WIDTH, height=HEIGHT, anchorX=40, anchorY=48, z=3 if kind == 'castle-bell' else 2), indent=2) + '\n')
        (folder / 'variants.json').write_text(json.dumps(registry, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--palette', type=Path, required=True)
    parser.add_argument('--paving-floor', required=True)
    parser.add_argument('--catalog', type=Path, required=True, help='Map-independent castle tile definitions')
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(REPO / 'tmp') or output.exists():
        parser.error('Use a new candidate directory under tmp/')
    floors = json.loads((REPO / 'pixel/art/terrain/floors.json').read_text())['palettes']
    generate(output, json.loads(args.palette.read_text()), floors[args.paving_floor], args.catalog)


if __name__ == '__main__':
    main()
