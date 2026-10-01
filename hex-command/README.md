# Hex Command

A standalone browser game about capturing villages, recruiting units, and
defeating the enemy commander on a hex battlefield.

Run it locally from this folder:

```sh
python3 -m http.server 8000 --directory dist
```

No build step or package installation is needed. The **Art style** selector
beside the title switches between Normal, Low-poly, Pixel, and the original
Classic markers. It remembers your preference in this browser and keeps the
current battle intact when switching.

`dist/engine.mjs` contains the game rules and enemy AI, `dist/app.mjs` handles
the interface, and `dist/art.mjs` displays regions of the generated PNG sheets
in `dist/assets/`. Terrain top faces are reprojected onto the board hexes,
so artwork and selection borders share the same edges. Enemy sprites use red accents; team bases and ownership dots
also identify sides. Pixel mode uses nearest-neighbor display.

The existing OpenAI Sites project is recorded in `.openai/hosting.json`; its
static directory is `dist`. Publish that entire directory, including the asset
sheets, through a Sites session with access to the project.

Site: https://hex-command-tfhlu4.lightdarkdaughter.chatgpt.site/

## Browser checks

With Python Playwright and Chromium installed, run from this folder:

```sh
python3 tests/smoke.py
```

The checks start their own local server and exercise style switching,
persistence, recruitment, movement, combat, enemy turns, mobile layout, and
asset-loading failures. Add `--screenshots /tmp/hex-command-checks` to capture
each style on desktop and mobile.
