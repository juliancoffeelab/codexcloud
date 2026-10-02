"""Terrain-first purchases, affordability and unavailable keep conditions."""
from functools import partial
from http.server import ThreadingHTTPServer
from pathlib import Path
from threading import Thread
import shutil
from playwright.sync_api import sync_playwright
from smoke import QuietHandler, TOOLS, snapshot, new_battle

server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(Path(__file__).resolve().parents[1] / 'dist')))
Thread(target=server.serve_forever, daemon=True).start()
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=shutil.which('chromium'))
        context = browser.new_context(viewport={'width': 390, 'height': 844}, is_mobile=True, has_touch=True)
        context.add_init_script(TOOLS)
        page = context.new_page()
        errors = []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.goto(f'http://127.0.0.1:{server.server_port}', wait_until='networkidle')
        def inspect(tile): page.click(f'[data-tile="{tile}"]')
        def options(): return page.locator('#terrain-panel [data-recruit]')
        def edit_state(script):
            new_battle(page)
            page.evaluate("""script => { const key='hex-command-battle';const save=JSON.parse(localStorage.getItem(key));new Function('state',script)(save.state);localStorage.setItem(key,JSON.stringify(save)); }""", script)
            page.reload(wait_until='networkidle')
        for kind, price in [('soldier', 8), ('archer', 8), ('scout', 5)]:
            new_battle(page)
            inspect('0,2')
            assert options().count() == 3
            assert not page.locator('#unit-panel').is_visible()
            before = snapshot(page)
            page.click(f'#terrain-panel [data-recruit="{kind}"]')
            after = snapshot(page)
            recruited = next(unit for unit in after['units'] if unit['id'] == 'u7')
            assert (recruited['type'], recruited['tile'], recruited['moved'], recruited['attacked']) == (kind, '0,2', True, True)
            assert after['gold']['player'] == before['gold']['player'] - price
            assert options().count() == 0
            assert page.locator('#undo-move').is_disabled()
        new_battle(page)
        inspect('0,3')
        assert 'Select an empty adjacent hex to recruit' in page.locator('#terrain-panel').inner_text()
        assert page.locator('#unit-panel [data-recruit]').count() == 0
        inspect('0,2')  # A selected commander moves normally instead of opening a purchase menu.
        assert next(unit for unit in snapshot(page)['units'] if unit['id'] == 'u1')['tile'] == '0,2'
        assert options().count() == 0
        new_battle(page)
        for tile in ['1,3', '0,0', '10,2', '5,2']:
            inspect(tile)
            assert options().count() == 0  # Occupied, impassable, enemy keep neighbor, and distant.
            page.keyboard.press('Escape')
        edit_state('state.gold.player=5')
        inspect('0,2')
        assert options().count() == 3
        assert page.locator('[data-recruit="soldier"]').is_disabled()
        assert page.locator('[data-recruit="archer"]').is_disabled()
        assert page.locator('[data-recruit="scout"]').is_enabled()
        edit_state('state.gold.player=0')
        inspect('0,2')
        assert options().count() == 3 and page.locator('#terrain-panel [data-recruit]:disabled').count() == 3
        assert 'Not enough gold' in page.locator('#terrain-panel').inner_text()
        edit_state("state.units.find(u=>u.id==='u1').tile='2,3'")
        inspect('0,2')
        assert options().count() == 3 and page.locator('#terrain-panel [data-recruit]:disabled').count() == 3
        assert 'Return your commander' in page.locator('#terrain-panel').inner_text()
        new_battle(page)
        page.click('#end-turn')
        page.locator('[data-tile="0,2"]').dispatch_event('click')
        assert options().count() == 3 and page.locator('#terrain-panel [data-recruit]:disabled').count() == 3
        assert 'Recruit on your turn' in page.locator('#terrain-panel').inner_text()
        page.wait_for_function("window.hexSmokeTools.get_battle_state.execute().round === 2")
        inspect('0,2')
        assert page.locator('[data-recruit="soldier"]').is_enabled()
        for width in [320, 390, 1280]:
            page.set_viewport_size({'width': width, 'height': 844})
            assert page.evaluate('document.documentElement.scrollWidth <= innerWidth')
        assert not errors, errors
        browser.close()
        print('PASS: exact terrain purchases, prices, next-turn restriction, movement precedence, gold, occupancy, keep ownership, commander location and turn gating')
finally:
    server.shutdown()
    server.server_close()
