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
  def setup(attacker='soldier',defender='soldier',target_tile='6,1',attacker_tile='5,1'):
   page.evaluate("window.hexSmokeTools.start_new_battle.execute()")
   page.evaluate('''([a,b,t,origin])=>{
    const key='hex-command-battle',s=JSON.parse(localStorage.getItem(key));
    s.state.units=s.state.units.filter(u=>['u1','u2','u4','u5'].includes(u.id));
    for(const [id,type,tile] of [['u2',a,origin],['u5',b,t]]){
     const u=s.state.units.find(u=>u.id===id);u.type=type;u.tile=tile;u.hp=type==='soldier'?12:8;u.moved=false;u.attacked=false;
    }
    localStorage.setItem(key,JSON.stringify(s));
   }''',[attacker,defender,target_tile,attacker_tile])
   page.reload(wait_until='networkidle')
   page.click(f'[data-tile="{attacker_tile}"]');page.click(f'[data-tile="{target_tile}"]')
  def hp(id):return next(u['hp'] for u in snapshot(page)['units'] if u['id']==id)
  def forecast(attack, player_after, enemy_after):
   row=page.locator(f'[data-attack="{attack}"]')
   for side, after, maximum in [('player',player_after,12 if hp('u2')==12 else 8),('enemy',enemy_after,12 if hp('u5')==12 else 8)]:
    bar=row.locator(f'[data-forecast="{side}"]')
    assert bar.locator('.forecast-label b').inner_text()==str(after)
    assert 'You' not in bar.locator('.forecast-label').inner_text() and 'Enemy' not in bar.locator('.forecast-label').inner_text()
    width=bar.locator('.health-left').evaluate("e=>parseFloat(e.style.width)")
    assert abs(width-after/maximum*100)<0.001
    assert bar.locator('.health-lost').count()==1
   assert abs(row.locator('[data-forecast="player"] .health-track').bounding_box()['y']-row.locator('[data-forecast="enemy"] .health-track').bounding_box()['y'])<1, 'attack bars align'
   assert row.locator("..").evaluate("e=>getComputedStyle(e).borderTopStyle")=='solid'
  def no_dots():
   assert not any(dot in page.locator('body').inner_text() for dot in ['\u00b7','\u2022']), 'no text dot separators'
   assert page.locator('.terrain-facts').evaluate('e=>getComputedStyle(e).listStyleType')=='none', 'no bullet markers'
  setup()
  assert page.locator('[data-attack]').count()==2
  assert page.locator('input[name="attack-choice"], #attack').count()==0
  assert page.locator('[data-attack="mace"] [data-forecast="enemy"] .forecast-impact').inner_text() == 'Deal −4 HP'
  assert page.locator('[data-attack="mace"] [data-forecast="player"] .forecast-impact').inner_text() == 'Take −4 HP'
  assert page.locator('[data-attack="sword"] [data-forecast="enemy"] .forecast-impact').inner_text() == 'Deal −3 HP'
  assert page.locator('[data-attack="sword"] [data-forecast="player"] .forecast-impact').inner_text() == 'Take −4 HP'
  assert not page.locator('#attack-details-sword').is_visible()
  before=snapshot(page)
  forecast('mace',8,8)
  no_dots()
  page.locator('[data-attack="sword"]').hover()
  forecast('sword',8,9)
  assert snapshot(page)==before, 'hovering a forecast is read-only'
  page.tap('[data-inspect="sword"]')
  assert page.locator('[data-inspect="sword"]').get_attribute('aria-expanded')=='true'
  assert 'Your attack: 4 blade, sword, melee' in page.locator('#attack-details-sword').inner_text()
  assert 'Their attack: 3 impact, mace, melee' in page.locator('#attack-details-sword').inner_text()
  assert 'round(4 × 75%) = 3' in page.locator('#attack-details-sword').inner_text()
  assert 'round(3 × 125%) = 4' in page.locator('#attack-details-sword').inner_text()
  assert page.locator('.attack-actions .resistances').count()==0, 'unit stats do not repeat in attack calculations'
  assert snapshot(page)==before,'inspecting a calculation is not an action'
  forecast('sword',8,9)
  no_dots()
  page.tap('[data-inspect="mace"]')
  page.locator('#unit-panel').screenshot(path='/tmp/hex-attack-equations.png')
  page.tap('[data-inspect="mace"]')
  page.tap('[data-inspect="sword"]')
  assert not page.locator('#attack-details-sword').is_visible()
  page.locator('[data-attack="mace"]').click()
  assert hp('u2')==8 and hp('u5')==8,'one weapon click executes the attack'
  page.reload(wait_until='networkidle');assert hp('u2')==8 and hp('u5')==8
  assert not page.locator('#unit-panel').is_visible()
  setup('archer','archer')
  assert page.locator('[data-attack="bow"] [data-forecast="enemy"] .forecast-impact').inner_text() == 'Deal −3 HP'
  assert page.locator('[data-attack="bow"] [data-forecast="player"] .forecast-impact').inner_text() == 'Take −3 HP'
  assert page.locator('[data-attack="knife"] [data-forecast="enemy"] .forecast-impact').inner_text() == 'Deal −2 HP'
  assert page.locator('[data-attack="knife"] [data-forecast="player"] .forecast-impact').inner_text() == 'Take −2 HP'
  page.locator('[data-attack="knife"]').focus()
  forecast('knife',6,6)
  page.keyboard.press('Enter')
  assert hp('u2')==6 and hp('u5')==6,'keyboard activation attacks immediately'
  setup('archer','soldier','7,1')
  assert page.locator('[data-attack="knife"]').is_disabled()
  page.locator('[data-inspect="bow"]').click()
  assert '0 (out of range)' in page.locator('#attack-details-bow').inner_text()
  assert 'Your attack: 3 pierce, bow, ranged' in page.locator('#attack-details-bow').inner_text()
  assert 'Their attack: none' in page.locator('#attack-details-bow').inner_text()
  page.locator('[data-inspect="bow"]').click()
  for width in [1280,390,320]:
   page.set_viewport_size({'width':width,'height':900})
   assert page.evaluate('document.documentElement.scrollWidth<=innerWidth'),'no page overflow'
   row=page.locator('.attack-action-row').first
   assert row.bounding_box()['height']>=44,'bordered action with its own forecast remains touch-sized'
   assert row.evaluate('(e)=>e.scrollWidth<=e.clientWidth'),'no action row overflow'
   box=row.locator('.attack-box').bounding_box()
   glass=row.locator('[data-inspect]').bounding_box()
   assert glass['x']>=box['x'] and glass['x']+glass['width']<=box['x']+box['width'], 'glass is inside attack border'
   assert row.locator('.attack-heading .forecast-impact, .attack-heading .attack-outcome').count()==0
   for side in ['player','enemy']:
    bar=row.locator(f'[data-forecast="{side}"]')
    track=bar.locator('.health-track').bounding_box()
    loss=bar.locator('.forecast-impact').bounding_box()
    assert loss['y']>=track['y']+track['height'], 'damage labels sit below the corresponding bar'
   assert page.locator('.combat-explanation:visible').count()==0
   forecast('bow',8,10)
   no_dots()
   path=f'/tmp/hex-combat-{width}.png';page.screenshot(path=path,full_page=True)
  page.locator('[data-attack="bow"]').click();assert hp('u2')==8 and hp('u5')==10
  setup('archer','archer','3,2','5,2')
  before=snapshot(page)
  page.tap('[data-inspect="bow"]')
  detail=page.locator('#attack-details-bow').inner_text()
  assert 'Your attack: 3 pierce, bow, ranged, against terrain: hills' in detail
  assert 'Their attack: 3 pierce, bow, ranged, against terrain: forest' in detail
  assert detail.count('round(3 × 100%) − 1 (cover) = 2')==2
  assert snapshot(page)==before
  page.locator('[data-attack="bow"]').click()
  assert hp('u2')==6 and hp('u5')==6
  assert not errors,errors
  print('PASS: one-tap attacks, optional damage inspection, exact outcomes, ranged replies, keyboard, saves and independent attack health forecasts, numeric rounded calculations, no dot separators and compact 320px combat')
  browser.close()
finally:server.shutdown()
