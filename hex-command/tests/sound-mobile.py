"""Trusted touch activation renders nonzero PCM, including deferred resume recovery."""
from functools import partial
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright

class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *_): pass

server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Quiet, directory=str(Path(__file__).resolve().parents[1] / 'dist')))
Thread(target=server.serve_forever, daemon=True).start()
AUDIO = r'''
window.audioProbe = {resumes:0, cues:0, peak:0, running:false, trusted:[]};
const NativeAudio = window.AudioContext;
window.AudioContext = class extends NativeAudio {
 constructor(...args) {
  super(...args);
  window.phoneAudio = this;
  if (window.deferFirstResume) super.suspend();
  const analyser = this.createAnalyser();
  analyser.fftSize = 512;
  analyser.connect(this.destination);
  this.meter = analyser;
  const values = new Float32Array(512);
  setInterval(() => {
   analyser.getFloatTimeDomainData(values);
   audioProbe.peak = Math.max(audioProbe.peak, ...values.map(Math.abs));
   audioProbe.running = this.state === 'running';
  }, 4);
 }
 resume() {
  audioProbe.resumes++;
  audioProbe.trusted.push(navigator.userActivation.isActive);
  if (window.deferFirstResume && audioProbe.resumes === 1) return new Promise(() => {});
  return super.resume();
 }
 createGain() {
  const gain = super.createGain(), connect = gain.connect.bind(gain);
  gain.connect = (destination, ...args) => connect(destination === this.destination ? this.meter : destination, ...args);
  return gain;
 }
 createBufferSource() {
  const source = super.createBufferSource(), start = source.start.bind(source);
  source.start = (...args) => {
   if (source.buffer.length > 1) audioProbe.cues++;
   return start(...args);
  };
  return source;
 }
};
'''
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path='/usr/bin/chromium', args=['--autoplay-policy=document-user-activation-required'])
        for deferred in [False, True]:
            context = browser.new_context(viewport={'width':393, 'height':800}, has_touch=True, is_mobile=True)
            context.add_init_script(f'window.deferFirstResume = {str(deferred).lower()};\n' + AUDIO)
            page = context.new_page()
            errors = []
            page.on('pageerror', lambda error: errors.append(str(error)))
            page.goto(f'http://127.0.0.1:{server.server_port}', wait_until='networkidle')
            assert page.evaluate('audioProbe.cues') == 0, 'no playback or context before a gesture'
            assert page.evaluate('typeof phoneAudio') == 'undefined'
            page.tap('[data-tile="1,3"]')
            page.wait_for_function('audioProbe.cues === 1 && audioProbe.running && audioProbe.peak > 0.001')
            if deferred:
                assert page.evaluate('audioProbe.resumes') >= 2, 'a completed tap retries the pending touch-start resume'
                assert page.evaluate('audioProbe.trusted.every(Boolean)'), 'resume calls retain trusted gesture activation'
            # A suspended background context stays silent until the next gesture.
            page.evaluate('phoneAudio.suspend()')
            before = page.evaluate('audioProbe.cues')
            page.tap('[data-tile="1,2"]')
            page.wait_for_function('before => audioProbe.running && audioProbe.cues === before + 1', arg=before)
            assert not errors, errors
            result = page.evaluate('audioProbe')
            print(f"PASS: {'deferred' if deferred else 'normal'} Android-style touch, recovered suspension, rendered peak {result['peak']:.4f}, no autoplay")
            context.close()
        browser.close()
finally:
    server.shutdown()
