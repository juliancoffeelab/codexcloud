"""Weapon choice, deterministic forecasts, matching responses and mobile combat."""
from functools import partial
from http.server import ThreadingHTTPServer
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright
from smoke import QuietHandler, TOOLS, snapshot
root=Path(__file__).resolve().parents[1]/'dist'
server=ThreadingHTTPServer(('127.0.0.1',0),partial(QuietHandler,directory=str(root)))
Thread(target=server.serve_forever,daemon=True).start()
try:
 with sync_playwright() as p:
  browser=p.chromium.launch(executable_path='/usr/bin/chromium')
  context=browser.new_context();context.add_init_script(TOOLS)
  page=context.new_page();errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
  page.goto(f'http://127.0.0.1:{server.server_port}',wait_until='networkidle')
  def setup(attacker='soldier',defender='soldier',target_tile='6,1'):
   page.evaluate("window.hexSmokeTools.start_new_battle.execute()")
   page.evaluate('''([a,b,t])=>{
    const key='hex-command-battle',s=JSON.parse(localStorage.getItem(key));
    s.state.units=s.state.units.filter(u=>['u1','u2','u4','u5'].includes(u.id));
    for(const [id,type,tile] of [['u2',a,'5,1'],['u5',b,t]]){
     const u=s.state.units.find(u=>u.id===id);u.type=type;u.tile=tile;u.hp=type==='soldier'?12:8;u.moved=false;u.attacked=false;
    }
    localStorage.setItem(key,JSON.stringify(s));
   }''',[attacker,defender,target_tile])
   page.reload(wait_until='networkidle')
   page.click('[data-tile="5,1"]');page.click(f'[data-tile="{target_tile}"]')
  def hp(id):return next(u['hp'] for u in snapshot(page)['units'] if u['id']==id)
  setup()
  assert page.locator('input[name="attack-choice"]').count()==2
  assert page.locator('input[value="mace"]').is_checked()
  assert 'Deal 4 · take 4' in page.locator('.attack-choice.chosen').inner_text()
  page.locator('.defender-resistances summary').click()
  assert 'Impact: 25% more' in page.locator('.defender-resistances').inner_text()
  before=snapshot(page)
  page.check('input[value="sword"]')
  assert 'Deal 3 · take 4' in page.locator('.attack-choice.chosen').inner_text()
  assert '25% blade resistance' in page.locator('.combat-explanation').inner_text()
  assert snapshot(page)==before,'choosing a weapon is not an action'
  page.check('input[value="mace"]');page.click('#attack')
  assert hp('u2')==8 and hp('u5')==8
  page.reload(wait_until='networkidle');assert hp('u2')==8 and hp('u5')==8
  assert not page.locator('#unit-panel').is_visible()
  setup('archer','archer')
  assert 'Deal 3 · take 3' in page.locator('.attack-choice.chosen').inner_text()
  page.check('input[value="knife"]')
  assert 'Deal 2 · take 2' in page.locator('.attack-choice.chosen').inner_text()
  page.keyboard.press('ArrowLeft')
  assert page.locator('input[value="bow"]').is_checked(),'native radio keyboard choice works'
  page.click('#attack');assert hp('u2')==5 and hp('u5')==5
  setup('archer','soldier','7,1')
  assert page.locator('input[value="knife"]').is_disabled()
  assert 'No ranged retaliation' in page.locator('.combat-explanation').inner_text()
  for width in [1280,390,320]:
   page.set_viewport_size({'width':width,'height':900})
   assert page.evaluate('document.documentElement.scrollWidth<=innerWidth'),'no page overflow'
   assert page.locator('.attack-choice').first.bounding_box()['height']>=44
   path=f'/tmp/hex-combat-{width}.png';page.screenshot(path=path,full_page=True)
  assert not errors,errors
  print('PASS: weapon choice, armor explanations, exact HP forecasts, ranged replies, keyboard, saves and 320px combat')
  browser.close()
finally:server.shutdown()
