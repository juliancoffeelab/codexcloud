"""Render exact arena map and roster inputs with the repository renderer."""
import shutil
import subprocess

from build import MODES, OUT, POST, ROOT, hex_at, read, write


def main():
    catalog = OUT / 'validated/catalog'
    renderer = ROOT / 'tmp/rl/current-arena-renders/renderer.mjs'
    for mode in MODES:
        ident = f'rl-marlow-cut-pass-{mode}'
        fixture = catalog / 'game/scenario' / ident
        output = OUT / 'renders' / mode
        output.mkdir(parents=True, exist_ok=True)
        for filename in ('map.json', 'scenario.json', 'roster.json'):
            shutil.copy2(fixture / filename, output / filename)
        roster = read(output / 'roster.json')
        player, enemy = roster['factions']
        commander = next(u for u in enemy['units'] if u['role'] == 'leader')
        knight = next(u for u in player['units'] if u['type'] == 'knight')
        annotations = [dict(**hex_at(POST), text='Watch post', color='#f2c765', dy=-35, ring=True),
                       dict(**commander['tile'], text='Enemy commander', color='#ea9f8f', dy=-28, ring=True),
                       dict(**knight['tile'], text='Knight', color='#a9d7ff', dy=-32, ring=True)]
        notes = ['Hold the watch post for six rounds or defeat the enemy commander.',
                 'Both starts use the same map and forces. The enemy commander starts at a different hex.']
        write(output / 'overlay.json', dict(title=f'Marlow Cut — {mode.title()} start', notes=notes,
                                           annotations=annotations))
        subprocess.run(['node', str(renderer), '--map', str(output / 'map.json'), '--roster',
                        str(output / 'roster.json'), '--scenario', str(output / 'scenario.json'),
                        '--overlay', str(output / 'overlay.json'), '--reveal-terrain', '--output',
                        str(output / 'preview.png')], cwd=ROOT, check=True)


if __name__ == '__main__':
    main()
