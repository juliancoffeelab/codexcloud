"""Author native, combinable fort frames offline. Runtime only selects images."""
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


def wall_segments(mask):
    return [(i, *edge_segment(i)) for i in range(6) if not mask & (1 << i)]


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


def masonry(mask, gate, base):
    edges = wall_segments(mask)
    exposed = {i for i, _, _ in edges}
    gate_edge = next((i for i in exposed if (i - 1) % 6 in exposed
                      and (i + 1) % 6 in exposed), None) if gate else None
    rows = [['.'] * 64 for _ in range(64)]
    for y in range(64):
        for x in range(64):
            if base[y][x] == '.':
                continue
            hits = []
            for edge in edges:
                depth, along = wall_sample(model_x(x), y, edge)
                if edge[0] == gate_edge and .25 < along < .75:
                    continue
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


def validate_joined_masonry(rows, mask, gate):
    # A three-sided boundary is one connected piece; a gate splits it in two.
    # This catches detached towers, regardless of their symbol or color.
    if mask not in GATES:
        return
    from collections import deque
    points = {(x, y) for y, row in enumerate(rows) for x, value in enumerate(row) if value != '.'}
    count = 0
    while points:
        count += 1
        queue = deque([points.pop()])
        while queue:
            x, y = queue.popleft()
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    neighbor = x + dx, y + dy
                    if neighbor in points:
                        points.remove(neighbor)
                        queue.append(neighbor)
    expected = 2 if gate else 1
    if count != expected:
        raise ValueError(f'Masonry mask {mask}: expected {expected} joined pieces, found {count}')


# A 64-pixel logical hex, padded without rescaling for explicitly allowed overflow.
WIDTH, HEIGHT, PAD_X, PAD_Y = 80, 96, 8, 16

GATES = {7, 14, 28, 35, 49, 56}
CORNERS = [(28, -16), (28, 16), (0, 32), (-28, 16), (-28, -16), (0, -32)]
CASTLES = ('castle-wall', 'castle-gate', 'castle-bell')
DIRECTIONS = ['e', 'se', 'sw', 'w', 'nw', 'ne']


def variant_id(mask, corners):
    return str(mask | sum(code << (6 + 9 * i) for i, code in enumerate(corners)))


def incident_centers(corner):
    vx, vy = CORNERS[corner]
    centers = [(-vx, -vy)]
    for edge in (corner, (corner - 1) % 6):
        angle = edge * math.pi / 3
        centers.append((round(56 * math.cos(angle)) - vx,
                        round(56 * math.sin(angle) * 48 / (28 * math.sqrt(3))) - vy))
    return sorted(centers, key=lambda p: (p[1], p[0]))



def validate_variants(kind, variants):
    if kind not in CASTLES or not isinstance(variants, list) or not variants:
        raise ValueError(f'Invalid castle family {kind}')
    accepted = {}
    for variant in variants:
        if not isinstance(variant, dict) or set(variant) != {'id', 'mask', 'wallCorners'}:
            raise ValueError(f'{kind}: explicit id, mask and wallCorners are required')
        mask, codes = variant['mask'], variant['wallCorners']
        if type(mask) is not int or not 0 <= mask < 64:
            raise ValueError(f'{kind}: invalid wall mask')
        if kind == 'castle-gate' and mask not in GATES:
            raise ValueError('Unsupported gate wall mask')
        if kind == 'castle-bell' and mask != 0:
            raise ValueError('Bell tiles use an enclosed wall mask')
        if not isinstance(codes, list) or len(codes) != 6 or any(type(v) is not int or not 0 <= v < 512 for v in codes):
            raise ValueError(f'{kind}: six explicit corner codes are required')
        for corner, code in enumerate(codes):
            if code:
                faces, members = code & 63, code >> 6
                own = (-CORNERS[corner][0], -CORNERS[corner][1])
                own_bit = 1 << incident_centers(corner).index(own)
                if faces.bit_count() < 2 or not members & own_bit:
                    raise ValueError(f'{kind}: invalid shared tower at corner {corner}')
        key = variant_id(mask, codes)
        if variant['id'] != key or key in accepted:
            raise ValueError(f'{kind}: invalid or duplicate tile id {variant["id"]}')
        accepted[key] = dict(id=key, mask=mask, wallCorners=list(codes))
    return accepted


def read_catalog(path):
    document = json.loads(Path(path).read_text())
    if set(document) != {'version', 'families'} or document['version'] != 1 or set(document['families']) != set(CASTLES):
        raise ValueError('Expected the explicit version-one castle tile catalog')
    return {kind: validate_variants(kind, document['families'][kind]) for kind in CASTLES}



def anchor_offset(faces):
    normals = [(math.cos(i * math.pi / 3) * APOTHEM / 28, math.sin(i * math.pi / 3))
               for i in range(6) if faces & (1 << i)]
    intersections = []
    for i, (ax, ay) in enumerate(normals):
        for bx, by in normals[i + 1:]:
            det = ax * by - ay * bx
            if abs(det) > 1e-8:
                intersections.append((2 * (ay - by) / det, 2 * (bx - ax) / det))
    if not intersections:
        raise ValueError('Corner needs intersecting walls')
    vertical = [-4.5 / nx for nx, ny in normals if abs(ny) < 1e-8]
    x = sum(vertical) / len(vertical) if vertical else sum(x for x, _ in intersections) / len(intersections)
    return math.floor(x + .5), math.floor(sum(y for _, y in intersections) / len(intersections) + .5)


def tower_pixel(dx, dy):
    value, height = '.', 0
    for z in range(10, -1, -1):
        radius = (dx / 6) ** 2 + ((dy + z) / 3) ** 2
        if radius > 1:
            continue
        height = z
        value = ('C' if radius > .5 else 'c') if z == 10 else ('C' if dx < -3 else 'k' if dx > 3 else 'c')
        if z in (3, 7) and dx >= -3:
            value = 'k'
        break
    for i in range(6):
        angle = i * math.pi / 3
        cx, cy = round(4 * math.cos(angle)), round(-10 + 2 * math.sin(angle))
        if cx - 1 <= dx < cx + 1 and cy - 2 <= dy < cy + 1:
            value, height = 'C', 10
    return value, height


def complete_tile(mask, codes, gate=False):
    """One complete matrix: paving, wall masonry and complementary tower volumes."""
    base = [''.join('G' if abs(x - CENTER) <= 28 and abs(y - CENTER) <= 32 - abs(x - CENTER) * 4 / 7 else '.'
                    for x in range(64)) for y in range(64)]
    wall = masonry(mask, gate, base)
    rows = [['.'] * WIDTH for _ in range(HEIGHT)]
    for y in range(64):
        for x in range(64):
            if base[y][x] != '.':
                rows[y + PAD_Y][x + PAD_X] = wall[y][x] if wall[y][x] != '.' else ('P' if paving_symbol(model_x(x), y) == 'G' else 'p')
    for corner, code in enumerate(codes):
        if not code:
            continue
        faces, members = code & 63, code >> 6
        vx, vy = CORNERS[corner]
        ox, oy = anchor_offset(faces)
        centers = [c for i, c in enumerate(incident_centers(corner)) if members & (1 << i)]
        own = (-vx, -vy)
        if own not in centers:
            raise ValueError('A tile cannot carry another tile’s corner')
        for y in range(max(0, int(vy + oy + CENTER + PAD_Y - 10)), min(HEIGHT, int(vy + oy + CENTER + PAD_Y + 10))):
            for x in range(max(0, int(vx + ox + CENTER + PAD_X - 8)), min(WIDTH, int(vx + ox + CENTER + PAD_X + 9))):
                px, py = x - PAD_X - CENTER, y - PAD_Y - CENTER
                dx, dy = px - vx - ox, py - vy - oy - 5
                symbol, height = tower_pixel(dx, dy)
                if symbol == '.':
                    continue
                # Partition the solid volume in ground coordinates BEFORE projection.
                gx, gy = px - vx, py + height - vy
                owner = min(centers, key=lambda c: ((gx - c[0]) ** 2 + (gy - c[1]) ** 2, c[1], c[0]))
                if owner == own:
                    rows[y][x] = symbol
    return [''.join(row) for row in rows]


def generate(output, materials, paving_colors, catalog):
    validate_paving()
    families = read_catalog(catalog)
    variants = {(kind, key): (variant['mask'], variant['wallCorners'])
                for kind, definitions in families.items() for key, variant in definitions.items()}
    palette = {'.': [0, 0, 0, 0], 'P': paving_colors['G'], 'p': paving_colors['g'],
               **{key: materials[key] for key in ('O', 'C', 'c', 'k')}}
    for kind in CASTLES:
        folder = output / kind
        folder.mkdir(parents=True)
        registry = []
        for (terrain, id), (mask, codes) in sorted(variants.items()):
            if terrain != kind:
                continue
            rows = complete_tile(mask, codes, kind == 'castle-gate')
            if kind == 'castle-bell':
                # The bell is the final (higher-z) authoring layer, baked into this tile.
                rows = [list(row) for row in rows]
                bell = (REPO / 'pixel/art/terrain/props/bell/structure/tile.txt').read_text().splitlines()
                for y, row in enumerate(bell):
                    for x, symbol in enumerate(row):
                        if symbol != '.':
                            rows[y + PAD_Y][x + PAD_X + 6] = symbol
                rows = [''.join(row) for row in rows]
            (folder / f'{id}.txt').write_text('\n'.join(rows) + '\n')
            registry.append(dict(id=id, mask=mask, wallCorners=codes))
        kind_palette = {**palette, **json.loads((REPO / 'pixel/art/terrain/props/bell/structure/palette.json').read_text())} if kind == 'castle-bell' else palette
        (folder / 'palette.json').write_text(json.dumps(kind_palette, indent=2) + '\n')
        (folder / 'canvas.json').write_text(json.dumps(dict(width=WIDTH, height=HEIGHT, anchorX=40, anchorY=48, z=1), indent=2) + '\n')
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
