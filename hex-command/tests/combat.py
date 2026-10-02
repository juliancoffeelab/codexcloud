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
  context=browser.new_context(viewport={'width':393,'height':800},is_mobile=True,has_touch=True);context.add_init_script(TOOLS)
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
  def forecast(attack, player_after, enemy_after):
   assert page.locator('#unit-panel').get_attribute('data-preview-attack') == attack
   for side, after, maximum in [('player',player_after,12 if hp('u2')==12 else 8),('enemy',enemy_after,12 if hp('u5')==12 else 8)]:
    card=page.locator(f'.duel .combatant.{side}')
    assert card.locator('.health-numbers [aria-label="health after attack"] b').inner_text()==str(after)
    width=card.locator('.health-left').evaluate("e=>parseFloat(e.style.width)")
    assert abs(width-after/maximum*100)<0.001
    assert card.locator('.health-lost').count()==1
   assert page.locator(f'[data-attack="{attack}"].previewing').count()==1
  def no_dots():
   assert not any(dot in page.locator('body').inner_text() for dot in ['\u00b7','\u2022']), 'no text dot separators'
   assert page.locator('.terrain-facts').evaluate('e=>getComputedStyle(e).listStyleType')=='none', 'no bullet markers'
  setup()
  assert page.locator('[data-attack]').count()==2
  assert page.locator('input[name="attack-choice"], #attack').count()==0
  assert page.locator('[data-attack="mace"] .attack-outcome').inner_text() == 'Deal 4\nTake 4'
  assert page.locator('[data-attack="sword"] .attack-outcome').inner_text() == 'Deal 3\nTake 4'
  assert not page.locator('#attack-details-sword').is_visible()
  before=snapshot(page)
  forecast('mace',8,8)
  no_dots()
  page.locator('[data-attack="sword"]').hover()
  forecast('sword',8,9)
  assert snapshot(page)==before, 'hovering a forecast is read-only'
  page.tap('[data-inspect="sword"]')
  assert page.locator('[data-inspect="sword"]').get_attribute('aria-expanded')=='true'
  assert '25% blade resistance' in page.locator('#attack-details-sword').inner_text()
  assert 'Impact: 25% more' in page.locator('#attack-details-sword').inner_text()
  assert snapshot(page)==before,'inspecting a calculation is not an action'
  forecast('sword',8,9)
  no_dots()
  page.tap('[data-inspect="sword"]')
  assert not page.locator('#attack-details-sword').is_visible()
  page.locator('[data-attack="mace"]').click()
  assert hp('u2')==8 and hp('u5')==8,'one weapon click executes the attack'
  page.reload(wait_until='networkidle');assert hp('u2')==8 and hp('u5')==8
  assert not page.locator('#unit-panel').is_visible()
  setup('archer','archer')
  assert page.locator('[data-attack="bow"] .attack-outcome').inner_text() == 'Deal 3\nTake 3'
  assert page.locator('[data-attack="knife"] .attack-outcome').inner_text() == 'Deal 2\nTake 2'
  page.locator('[data-attack="knife"]').focus()
  forecast('knife',6,6)
  page.keyboard.press('Enter')
  assert hp('u2')==6 and hp('u5')==6,'keyboard activation attacks immediately'
  setup('archer','soldier','7,1')
  assert page.locator('[data-attack="knife"]').is_disabled()
  page.locator('[data-inspect="bow"]').click()
  assert 'No ranged retaliation' in page.locator('#attack-details-bow').inner_text()
  page.locator('[data-inspect="bow"]').click()
  for width in [1280,390,320]:
   page.set_viewport_size({'width':width,'height':900})
   assert page.evaluate('document.documentElement.scrollWidth<=innerWidth'),'no page overflow'
   row=page.locator('.attack-action-row').first
   assert row.bounding_box()['height']==44,'each action stays a single compact touch row'
   assert row.evaluate('(e)=>e.scrollWidth<=e.clientWidth'),'no action row overflow'
   assert page.locator('.combat-explanation:visible').count()==0
   forecast('bow',8,10)
   no_dots()
   path=f'/tmp/hex-combat-{width}.png';page.screenshot(path=path,full_page=True)
  page.locator('[data-attack="bow"]').click();assert hp('u2')==8 and hp('u5')==10
  assert not errors,errors
  print('PASS: one-tap attacks, optional damage inspection, exact outcomes, ranged replies, keyboard, saves and restored health forecasts, no dot separators and compact 320px combat')
  browser.close()
finally:server.shutdown()
