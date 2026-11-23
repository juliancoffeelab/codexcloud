"""Render accepted fixtures with concise, visible stockade and exit annotations."""
import argparse
import subprocess
import sys

from build import CAMPAIGNS, OUT, ROOT, read, write


def render(variant):
    stage = OUT / 'catalog-final' / CAMPAIGNS / f'rl-hollowbank-extraction-{variant}'
    target = OUT / 'renders' / variant
    helper = ROOT / 'tmp/rl/new-env-validation/render.py'
    subprocess.run([sys.executable, helper, stage, target], cwd=ROOT, check=True)
    labels = {'crossing-post': 'Crossing post', 'stockade': 'Mara’s stockade',
              'mara-exit': 'Mara exit', 'tavin-exit': 'Tavin exit'}
    data = read(stage / 'map.json')
    annotations = [dict(col=tile['col'], row=tile['row'], text=labels[tile['id']],
                        color='#e7cb7a', dy=-40, ring=True)
                   for tile in data['tiles'] if tile.get('id') in labels]
    notes = ['Secure the crossing post, free Mara, and reach both exits before turn 14.']
    if variant == 'crossfire-nearside':
        annotations.append(dict(col=3, row=7, text='Reserve on rescue', color='#e8897a',
                                dy=-28, ring=True))
    write(target / 'overlay.json', dict(title=data['name'], notes=notes, annotations=annotations))
    command = ['node', ROOT / 'tmp/rl/current-arena-renders/renderer.mjs']
    for name in ('map', 'roster', 'scenario', 'overlay'):
        command += [f'--{name}', target / f'{name}.json']
    command += ['--reveal-terrain', '--output', target / 'preview.png']
    subprocess.run(command, cwd=ROOT, check=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('variants', nargs='+')
    for variant in parser.parse_args().variants:
        render(variant)
