"""Selection dismissal without changing battle state or blocking actions."""
from functools import partial
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright
from smoke import TOOLS, snapshot, new_battle
class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *_): pass
server=ThreadingHTTPServer(('127.0.0.1',0),partial(Quiet,directory=str(Path(__file__).resolve().parents[1]/'dist')))
Thread(target=server.serve_forever,daemon=True).start()
try:
 with sync_playwright() as p:
  browser=p.chromium.launch(executable_path='/usr/bin/chromium')
  context=browser.new_context(viewport={'width':390,'height':844});context.add_init_script(TOOLS)
  page=context.new_page();page.goto(f'http://127.0.0.1:{server.server_port}',wait_until='networkidle')
  initial=snapshot(page)
  def cleared():
   assert page.locator('.hex.selected,.hex.reachable,.hex.target,.hex.recruitable').count()==0
   assert not page.locator('#unit-panel').is_visible()
  cleared();assert page.locator('#map [tabindex="0"]').count()==1
  page.click('[data-tile="1,3"]');retile=page.locator('.hex.selected').get_attribute('data-tile');assert retile=='1,3'
  page.click('[data-tile="1,3"]');cleared();assert snapshot(page)==initial
  page.click('[data-tile="1,2"]');assert page.locator('.hex.selected').get_attribute('data-tile')=='1,2'
  page.click('[data-tile="10,0"]');cleared();assert snapshot(page)==initial
  def retained(tile):
   assert page.locator('.hex.selected').get_attribute('data-tile')==tile
   assert page.locator('#unit-panel').is_visible()
  page.click('[data-tile="1,3"]')
  page.locator('#map').click(position={'x':10,'y':page.locator('#map').bounding_box()['height']-5});retained('1,3')
  page.click('.header-economy');retained('1,3')
  page.click('#unit-panel h2');retained('1,3');assert snapshot(page)==initial
  page.keyboard.press('Escape');cleared()
  page.click('[data-tile="0,3"]');page.click('[data-recruit="archer"]')
  recruitable=page.locator('.hex.recruitable').count();assert recruitable>0
  page.click('.header-economy');retained('0,3')
  assert page.locator('.hex.recruitable').count()==recruitable
  assert page.locator('#cancel').is_visible()
  page.click('[data-tile="10,0"]');cleared();assert not page.locator('#cancel').is_visible()
  new_battle(page);cleared();assert page.locator('#map [tabindex="0"]').count()==1
  page.locator('#map [tabindex="0"]').focus();page.keyboard.press('Enter')
  assert page.locator('.hex.selected').count()==1
  page.click('#end-turn');page.wait_for_function("window.hexSmokeTools.get_battle_state.execute().round === 2")
  cleared();assert page.locator('#map [tabindex="0"]').count()==1
  new_battle(page);cleared()
  print('PASS: no automatic selection on load/reset/player turn; keyboard focus; tile dismissal; selection retained on map background, header and panel; Escape; recruitment cancellation')
  # Place a soldier in valid attack range without changing the game rules.
  page.evaluate('''() => {const key='hex-command-battle';const saved=JSON.parse(localStorage.getItem(key)); saved.state.units.find(u=>u.id==='u2').tile='8,3';localStorage.setItem(key,JSON.stringify(saved));}''')
  page.reload(wait_until='networkidle');cleared();assert page.locator('#map [tabindex="0"]').count()==1;before=snapshot(page)
  page.click('[data-tile="8,3"]')
  page.click('[data-tile="9,3"]');assert page.locator('.duel').is_visible()
  duel_text=page.locator('.duel').inner_text()
  page.click('.header-economy');assert page.locator('.duel').is_visible()
  assert page.locator('.duel').inner_text()==duel_text;assert snapshot(page)==before
  page.click('[data-tile="9,3"]');cleared();assert snapshot(page)==before
  page.click('[data-tile="8,3"]');page.click('[data-tile="9,3"]');page.click('#attack')
  assert next(u for u in snapshot(page)['units'] if u['id']=='u5')['hp']<12
  print('PASS: target toggle and combat button remain functional')
  browser.close()
finally: server.shutdown()
