"""Apply individually reviewed, guarded edits to a new candidate; preserve the input."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
from review import ROOT, render, faction_palette, previews
require_canvas = render.require_canvas


def apply(matrix, edits_path, output):
    render.rgba_pixels(matrix, 'player')
    output = output.resolve()
    if not output.is_relative_to(ROOT / 'tmp') or output.exists():
        raise ValueError('Use a new candidate directory under project tmp/')
    document = json.loads(edits_path.read_text())
    if document['matrix_sha256'] != hashlib.sha256(matrix.read_bytes()).hexdigest():
        raise ValueError('Matrix changed since review; inspect it again')
    palette_path = matrix.parent / 'palette.json'
    if document['palette_sha256'] != hashlib.sha256(palette_path.read_bytes()).hexdigest():
        raise ValueError('Palette changed since review; inspect it again')
    rows = [list(r) for r in render.read_matrix(matrix)]
    require_canvas(rows, matrix)
    palette = render.read_palette(matrix)
    symbols = set(palette) | (set(faction_palette('player')) if matrix.stem == 'sprite' else set())
    seen = set()
    if not document['edits']:
        raise ValueError('No edits supplied')
    for edit in document['edits']:
        x, y = edit['x'], edit['y']; before, after = edit['from'], edit['to']
        if type(x) is not int or type(y) is not int or not (0 <= x < len(rows[0]) and 0 <= y < len(rows)):
            raise ValueError('Edit coordinates must be native integer pixels inside the matrix')
        if (x, y) in seen or rows[y][x] != before or before == after or after not in symbols:
            raise ValueError(f'Invalid, duplicate, stale or empty edit at {x},{y}')
        if not edit.get('reason', '').strip() or not edit.get('material', '').strip():
            raise ValueError('Every edit needs its local material and a reason')
        if (before in '.O' or after in '.O') and not edit.get('boundary_reviewed'):
            raise ValueError(f'Outline/transparency edit at {x},{y} needs boundary_reviewed: true')
        rows[y][x] = after; seen.add((x, y))
    output.mkdir(parents=True)
    (output / matrix.name).write_text('\n'.join(''.join(r) for r in rows) + '\n')
    # Scratch terrain palettes must carry resolved floor shades so the candidate
    # can render outside its canonical folder. The baseline remains byte-exact.
    candidate_palette = json.loads(palette_path.read_text())
    candidate_palette.update(palette)
    (output / 'palette.json').write_text(json.dumps(candidate_palette, indent=2) + '\n')
    before_dir = output / 'before'; before_dir.mkdir()
    shutil.copyfile(matrix, before_dir / matrix.name); shutil.copyfile(palette_path, before_dir / 'palette.json')
    # Ownership stays a review input, never a blanket painting mask.
    ownership = matrix.with_suffix('.ownership.json')
    if ownership.exists():
        shutil.copyfile(ownership, output / ownership.name)
    (output / 'changes.json').write_text(json.dumps(document, indent=2) + '\n')
    previews([matrix, output / matrix.name], output / 'comparison.png')
    print(json.dumps({'candidate': str(output / matrix.name), 'changedPixels': len(seen)}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('matrix', type=Path)
    parser.add_argument('edits', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    apply(args.matrix, args.edits, args.output)
