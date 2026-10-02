"""Exact browser rendering and public battle-state parity across a UI migration.

Capture the immutable baseline first:
  python tests/parity.py --capture /workspace/hex-command-baseline/dist --output /tmp/hex-parity/baseline
Then compare a production build:
  python tests/parity.py --capture dist --output /tmp/hex-parity/current --compare /tmp/hex-parity/baseline
Both sides use identical browser configuration; no image-difference tolerance.
"""

import argparse
import json
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import shutil
import tempfile
from threading import Thread

import numpy as np
from PIL import Image
from playwright.sync_api import sync_playwright


TOOLS = """Object.defineProperty(document, 'modelContext', {value: {
registerTool(tool) {window.hexSmokeTools ??= {}; window.hexSmokeTools[tool.name] = tool;}
}});"""
VIEWPORTS = ((1280, 900), (390, 844), (320, 844))


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_):
        pass


def snapshot(page):
    return page.evaluate("window.hexSmokeTools.get_battle_state.execute()")


def tool(page, name, payload=None):
    return page.evaluate("([name,input]) => window.hexSmokeTools[name].execute(input)",
                         [name, payload or {}])


def ready(page):
    page.wait_for_function("window.hexSmokeTools?.get_battle_state && document.querySelectorAll('#map [data-tile]').length > 0")
    page.wait_for_function("[...document.images].every(image => image.complete)")
    page.evaluate("() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))")


def style(page, name):
    page.click("#settings-button")
    page.select_option("#art-style", name)
    page.wait_for_function("name => document.documentElement.dataset.artStyle === name && !document.querySelector('#art-style').disabled", arg=name)
    page.click("#close-settings")
    ready(page)


def fresh(page, map_id="mirror"):
    tool(page, "start_new_battle", {"mapId": map_id})
    ready(page)


def fixture(page, kind):
    fresh(page)
    page.evaluate("""kind => {
      const key='hex-command-battle', saved=JSON.parse(localStorage.getItem(key));
      saved.state.units=saved.state.units.filter(u=>['u1','u2','u4','u5'].includes(u.id));
      const attacker=saved.state.units.find(u=>u.id==='u2');
      const defender=saved.state.units.find(u=>u.id==='u5');
      attacker.tile='5,1'; defender.tile=kind==='ranged'?'7,1':'6,1';
      if(kind==='ranged'){attacker.type='archer';attacker.hp=8;}
      if(kind==='victory'){
        saved.state.winner='player';
        saved.state.units=saved.state.units.filter(u=>u.id!=='u4');
        saved.state.log=['Enemy commander defeated. You win!'];
      }
      localStorage.setItem(key,JSON.stringify(saved));
    }""", kind)
    page.reload(wait_until="networkidle")
    ready(page)


def capture(directory, output):
    output.mkdir(parents=True, exist_ok=True)
    # Other agents can rebuild dist while verification runs. Serve a fixed copy
    # so every navigation uses the same index, hashed bundle and assets.
    staging = tempfile.TemporaryDirectory(prefix="hex-command-capture-")
    site = Path(staging.name) / "site"
    shutil.copytree(directory.resolve(), site)
    if not (site / "index.html").is_file():
        staging.cleanup()
        raise FileNotFoundError(f"Capture input has no index.html: {directory}")
    server = ThreadingHTTPServer(("127.0.0.1", 0), partial(QuietHandler, directory=str(site)))
    Thread(target=server.serve_forever, daemon=True).start()
    manifest = {}
    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=shutil.which("chromium"),
                                                args=["--disable-gpu", "--force-color-profile=srgb",
                                                      "--disable-features=PartialRaster",
                                                      "--disable-partial-raster",
                                                      "--disable-gpu-rasterization",
                                                      "--disable-threaded-scrolling",
                                                      "--run-all-compositor-stages-before-draw"])
            for width, height in VIEWPORTS:
                context = browser.new_context(viewport={"width": width, "height": height}, reduced_motion="reduce", device_scale_factor=1)
                context.add_init_script(TOOLS)
                page = context.new_page()
                errors, failures = [], []
                page.on("pageerror", lambda error: errors.append(str(error)))
                page.on("response", lambda response: failures.append(f"{response.status} {response.url}") if response.status >= 400 else None)
                page.goto(f"http://127.0.0.1:{server.server_port}/", wait_until="networkidle")
                ready(page)

                def shot(name):
                    ready(page)
                    page.evaluate("window.scrollTo(0,0)")
                    ready(page)
                    key = f"{width}-{name}"
                    # Warm the full-page compositor after viewport scrolling.
                    page.screenshot(full_page=True, animations="disabled", caret="hide")
                    page.screenshot(path=str(output / f"{key}.png"), full_page=True, animations="disabled", caret="hide")
                    manifest[key] = snapshot(page)

                shot("initial-normal")
                for name in ("low-poly", "pixel"):
                    style(page, name)
                    shot(f"initial-{name}")
                style(page, "normal")
                page.click('[data-tile="1,3"]')
                shot("commander")
                page.click('[data-tile="1,2"]')
                shot("scout")
                page.click('[data-tile="3,1"]')
                shot("capture")
                page.click("#undo-move")
                shot("undo")
                fresh(page)
                page.click('[data-tile="0,2"]')
                shot("recruit-options")
                page.click('[data-recruit="soldier"]')
                shot("recruited-soldier")
                page.click("#settings-button")
                shot("settings")
                page.locator("details.panel summary").click()
                shot("manual")
                page.locator("details.panel summary").click()
                page.locator("details.dispatch summary").click()
                shot("journal")
                page.click("#new-game")
                shot("map-picker")
                page.click("#keep-playing")
                if page.locator("#settings-dialog").evaluate("el => el.open"):
                    page.click("#close-settings")
                fixture(page, "melee")
                page.click('[data-tile="5,1"]')
                page.click('[data-tile="6,1"]')
                shot("combat")
                page.click('[data-inspect="sword"]')
                shot("combat-sword-calculation")
                page.click('[data-inspect="mace"]')
                shot("combat-both-calculations")
                page.click('[data-attack="mace"]')
                shot("combat-after")
                fixture(page, "ranged")
                page.click('[data-tile="5,1"]')
                page.click('[data-tile="7,1"]')
                page.click('[data-inspect="bow"]')
                shot("ranged-calculation")
                fresh(page, "river")
                shot("river-normal")
                page.click('[data-tile="2,3"]')
                page.click('[data-tile="5,3"]')
                shot("river-capture")
                page.click('[data-tile="5,7"]')
                shot("river-shallows")
                for name in ("low-poly", "pixel"):
                    style(page, name)
                    shot(f"river-{name}")
                style(page, "normal")
                fresh(page)
                tool(page, "end_turn")
                ready(page)
                shot("enemy-turn-complete")
                fixture(page, "victory")
                shot("victory")
                assert not errors, errors
                assert not failures, failures
                context.close()
                print(f"Captured {width}x{height}", flush=True)
            browser.close()
    finally:
        server.shutdown()
        server.server_close()
        staging.cleanup()
    (output / "states.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(f"Captured {len(manifest)} cases in {output}", flush=True)


def compare(baseline, current):
    expected = json.loads((baseline / "states.json").read_text())
    actual = json.loads((current / "states.json").read_text())
    problems = []
    for key in sorted(set(expected) | set(actual)):
        # Reusing an artifact directory must not leave obsolete diagnostics
        # after a later successful comparison.
        for suffix in ("-diff.png", "-marked.png"):
            (current / f"{key}{suffix}").unlink(missing_ok=True)
        if expected.get(key) != actual.get(key):
            problems.append(f"{key}: battle state differs")
        left = Image.open(baseline / f"{key}.png").convert("RGBA")
        right = Image.open(current / f"{key}.png").convert("RGBA")
        if left.size != right.size:
            problems.append(f"{key}: image dimensions {left.size} != {right.size}")
            continue
        a, b = np.asarray(left), np.asarray(right)
        mask = np.any(a != b, axis=2)
        if mask.any():
            ys, xs = np.where(mask)
            problems.append(f"{key}: {int(mask.sum())} changed pixels; bounds ({xs.min()}, {ys.min()})..({xs.max()}, {ys.max()})")
            diff = np.abs(a.astype(np.int16) - b.astype(np.int16)).astype(np.uint8)
            diff[:, :, 3] = 255
            Image.fromarray(diff).save(current / f"{key}-diff.png")
            overlay = np.asarray(right).copy()
            overlay[mask] = [255, 0, 255, 255]
            Image.fromarray(overlay).save(current / f"{key}-marked.png")
    (current / "comparison.txt").write_text("\n".join(problems) + ("\n" if problems else ""))
    if problems:
        print("\n".join(problems))
        raise SystemExit(f"FAIL: {len(problems)} parity differences; see {current}")
    print(f"PASS: {len(expected)} full-page images and battle states match exactly")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--compare", type=Path)
    args = parser.parse_args()
    if args.capture:
        capture(args.capture, args.output)
    if args.compare:
        compare(args.compare, args.output)
