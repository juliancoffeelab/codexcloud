"""Move-only undo through browser controls, action barriers and autosave."""
from functools import partial
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright
from smoke import TOOLS, snapshot

class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *_): pass

server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Quiet, directory=str(Path(__file__).resolve().parents[1] / 'dist')))
Thread(target=server.serve_forever, daemon=True).start()
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path='/usr/bin/chromium')
        context = browser.new_context(viewport={'width': 390, 'height': 844})
        context.add_init_script(TOOLS)
        page = context.new_page()
        errors = []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.goto(f'http://127.0.0.1:{server.server_port}', wait_until='networkidle')
        def act(name, **args):
            return page.evaluate('([name,args]) => window.hexSmokeTools[name].execute(args)', [name,args])
        button = page.locator('#undo-move')
        initial = snapshot(page)
        assert button.is_disabled()
        page.click('[data-tile="1,2"]')
        page.click('[data-tile="3,1"]')
        assert button.is_enabled()
        assert next(v for v in snapshot(page)['villages'] if v['id']=='3,1')['owner']=='player'
        button.click()
        assert snapshot(page)==initial and button.is_disabled()
        assert page.locator('.hex.selected').get_attribute('data-tile')=='1,2'
        assert page.locator('.hex.reachable').count()>0
        page.reload(wait_until='networkidle')
        assert snapshot(page)==initial and button.is_disabled()
        act('move_unit', unitId='u2', tileId='2,3')
        first = snapshot(page)
        act('move_unit', unitId='u3', tileId='3,1')
        button.click()
        assert snapshot(page)==first and button.is_enabled()
        button.click()
        assert snapshot(page)==initial and button.is_disabled()
        act('move_unit', unitId='u3', tileId='3,1')
        act('recruit_unit', commanderId='u1', type='soldier', tileId='0,2')
        after = snapshot(page)
        assert button.is_disabled()
        act('undo_move')
        assert snapshot(page)==after
        act('start_new_battle')
        page.evaluate("""() => {const key='hex-command-battle';const saved=JSON.parse(localStorage.getItem(key));saved.state.units.find(u=>u.id==='u2').tile='8,3';localStorage.setItem(key,JSON.stringify(saved));}""")
        page.reload(wait_until='networkidle')
        act('move_unit', unitId='u3', tileId='3,1')
        act('attack_unit', unitId='u2', targetId='u5')
        after = snapshot(page)
        assert button.is_disabled()
        act('undo_move')
        assert snapshot(page)==after
        act('start_new_battle')
        act('move_unit', unitId='u3', tileId='3,1')
        page.click('#end-turn')
        assert button.is_disabled()
        page.wait_for_function("window.hexSmokeTools.get_battle_state.execute().turn==='player'", timeout=10000)
        assert button.is_disabled()
        act('move_unit', unitId='u2', tileId='2,3')
        assert button.is_enabled()
        act('start_new_battle')
        assert snapshot(page)==initial and button.is_disabled()
        assert errors==[], errors
        print('PASS: move button, capture rollback, LIFO, autosave and recruitment/combat/turn/reset barriers')
        browser.close()
finally:
    server.shutdown()
