"""Map picker, larger battlefield, bridge inspection and saved-map browser checks."""
from functools import partial
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright
from smoke import TOOLS, snapshot, choose_style

class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *_): pass

server = ThreadingHTTPServer(('127.0.0.1',0), partial(Quiet,directory=str(Path(__file__).resolve().parents[1]/'dist')))
Thread(target=server.serve_forever,daemon=True).start()
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path='/usr/bin/chromium')
        context = browser.new_context(viewport={'width':390,'height':844})
        context.add_init_script(TOOLS)
        page = context.new_page()
        errors, failures = [], []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.on('response', lambda response: failures.append(response.url) if response.status>=400 else None)
        page.goto(f'http://127.0.0.1:{server.server_port}', wait_until='networkidle')
        mirror = snapshot(page)
        def picker():
            page.click('#settings-button');page.click('#new-game')
            assert page.locator('#reset-dialog').evaluate('el=>el.open')
        picker()
        assert page.locator('input[value="mirror"]').is_checked()
        page.check('input[value="river"]');page.click('#keep-playing')
        assert snapshot(page)==mirror
        picker();page.check('input[value="river"]');page.click('#confirm-reset')
        assert snapshot(page)['mapId']=='river'
        assert page.locator('#map [data-tile]').count()==135
        assert page.locator('#map .terrain-bridge').count()==1
        assert page.locator('#village-total').inner_text()=='6'
        assert page.locator('#map .terrain-ford').count()==1
        page.click('[data-tile="2,3"]');page.click('[data-tile="5,3"]')
        captured = snapshot(page)
        assert next(v for v in captured['villages'] if v['id']=='5,3')['owner']=='player'
        page.reload(wait_until='networkidle')
        assert snapshot(page)==captured and page.locator('#map [data-tile]').count()==135
        page.evaluate("""() => {const key='hex-command-battle';const saved=JSON.parse(localStorage.getItem(key));saved.state.units.find(u=>u.id==='u2').tile='6,3';localStorage.setItem(key,JSON.stringify(saved));}""")
        page.reload(wait_until='networkidle')
        before = snapshot(page)
        page.click('[data-tile="7,3"]')
        assert next(u for u in snapshot(page)['units'] if u['id']=='u2')['tile']=='7,3'
        assert page.locator('#terrain-panel h2').inner_text()=='Bridge'
        assert 'Movement: 1' in page.locator('#terrain-panel').inner_text()
        for style in ['low-poly','pixel','normal']:
            bridge_state = snapshot(page)
            choose_style(page,style)
            assert snapshot(page)==bridge_state
            assert page.locator('.terrain-bridge .terrain-art').count()==1
        page.click('#undo-move')
        assert snapshot(page)==before
        page.click('[data-tile="8,2"]')
        assert page.locator('#terrain-panel h2').inner_text()=='Water'
        assert snapshot(page)==before
        page.click('[data-tile="5,7"]')
        assert page.locator('#terrain-panel h2').inner_text()=='Shallow water'
        assert 'Movement: 3 for soldiers and commanders; 2 for archers and scouts' in page.locator('#terrain-panel').inner_text()
        assert 'Exposure: +1 damage received' in page.locator('#terrain-panel').inner_text()
        for width in [320,390,1280]:
            page.set_viewport_size({'width':width,'height':900})
            assert page.evaluate('document.documentElement.scrollWidth<=innerWidth')
            assert page.locator('#map').evaluate('el=>Array.from(el.querySelectorAll(".tile-ground")).every(tile=>{const b=tile.getBBox(),v=el.viewBox.baseVal;return b.x>=0&&b.y>=0&&b.x+b.width<=v.width&&b.y+b.height<=v.height})')
            picker()
            assert page.locator('input[value="river"]').is_checked()
            assert page.locator('#reset-dialog').evaluate('el=>el.scrollWidth<=el.clientWidth')
            page.click('#keep-playing')
        page.screenshot(path='/tmp/hex-river-map.png',full_page=True)
        picker();page.check('input[value="mirror"]');page.click('#confirm-reset')
        assert snapshot(page)==mirror and page.locator('#map [data-tile]').count()==77
        page.evaluate("""() => {const key='hex-command-battle';const saved=JSON.parse(localStorage.getItem(key));saved.state.winner='enemy';saved.state.turn='enemy';saved.state.units=saved.state.units.filter(u=>u.id!=='u1');localStorage.setItem(key,JSON.stringify(saved));}""")
        page.reload(wait_until='networkidle');page.click('#play-again')
        assert page.locator('#reset-dialog').evaluate('el=>el.open')
        page.check('input[value="river"]');page.click('#confirm-reset')
        assert snapshot(page)['mapId']=='river' and snapshot(page)['winner'] is None
        page.click('#end-turn')
        page.reload(wait_until='domcontentloaded')
        page.wait_for_function("window.hexSmokeTools?.get_battle_state.execute().round===2",timeout=10000)
        continued = snapshot(page)
        assert continued['mapId']=='river' and continued['turn']=='player'
        page.reload(wait_until='networkidle')
        assert snapshot(page)==continued
        assert not errors and not failures, (errors,failures)
        print('PASS: map choice/cancel/replay, river capture and bridge undo, saves, full geometry and mobile layout')
        browser.close()
finally:
    server.shutdown()
