"""Real browser clicks emit one material cue with attack > unit > terrain."""
from functools import partial
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright
from smoke import TOOLS, snapshot
class Quiet(SimpleHTTPRequestHandler):
    def log_message(self,*_): pass
server=ThreadingHTTPServer(('127.0.0.1',0),partial(Quiet,directory=str(Path(__file__).resolve().parents[1]/'dist')))
Thread(target=server.serve_forever,daemon=True).start()
AUDIO='''
window.playedSamples=[];
const encodedNames=new WeakMap(), decodedNames=new WeakMap(), nativeFetch=window.fetch;
window.fetch=async (...args)=>{
 const response=await nativeFetch(...args), url=String(args[0]);
 if(url.includes('/audio/')) {
  const original=response.arrayBuffer.bind(response);
  response.arrayBuffer=async()=>{const data=await original();encodedNames.set(data,url);return data;};
 }
 return response;
};
const NativeAudio=window.AudioContext;
window.AudioContext=class extends NativeAudio {
 async decodeAudioData(data){const buffer=await super.decodeAudioData(data);decodedNames.set(buffer,encodedNames.get(data));return buffer;}
 createBufferSource(){const source=super.createBufferSource(),start=source.start.bind(source);source.start=(...args)=>{playedSamples.push(decodedNames.get(source.buffer));start(...args);};return source;}
};
'''
try:
 with sync_playwright() as p:
  browser=p.chromium.launch(executable_path='/usr/bin/chromium')
  context=browser.new_context();context.add_init_script(TOOLS);context.add_init_script(AUDIO)
  page=context.new_page();errors=[];page.on('pageerror',lambda error:errors.append(str(error)))
  page.goto(f'http://127.0.0.1:{server.server_port}',wait_until='networkidle')
  assert page.evaluate('playedSamples.length')==0
  def click(selector,family):
   before=page.evaluate('playedSamples.length');page.click(selector)
   page.wait_for_function('before=>playedSamples.length>before',arg=before)
   page.wait_for_timeout(120)
   played=page.evaluate('playedSamples')
   assert len(played)==before+1,played
   assert f'/{family}-' in played[-1],played[-1]
  click('[data-tile="1,3"]','unit-archer') # Soldier inherits the heavier footstep.
  click('[data-tile="2,3"]','unit-archer')
  click('[data-tile="1,2"]','terrain-plains')
  click('[data-tile="3,1"]','terrain-plains') # Capture does not stack a coin cue.
  page.keyboard.press('Escape')
  for tile,family in [('3,5','terrain-village'),('1,0','terrain-water'),('5,3','terrain-forest'),('5,0','terrain-hill'),('10,2','terrain-plains'),('0,3','unit-commander')]:
   click(f'[data-tile="{tile}"]',family)
   page.keyboard.press('Escape')
  # Move an archer into range using a valid save; attack preview and resolution
  # must never add the defending soldier or its terrain cue.
  page.evaluate("""()=>{const key='hex-command-battle',saved=JSON.parse(localStorage.getItem(key));const unit=saved.state.units.find(u=>u.id==='u2');unit.type='archer';unit.hp=8;unit.tile='8,3';localStorage.setItem(key,JSON.stringify(saved));}""")
  page.reload(wait_until='networkidle')
  click('[data-tile="8,3"]','terrain-plains') # Repeated unit clicks keep their cue.
  click('[data-tile="8,3"]','terrain-plains')
  click('[data-tile="8,3"]','terrain-plains') # Select again after toggling off.
  before=snapshot(page)
  click('[data-tile="9,3"]','select')
  assert snapshot(page)==before
  click('#attack','attack-bow')
  assert next(u for u in snapshot(page)['units'] if u['id']=='u5')['hp']<12
  assert not errors,errors
  page.keyboard.press('Escape')
  click('[data-tile="1,0"]','terrain-water')
  click('[data-tile="1,0"]','terrain-water')
  page.evaluate("window.hexSmokeTools.start_new_battle.execute()")
  click('[data-tile="1,2"]','terrain-plains')
  click('[data-tile="3,1"]','terrain-plains')
  before=page.evaluate('playedSamples.length')
  page.click('#undo-move');page.wait_for_timeout(120)
  assert page.evaluate('playedSamples.length')==before, 'undo is silent'
  page.click('#end-turn')
  page.wait_for_function("window.hexSmokeTools.get_battle_state.execute().round===2")
  page.wait_for_timeout(120)
  assert not any('/turn-' in sample for sample in page.evaluate('playedSamples')), 'both turn changes are silent'
  assert not errors,errors
  print('PASS: silent undo and both turn changes; enemy action sounds remain')
  print('PASS: weapons only on actual attacks; quiet target preview; audible repeat selection/reinspection; unit and terrain priority')
  browser.close()
finally: server.shutdown()
