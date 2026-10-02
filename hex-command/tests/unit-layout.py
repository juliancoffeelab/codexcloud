"""Mobile unit identity, compact weapon rows, and unchanged battlefield geometry."""
from functools import partial
from http.server import ThreadingHTTPServer
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright
from smoke import QuietHandler, TOOLS, snapshot, choose_style
root=Path(__file__).resolve().parents[1]/'dist'
server=ThreadingHTTPServer(('127.0.0.1',0),partial(QuietHandler,directory=str(root)))
Thread(target=server.serve_forever,daemon=True).start()
try:
 with sync_playwright() as p:
  browser=p.chromium.launch(executable_path='/usr/bin/chromium')
  context=browser.new_context(viewport={'width':393,'height':800},device_scale_factor=1.5,is_mobile=True,has_touch=True)
  context.add_init_script(TOOLS)
  page=context.new_page();errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
  page.goto(f'http://127.0.0.1:{server.server_port}',wait_until='networkidle')
  page.evaluate("window.hexSmokeTools.move_unit.execute({unitId:'u1',tileId:'2,3'})")
  before=snapshot(page)
  for style in ['pixel','normal','low-poly']:
   choose_style(page,style)
   assert snapshot(page)==before
   for width in [393,320,768,1280]:
    page.set_viewport_size({'width':width,'height':800})
    card=page.locator('.unit-detail>.combatant').bounding_box()
    panel=page.locator('#unit-panel').bounding_box()
    info=page.locator('.unit-info').bounding_box()
    assert card['y']-panel['y']<20,'unit identity stays at the top'
    assert card['height']<110,'portrait does not stretch alongside all details'
    assert info['y']>=card['y']+card['height'],'details follow the identity row'
    assert page.evaluate('document.documentElement.scrollWidth<=innerWidth'),'no horizontal page overflow'
    assert page.locator('#map').evaluate('''el=>{
     const r=el.getBoundingClientRect(),v=el.viewBox.baseVal;
     return Math.abs(r.height-r.width*v.height/v.width)<1&&r.height>300;
    }'''),'map keeps its full aspect ratio'
    assert page.locator('.map-scroll').evaluate('el=>el.clientHeight>=el.querySelector("svg").getBoundingClientRect().height-1'),'board is not clipped vertically'
    for row in page.locator('.unit-detail .arsenal p').all():
     assert row.bounding_box()['height']<35,'weapon and reach stay on one row'
    if width==393 and style=='pixel':
     page.locator('#unit-panel').screenshot(path='/tmp/hex-unit-panel-fixed.png')
     page.screenshot(path='/tmp/hex-unit-mobile-fixed.png',full_page=True)
  assert not errors,errors
  print('PASS: top-aligned unit identity, compact weapons, all art styles, unchanged board geometry and no page overflow')
  browser.close()
finally:server.shutdown()
