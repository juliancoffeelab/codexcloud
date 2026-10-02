"""Browser smoke checks. Requires Playwright and Chromium; starts its own server."""

import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import shutil
from threading import Thread

from playwright.sync_api import sync_playwright


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_):
        pass


TOOLS = """
Object.defineProperty(document, 'modelContext', {value: {
  registerTool(tool) {
    window.hexSmokeTools ??= {};
    window.hexSmokeTools[tool.name] = tool;
  }
}});
"""


def snapshot(page):
    return page.evaluate("window.hexSmokeTools.get_battle_state.execute()")


def choose_style(page, style):
    if not page.locator("#settings-dialog").evaluate("el => el.open"):
        page.click("#settings-button")
    page.select_option("#art-style", style)
    page.wait_for_function(
        "style => document.documentElement.dataset.artStyle === style && "
        "!document.querySelector('#art-style').disabled",
        arg=style,
    )
    page.click("#close-settings")


def new_battle(page):
    page.click("#settings-button")
    page.click("#new-game")
    page.click("#confirm-reset")
    assert snapshot(page)["round"] == 1


def run(url, screenshots):
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(executable_path=shutil.which("chromium"))
        context = browser.new_context(viewport={"width": 1280, "height": 900})
        context.add_init_script(TOOLS)
        page = context.new_page()
        errors, failures = [], []
        page.on("pageerror", lambda error: errors.append(str(error)))
        page.on("requestfailed", lambda request: failures.append(request.url))
        page.on("response", lambda response: failures.append(response.url)
                if response.status >= 400 else None)
        page.goto(url, wait_until="networkidle")
        assert page.locator("#map [data-tile]").count() == 77

        assert page.locator('#end-turn #round').inner_text() == 'Turn 1'
        assert page.locator('#art-style option[value="classic"]').count() == 0
        assert page.locator('.hex.selected,.hex.reachable').count() == 0
        assert not page.locator('#unit-panel').is_visible()
        page.click('[data-tile="1,3"]')
        assert page.locator('.unit-stat svg').count() == 1
        assert page.locator('.weapon-table tbody tr').count() == 2
        assert page.locator('.resistance').count() == 3
        assert page.locator('#turn-badge').count() == 0
        assert page.locator('#turn-label').inner_text() == 'End turn'
        assert page.locator('.terrain-keep .terrain-art').count() == 2
        # A real move captures a village, then each style must preserve that battle.
        page.click('[data-tile="1,2"]')
        page.click('[data-tile="3,1"]')
        state = snapshot(page)
        assert next(u for u in state["units"] if u["id"] == "u3")["tile"] == "3,1"
        assert next(v for v in state["villages"] if v["id"] == "3,1")["owner"] == "player"
        for style in ("low-poly", "pixel", "normal"):
            choose_style(page, style)
            assert snapshot(page) == state, f"{style} reset the battle"
            assert page.locator("#map .unit-art").count() == 6
            assert page.locator("#map .terrain-art").count() == 77
            assert page.locator('#map .unit-art[width="62"]').count() == 2
            assert page.locator('#map .unit-art[data-faction="player"]').count() == 3
            assert page.locator('#map .unit-art[data-faction="enemy"]').count() == 3
            assert page.locator('#map .enemy-unit').first.evaluate("el => getComputedStyle(el).filter") == "none"
            assert float(page.locator('#map .unit-art[data-faction="enemy"] img').first.evaluate("el => parseFloat(el.style.top)")) < -400
            if screenshots:
                page.screenshot(path=str(screenshots / f"desktop-{style}.png"), full_page=True)
        choose_style(page, "pixel")
        assert page.locator(".pixel-art img").first.evaluate(
            "image => getComputedStyle(image).imageRendering") == "pixelated"
        page.reload(wait_until="networkidle")
        assert page.input_value("#art-style") == "pixel"
        assert not page.locator("#settings-dialog").evaluate("el => el.open")
        print("PASS: all three styles render; switching preserves battle; preference persists")

        # Choose the destination terrain first, then purchase directly onto it.
        for unit_type, price in (("soldier", 8), ("archer", 8), ("scout", 5)):
            new_battle(page)
            tile = '0,2'
            page.click(f'[data-tile="{tile}"]')
            assert page.locator('#unit-panel [data-recruit]').count() == 0
            page.click(f'#terrain-panel [data-recruit="{unit_type}"]')
            recruited = next(u for u in snapshot(page)["units"] if u["id"] == "u7")
            assert recruited["type"] == unit_type and recruited["tile"] == tile
            assert recruited["moved"] and recruited["attacked"]
            assert snapshot(page)["gold"]["player"] == 12 - price
        print("PASS: soldier, archer, and scout recruitment, cost, and next-turn restriction")

        new_battle(page)
        page.locator('[data-tile="1,3"]').focus()
        page.keyboard.press("ArrowUp")
        page.keyboard.press("Enter")
        assert page.locator("#unit-panel h2").first.inner_text() == "Scout"
        page.click('[data-tile="3,1"]')
        page.click("#end-turn")
        assert page.locator("#turn-label").inner_text() == "Enemy turn"
        page.wait_for_function("window.hexSmokeTools.get_battle_state.execute().round === 2")
        assert snapshot(page)["turn"] == "player"
        assert snapshot(page)["gold"]["player"] == 14
        assert page.locator('#end-turn #round').inner_text() == 'Turn 2'
        print("PASS: keyboard selection, village capture, income, and enemy turn")

        # Advance using legal moves until a real UI attack becomes available.
        new_battle(page)
        attacked = False
        for _ in range(12):
            for unit in [u for u in snapshot(page)["units"] if u["side"] == "player"]:
                actions = page.evaluate(
                    "id => window.hexSmokeTools.get_unit_actions.execute({unitId:id})", unit["id"])
                page.click(f'[data-tile="{unit["tile"]}"]')
                if actions["attacks"]:
                    victim_id = actions["attacks"][0]["id"]
                    victim = next(u for u in snapshot(page)["units"] if u["id"] == victim_id)
                    page.click(f'[data-tile="{victim["tile"]}"]')
                    assert page.locator(".duel .combatant").count() == 2
                    forecast = actions["attacks"][0]
                    health = page.locator(".health-numbers").all_inner_texts()
                    assert f'{victim["hp"]} /' in health[1]
                    assert f'{unit["hp"]} /' in health[0]
                    if screenshots:
                        page.screenshot(path=str(screenshots / "desktop-combat.png"), full_page=True)
                        page.set_viewport_size({"width": 390, "height": 844})
                        page.screenshot(path=str(screenshots / "mobile-combat.png"), full_page=True)
                        page.set_viewport_size({"width": 1280, "height": 900})
                    page.click(f'[data-attack="{forecast["recommendedAttackId"]}"]')
                    after = next((u for u in snapshot(page)["units"] if u["id"] == victim_id), None)
                    assert after is None or after["hp"] < victim["hp"]
                    attacked = True
                    break
                if actions["moves"]:
                    destination = max(actions["moves"], key=lambda move: int(move["tile"].split(",")[0]))
                    page.click(f'[data-tile="{destination["tile"]}"]')
            if attacked or snapshot(page)["winner"]:
                break
            round_number = snapshot(page)["round"]
            page.click("#end-turn")
            page.wait_for_function(
                "round => {const s=window.hexSmokeTools.get_battle_state.execute(); "
                "return s.round > round || !!s.winner}", arg=round_number)
        assert attacked, "No real combat executed"
        print("PASS: sprite selection, attack preview, damage, and enemy progression")

        new_battle(page)
        for width in (390, 320):
            page.set_viewport_size({"width": width, "height": 844})
            assert page.evaluate("document.documentElement.scrollWidth <= innerWidth")
            bar = page.locator(".battle-bar").bounding_box()
            for selector in (".header-economy", "#round", ".treasury", "#end-turn", "#settings-button"):
                box = page.locator(selector).bounding_box()
                assert box["y"] >= bar["y"] and box["y"] + box["height"] <= bar["y"] + bar["height"]
            assert page.locator("#settings-button").bounding_box()["height"] >= 44
            assert page.locator("#terrain-legend, .orders, .map-caption").count() == 0
            assert page.locator("#status").bounding_box()["height"] == 1
            page.locator(".map-scroll").evaluate("el => el.scrollLeft = el.scrollWidth")
            assert page.locator(".map-scroll").evaluate("el => el.scrollLeft > 0")
            page.locator(".map-scroll").evaluate("el => el.scrollLeft = 0")
            for style in ("normal", "low-poly", "pixel"):
                choose_style(page, style)
                if screenshots:
                    page.screenshot(path=str(screenshots / f"mobile-{width}-{style}.png"), full_page=True)
        print("PASS: 390px and 320px layouts, touch-sized selector, and battlefield scrolling")
        assert not errors, errors
        assert not failures, failures
        context.close()

        context = browser.new_context()
        context.add_init_script(TOOLS)
        page = context.new_page()
        page.goto(url, wait_until="networkidle")
        state = snapshot(page)
        pattern = "**/assets/terrain-pixel.png*"
        page.route(pattern, lambda route: route.abort())
        page.click("#settings-button")
        page.select_option("#art-style", "pixel")
        page.wait_for_function("!document.querySelector('#art-style').disabled")
        assert page.input_value("#art-style") == "normal"
        assert snapshot(page) == state
        page.unroute(pattern)
        choose_style(page, "pixel")
        context.close()

        context = browser.new_context()
        context.add_init_script(TOOLS)
        context.add_init_script("Object.defineProperty(window, 'localStorage', {get(){throw Error('blocked')}})")
        page = context.new_page()
        page.goto(url, wait_until="networkidle")
        assert page.input_value("#art-style") == "normal"
        choose_style(page, "low-poly")
        context.close()

        context = browser.new_context()
        context.add_init_script(TOOLS)
        context.route("**/assets/*-normal.png*", lambda route: route.abort())
        page = context.new_page()
        page.goto(url, wait_until="networkidle")
        page.wait_for_function("document.documentElement.dataset.artStyle === 'low-poly'")
        assert len(snapshot(page)["units"]) == 6
        context.close()
        print("PASS: failed asset retry, blocked storage, and initial alternate artwork fallback")
        browser.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--screenshots", type=Path)
    args = parser.parse_args()
    if args.screenshots:
        args.screenshots.mkdir(parents=True, exist_ok=True)
    directory = Path(__file__).resolve().parents[1] / "dist"
    server = ThreadingHTTPServer(("127.0.0.1", 0), partial(QuietHandler, directory=str(directory)))
    Thread(target=server.serve_forever, daemon=True).start()
    try:
        run(f"http://127.0.0.1:{server.server_port}/", args.screenshots)
    finally:
        server.shutdown()
        server.server_close()
