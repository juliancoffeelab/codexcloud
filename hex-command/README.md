# Hex Command

A standalone browser game about capturing villages, recruiting units, and
defeating the enemy commander on a hex battlefield.

Run it locally from this folder:

```sh
python3 -m http.server 8000 --directory dist
```

No build step or package installation is needed. The **Art style** selector
in the cog settings menu switches between Normal, Low-poly, and Pixel. It remembers your preference in this browser and keeps the
current battle intact when switching.

`dist/engine.mjs` contains the game rules and enemy AI, `dist/app.mjs` handles
the interface, and `dist/art.mjs` displays regions of the generated PNG sheets
in `dist/assets/`. Terrain uses dedicated flat pointy-top hex artwork, rendered with square crops
and uniform scaling, clipped to the board edges. No mesh warping is used. Unit sheets contain teal player and crimson enemy rows with natural skin tones.
Terrain shares a grassy base with lightly shaded forest floor, small stone patches, village dirt paths,
and walled castles with paved courtyards. Grass, trees, and waves use irregular
arrangements; the two main hill formations and village houses retain their familiar shapes.
Roles have distinct silhouettes, with no color filter applied. Commanders carry
faction standards and render slightly larger on the map. Team bases and ownership dots
also identify sides. Pixel units use coarse blocks and simplified shading to match the terrain,
with nearest-neighbor display.

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

The sticky top bar holds income, village count, coin treasury, End turn with its turn counter,
and settings. Settings contains New battle, the field manual, and
the battle journal. Selecting an attack target shows both units with current
and predicted health, including retaliation and defeats.

Battles autosave in this browser after player actions and at the start of each
player turn. Before an enemy turn, a checkpoint is saved for deterministic replay
if the page closes during its animation. Finished battles also persist. New battle
replaces the save; unsupported or invalid saves start fresh. No migration or
cross-device synchronization is performed.

Click a tile to inspect its terrain, including beneath friendly or enemy units.
Terrain cards explain movement, protection, village income and healing, and keep
ownership and recruitment. Inspection also works during enemy turns and after a battle.
The commander’s recruit buttons live in its ability section.

Tap the selected unit or attack target again, or an inactive map tile, to deselect.
Clicks on the header, unit panel, and empty battlefield background preserve selection.
Escape clears selection and cancels recruitment. Selecting a different unit still switches to it.

Village approaches mirror across the battlefield: a nearby hill on each side,
no adjacent forest, and balanced central forest and hill positions. Existing saved
battles retain their progress and load the updated terrain.
