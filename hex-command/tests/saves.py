"""Persistence and interrupted enemy-turn browser checks."""
from functools import partial
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright
from smoke import TOOLS, snapshot, new_battle

class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *_): pass

root = Path(__file__).resolve().parents[1] / 'dist'
server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Quiet, directory=str(root)))
Thread(target=server.serve_forever, daemon=True).start()
url = f'http://127.0.0.1:{server.server_port}/'
key = 'hex-command-battle'
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path='/usr/bin/chromium')
        context = browser.new_context()
        context.add_init_script(TOOLS)
        page = context.new_page(); page.goto(url, wait_until='networkidle')
        initial = snapshot(page)
        page.click('[data-tile="1,2"]'); page.click('[data-tile="3,1"]')
        moved = snapshot(page)
        page.reload(wait_until='networkidle')
        assert snapshot(page) == moved
        assert next(u for u in moved['units'] if u['id']=='u3')['moved']
        print('PASS: player movement, ownership, and action flags survive reload')
        page.click('#end-turn')
        checkpoint = page.evaluate('(key) => localStorage.getItem(key)', key)
        assert page.evaluate('(key) => JSON.parse(localStorage.getItem(key)).pendingEnemy', key)
        page.wait_for_timeout(750)
        assert page.evaluate('(key) => localStorage.getItem(key)', key) == checkpoint
        page.reload(wait_until='domcontentloaded')
        page.wait_for_function('window.hexSmokeTools?.get_battle_state.execute().round === 2')
        replayed = snapshot(page)
        assert not page.evaluate('(key) => JSON.parse(localStorage.getItem(key)).pendingEnemy', key)
        page.reload(wait_until='networkidle')
        assert snapshot(page) == replayed
        # Replay the same checkpoint in another browser context and compare the
        # resulting state, IDs, and log (including income and recruitment).
        other = browser.new_context(); other.add_init_script(TOOLS)
        other.add_init_script('localStorage.setItem('+repr(key)+','+repr(checkpoint)+')')
        twin = other.new_page(); twin.goto(url, wait_until='domcontentloaded')
        twin.wait_for_function('window.hexSmokeTools?.get_battle_state.execute().round === 2')
        assert snapshot(twin) == replayed
        assert page.evaluate('(key) => JSON.parse(localStorage.getItem(key)).state', key) == twin.evaluate('(key) => JSON.parse(localStorage.getItem(key)).state', key)
        other.close()
        print('PASS: no intermediate enemy saves; interruption replays deterministically')
        new_battle(page); page.click('[data-tile="0,2"]')
        page.click('#terrain-panel [data-recruit="archer"]')
        recruited = snapshot(page); page.reload(wait_until='networkidle')
        assert snapshot(page) == recruited
        new_battle(page); page.reload(wait_until='networkidle')
        assert snapshot(page) == initial
        print('PASS: recruitment and New battle persist')
        for invalid in ('{"version":999}', '{broken', '{"version":1,"pendingEnemy":false,"state":{}}'):
            page.evaluate('([key,value]) => localStorage.setItem(key,value)', [key, invalid])
            page.reload(wait_until='networkidle')
            assert snapshot(page) == initial
        print('PASS: unknown versions and invalid saves silently start fresh')
        # A terminal result must also survive refresh.
        page.evaluate('key => {const s=JSON.parse(localStorage.getItem(key)); s.state.winner="enemy"; s.state.turn="enemy"; s.state.units=s.state.units.filter(u=>!(u.type==="commander" && u.side==="player")); localStorage.setItem(key,JSON.stringify(s));}',key)
        page.reload(wait_until='networkidle')
        assert snapshot(page)['winner'] == 'enemy'
        assert page.locator('#play-again').count() == 1
        print('PASS: completed battles restore')
        browser.close()
finally:
    server.shutdown()
