"""Extract each manifest output box, retaining raw evidence and rejecting clipped art."""
import argparse
import importlib.util
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location("build_pixel_art", ROOT / "scripts/build-pixel-art.py")
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)
faction_palette = renderer.faction_palette



def extract(source, manifest_path, output):
    output = output.resolve()
    if not output.is_relative_to(ROOT / 'tmp') or output.exists():
        raise ValueError('Use a new output directory under project tmp/')
    manifest = json.loads(manifest_path.read_text())
    if manifest.get('version') != 1 or manifest.get('logicalSlot') != 64:
        raise ValueError('Unsupported manifest')
    if len(manifest.get('anchors', [])) != 3 or len(manifest.get('slots', [])) != 1:
        raise ValueError('Pinkbox requires exactly three references and one fill box')
    if manifest.get('canvas') != [1024, 1168] or manifest['slots'][0].get('rect') != [512, 576, 512, 512]:
        raise ValueError('Pinkbox requires a 2x2 layout with the fill box at bottom right')
    if manifest['slots'][0].get('native') != 64 or manifest['slots'][0].get('origin') != [0, 0]:
        raise ValueError('Pixel artwork outputs must be exactly 64×64 at origin (0,0)')
    if type(manifest['slots'][0].get('flying', False)) is not bool:
        raise ValueError("Slot 'flying' must be a boolean")
    kind = manifest['slots'][0].get('kind', 'unit')
    if kind not in ('unit', 'terrain', 'prop'):
        raise ValueError('Unknown output kind')
    if kind != 'unit' and (manifest['palette'].keys() & {'^', '*'} or manifest['slots'][0].get('flying', False)):
        raise ValueError('Terrain and props do not use faction symbols or flying metadata')
    if kind == 'unit' and any(manifest['palette'].get(s) != c for s, c in faction_palette('player').items()):
        raise ValueError('Faction colors changed since this template; refresh generation inputs')
    image = Image.open(source)
    if image.format != 'PNG':
        raise ValueError('Use the unchanged PNG output, not JPEG')
    image = image.convert('RGBA')
    mw, mh = manifest['canvas']
    if abs((image.width / image.height) / (mw / mh) - 1) > 0.02:
        raise ValueError('Generated aspect ratio differs: inspect layout and update explicit slot geometry before extraction')
    output.mkdir(parents=True)
    (output / 'raw.png').write_bytes(source.read_bytes())
    (output / 'manifest.json').write_bytes(manifest_path.read_bytes())
    palette = manifest['palette']
    colors = [(s, c) for s, c in palette.items() if c[3]]
    cache = {}

    def classify(pixel):
        if pixel not in cache:
            r, g, b, a = pixel
            if a == 0 or (r > 110 and b > 90 and r - g > 65 and b - g > 65):
                cache[pixel] = '.'
            else:
                cache[pixel] = min(colors, key=lambda item: sum((pixel[k] - item[1][k]) ** 2 for k in range(3)))[0]
        return cache[pixel]

    failures = []
    for index, slot in enumerate(manifest['slots'], 1):
        x, y, w, h = slot['rect']
        if min(x, y) < 0 or min(w, h) <= 0 or x + w > mw or y + h > mh:
            raise ValueError('Slot lies outside manifest canvas')
        bounds = (round(x * image.width / mw), round(y * image.height / mh),
                  round((x + w) * image.width / mw), round((y + h) * image.height / mh))
        crop = image.crop(bounds)
        if min(crop.size) < 64:
            raise ValueError('Generated slot is smaller than native grid')
        pixels = crop.load()
        rows = []
        for yy in range(64):
            row = ''
            for xx in range(64):
                counts = Counter(classify(pixels[px, py])
                                 for py in range(yy * crop.height // 64, (yy + 1) * crop.height // 64)
                                 for px in range(xx * crop.width // 64, (xx + 1) * crop.width // 64))
                total = sum(counts.values())
                foreground = total - counts.pop('.', 0)
                row += counts.most_common(1)[0][0] if foreground / total >= 0.35 else '.'
            rows.append(row)
        size = slot['native']; ox, oy = slot['origin']
        if size != 64 or (ox, oy) != (0, 0):
            raise ValueError('Pixel artwork outputs must be exactly 64×64 without cropping')
        outside = [[xx, yy] for yy, row in enumerate(rows) for xx, s in enumerate(row)
                   if s != '.' and not (ox <= xx < ox + size and oy <= yy < oy + size)]
        directory = output / f'{kind}-{index}'
        directory.mkdir()
        crop.save(directory / 'crop.png')
        (directory / 'extracted-64.txt').write_text('\n'.join(rows) + '\n')
        (directory / 'extraction.json').write_text(json.dumps({'label': slot['label'], 'source': list(image.size),
            'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(), 'bounds': bounds,
            'native': size, 'origin': [ox, oy], 'outside': outside,
            'materialOwnership': 'Provisional color quantization: inspect intended surfaces and boundaries'}, indent=2) + '\n')
        materials = {s: c for s, c in palette.items() if s not in '^*'}
        if kind == 'unit' and slot.get('flying', False):
            materials['flying'] = True
        (directory / 'palette.json').write_text(json.dumps(materials, indent=2) + '\n')
        if outside:
            failures.append(f'{slot["label"]}: {len(outside)} foreground pixels outside requested crop')
            continue
        native = [r[ox:ox + size] for r in rows[oy:oy + size]]
        if not any(s != '.' for r in native for s in r):
            failures.append(f'{slot["label"]}: empty output box')
            continue
        if kind == 'unit':
            try:
                renderer.require_grounded(native, palette, slot.get('flying', False), slot['label'])
            except ValueError as error:
                failures.append(str(error))
                continue
        elif kind == 'terrain':
            # Reject, never silently clip artwork outside the native terrain hex.
            outside_hex = [(x, y) for y, row in enumerate(native) for x, s in enumerate(row)
                           if s != '.' and (abs(x + .5 - 32) > 16 * math.sqrt(3)
                           or abs(y + .5 - 32) > 32 - abs(x + .5 - 32) / math.sqrt(3))]
            if outside_hex:
                failures.append(f'{slot["label"]}: {len(outside_hex)} pixels outside the pointy-top hex')
                continue
        filename = 'sprite.txt' if kind == 'unit' else 'tile.txt'
        (directory / filename).write_text('\n'.join(native) + '\n')
        print(directory / filename)
    if failures:
        raise ValueError('; '.join(failures) + '. Evidence retained; inspect or regenerate, never silently trim anatomy.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('manifest', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    extract(args.source, args.manifest, args.output)
