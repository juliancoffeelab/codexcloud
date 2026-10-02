"""Phone terrain facts retain readable symbols instead of native list bullets."""
from functools import partial
from http.server import ThreadingHTTPServer
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright
from smoke import QuietHandler, TOOLS, snapshot, choose_style

server = ThreadingHTTPServer(('127.0.0.1', 0), partial(
    QuietHandler, directory=str(Path(__file__).resolve().parents[1] / 'dist')))
Thread(target=server.serve_forever, daemon=True).start()
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path='/usr/bin/chromium')
        context = browser.new_context(viewport={'width': 393, 'height': 800},
                                      is_mobile=True, has_touch=True)
        context.add_init_script(TOOLS)
        page = context.new_page()
        errors = []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.goto(f'http://127.0.0.1:{server.server_port}', wait_until='networkidle')
        initial = snapshot(page)
        for width in (320, 393):
            page.set_viewport_size({'width': width, 'height': 800})
            for style in ('pixel', 'normal', 'low-poly'):
                choose_style(page, style)
                for tile in ('0,0', '5,2', '3,2', '3,1', '10,3'):
                    page.locator(f'[data-tile="{tile}"]').dispatch_event('click')
                    facts = page.locator('.terrain-facts')
                    assert facts.evaluate('el => getComputedStyle(el).listStyleType') == 'none'
                    for row in facts.locator('li').all():
                        icon = row.locator('svg.ui-icon')
                        assert icon.count() == 1
                        assert icon.get_attribute('aria-hidden') == 'true'
                        assert icon.evaluate('el => el.getBBox().width > 0 && el.getBBox().height > 0')
                        assert icon.bounding_box()['width'] == 17
                        assert row.inner_text().strip(), 'icons supplement readable facts'
                    assert page.evaluate('document.documentElement.scrollWidth <= innerWidth')
                    assert snapshot(page) == initial, 'inspection remains read-only'
        page.locator('[data-tile="5,2"]').dispatch_event('click')
        page.locator('#terrain-panel').screenshot(path='/tmp/hex-terrain-icons.png')
        assert not errors, errors
        browser.close()
        print('PASS: meaningful terrain SVGs, no list bullets, read-only inspection, all art styles at phone widths')
finally:
    server.shutdown()
    server.server_close()
