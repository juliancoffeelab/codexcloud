"""Terrain inspection, recruitment placement, and tactical action regressions."""
from functools import partial
from http.server import ThreadingHTTPServer
from pathlib import Path
from threading import Thread
import shutil
from playwright.sync_api import sync_playwright
from smoke import QuietHandler, TOOLS, snapshot, choose_style, new_battle

server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(Path(__file__).resolve().parents[1] / 'dist')))
Thread(target=server.serve_forever, daemon=True).start()
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=shutil.which('chromium'))
        context = browser.new_context(viewport={'width': 390, 'height': 844})
        context.add_init_script(TOOLS)
        page = context.new_page()
        errors = []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.goto(f'http://127.0.0.1:{server.server_port}', wait_until='networkidle')
        terrain = page.locator('#terrain-panel')
        def inspect(tile, name):
            page.click(f'[data-tile="{tile}"]')
            assert terrain.locator('h2').inner_text() == name
            assert terrain.locator('img').evaluate('img => img.complete && img.naturalWidth > 0')
        screenshots = Path('/tmp/hex-command-cards'); screenshots.mkdir(exist_ok=True)
        initial = snapshot(page)
        assert not terrain.is_visible() and not page.locator('#unit-panel').is_visible()
        inspect('1,3', 'Plains')
        inspect('1,3', 'Plains')  # Repeated unit selection still deselects tactically.
        assert not page.locator('#unit-panel').is_visible()
        for tile, name, mechanics in [
            ('0,0', 'Water', ['impassable']),
            ('5,2', 'Forest', ['Movement: 2 (scouts: 1)', '−1 damage from ranged attacks', 'Blocks ranged shots']),
            ('3,2', 'Hills', ['Movement: 2 (scouts: 1)', '−1 damage from all attacks', 'Blocks ranged shots']),
            ('3,1', 'Village', ['Unclaimed village', '+2 gold', '2 HP', 'start of its turn']),
            ('10,3', 'Keep', ['Enemy keep', 'adjacent empty land', 'cannot be captured']),
        ]:
            inspect(tile, name)
            assert all(text in terrain.inner_text() for text in mechanics)
            assert snapshot(page) == initial
            if name == 'Forest': page.screenshot(path=str(screenshots / 'mobile-empty-forest.png'), full_page=True)
        assert 'enemy unit' in page.locator('#unit-panel').inner_text().lower()
        page.keyboard.press('Escape')
        assert not terrain.is_visible() and not page.locator('#unit-panel').is_visible()
        inspect('1,2', 'Plains')
        inspect('3,1', 'Village')  # Move/capture executes on the first destination click.
        assert next(u for u in snapshot(page)['units'] if u['id'] == 'u3')['tile'] == '3,1'
        assert 'Your village' in terrain.inner_text()
        for style in ('pixel', 'low-poly', 'normal'):
            choose_style(page, style)
            assert style in terrain.locator('img').get_attribute('src')
        inspect('0,3', 'Keep')
        page.screenshot(path=str(screenshots / 'mobile-commander-keep.png'), full_page=True)
        page.set_viewport_size({'width': 1280, 'height': 900})
        page.screenshot(path=str(screenshots / 'desktop-commander-keep.png'), full_page=True)
        page.set_viewport_size({'width': 390, 'height': 844})
        assert page.locator('.unit-info [data-recruit]').count() == 3
        assert page.locator('#unit-panel > .recruit-list, #unit-panel > .recruit-title').count() == 0
        page.click('[data-recruit="soldier"]')
        destination = page.locator('.hex.recruitable').first
        tile = destination.get_attribute('data-tile')
        destination.click()
        assert next(u for u in snapshot(page)['units'] if u['id'] == 'u7')['tile'] == tile
        assert terrain.is_visible()
        print('PASS: terrain cards, unit terrain, ownership, rules, artwork styles, first-click movement and inline recruitment')
        # Read-only inspection remains available during the animated enemy turn.
        page.click('#end-turn')
        page.locator('[data-tile="0,0"]').dispatch_event('click')
        assert terrain.locator('h2').inner_text() == 'Water'
        page.keyboard.press('Escape')
        assert not terrain.is_visible()
        page.wait_for_function("window.hexSmokeTools.get_battle_state.execute().round === 2")
        assert not terrain.is_visible() and not page.locator('#unit-panel').is_visible()
        # Restore an attack setup using the normal save mechanism.
        new_battle(page)
        page.evaluate("""() => {const key='hex-command-battle';const s=JSON.parse(localStorage.getItem(key));s.state.units.find(u=>u.id==='u2').tile='8,3';localStorage.setItem(key,JSON.stringify(s));}""")
        page.reload(wait_until='networkidle')
        inspect('8,3', 'Plains')
        inspect('9,3', 'Plains')
        assert page.locator('.duel').is_visible()
        page.click('#attack')
        assert next(u for u in snapshot(page)['units'] if u['id'] == 'u5')['hp'] == 8
        page.evaluate("""() => {const key='hex-command-battle';const s=JSON.parse(localStorage.getItem(key));s.state.winner='player';s.state.units=s.state.units.filter(u=>!(u.side==='enemy'&&u.type==='commander'));localStorage.setItem(key,JSON.stringify(s));}""")
        page.reload(wait_until='networkidle')
        final = snapshot(page)
        inspect('5,2', 'Forest')
        assert snapshot(page) == final
        assert page.locator('#play-again').is_visible()
        page.keyboard.press('Escape')
        assert not terrain.is_visible()
        assert page.locator('#play-again').is_visible()
        print('PASS: off-turn and finished terrain inspection, Escape, no new-turn selection, attack-target terrain, combat')
        for width in (320, 390, 1280):
            page.set_viewport_size({'width': width, 'height': 844})
            inspect('3,1', 'Village')
            assert page.evaluate('document.documentElement.scrollWidth <= innerWidth')
        assert not errors, errors
        browser.close()
finally:
    server.shutdown()
    server.server_close()
