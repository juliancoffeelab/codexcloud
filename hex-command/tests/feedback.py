"""Feedback preferences, gesture-only audio, and reduced-motion integration."""
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
AUDIO = '''
window.audioStarts = 0; window.audioNotes = 0;
const NativeAudio = window.AudioContext;
if (NativeAudio) window.AudioContext = class extends NativeAudio {
  constructor(...args) { super(...args); window.audioStarts++; }
  createOscillator() { window.audioNotes++; return super.createOscillator(); }
};
'''
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path='/usr/bin/chromium')
        context = browser.new_context()
        context.add_init_script(TOOLS); context.add_init_script(AUDIO)
        page = context.new_page(); errors = []
        page.on('pageerror', lambda e: errors.append(str(e)))
        url = f'http://127.0.0.1:{server.server_port}/'
        page.goto(url, wait_until='networkidle')
        assert page.evaluate('audioStarts') == 0
        initial = snapshot(page)
        page.click('[data-tile="1,2"]')
        page.wait_for_function('audioNotes > 0')
        assert snapshot(page) == initial
        page.click('[data-tile="3,1"]')
        assert page.locator('[data-tile="3,1"].motion-capture').count() == 1
        page.click('#settings-button'); page.uncheck('#sound-effects'); page.click('#close-settings')
        notes = page.evaluate('audioNotes')
        page.click('[data-tile="10,0"]')
        assert page.evaluate('audioNotes') == notes
        page.reload(wait_until='networkidle')
        assert page.evaluate('audioStarts') == 0
        page.click('#settings-button')
        assert not page.is_checked('#sound-effects')
        assert page.evaluate('audioStarts') == 0
        page.check('#sound-effects'); page.wait_for_function('audioNotes > 0')
        assert page.evaluate('audioStarts') == 1
        assert not errors, errors
        context.close()
        context = browser.new_context(reduced_motion='reduce')
        context.add_init_script(TOOLS)
        context.add_init_script("localStorage.setItem('hex-command-sound-enabled', 'false')")
        page = context.new_page(); page.goto(url, wait_until='networkidle')
        page.click('[data-tile="1,2"]'); page.click('[data-tile="3,1"]')
        assert page.locator('#map [class*="motion-"]').count() == 0
        assert next(u for u in snapshot(page)['units'] if u['id'] == 'u3')['tile'] == '3,1'
        context.close(); browser.close()
        print('PASS: no autoplay, gesture audio, mute persistence, reenabling, reduced motion, and unchanged actions')
finally:
    server.shutdown(); server.server_close()
