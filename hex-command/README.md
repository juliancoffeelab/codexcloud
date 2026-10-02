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
With your commander on your own keep, click an adjacent empty land hex to
open its terrain recruitment menu. Choose a unit to buy it directly onto that hex.
Prices and unavailable purchases are shown there; new recruits act next turn.
Selecting a unit first preserves normal movement into that hex.

Tap the selected unit or attack target again, or an inactive map tile, to deselect.
Clicks on the header, unit panel, and empty battlefield background preserve selection.
Escape clears selection and terrain inspection. Selecting a different unit still switches to it.

Village approaches mirror across the battlefield: a nearby hill on each side,
no adjacent forest, and balanced central forest and hill positions. Existing saved
battles retain their progress and load the updated terrain.

Short, quiet sound effects accompany selection and battle actions after a user
interaction. Toggle Sound effects in Settings; the preference is remembered.
Movement, combat, recruitment, capture, and turn changes have brief visual feedback.
Animations respect the device’s reduced-motion preference.

The Undo move button between End turn and Settings rewinds moves within the
current turn, including movement flags and captured village ownership. Attacking,
recruiting, ending the turn, or starting a new battle clears earlier moves. Undo
history lasts for the current page session; battle progress still autosaves.

Start a new battle from Settings to choose **Classic mirror** (the original
11×7 map) or **River crossings** (15×9, one bridge, one shallow crossing and six villages).
Play again also opens this choice. The river is impassable except at the bridge and shallow crossing;
the bridge costs 1 movement. Shallow water costs 3 for soldiers and commanders,
2 for archers and scouts, and adds 1 to damage received (including retaliation). River enemies use land routes
to reach crossings while keeping their existing combat priorities.
The chosen battlefield and battle progress survive reloads; older saves load
on Classic mirror. Existing river battles retain their original map layout.

Shallow water uses its own artwork in each style, including pixel-only waves
and texture in Pixel. The sound library includes 17 material-inspired families with
three shuffled variations each: armor clicks, soft grass brushes, leather and cloth,
bow release and melee impact, coins, splashes, leaves, wind, grass and gates,
plus UI/recruitment/turn/result cues. A tile interaction plays at most one sound:
attack takes priority over unit, then terrain. Rapid interactions replace the
previous cue, and muted or still-loading audio cannot play later by accident.
Weapon sounds play only when an attack resolves; target selection uses a quiet
UI tick. Repeat unit clicks and terrain inspection keep their material cue. Each
unit/terrain clip is one brief contact or texture. The commander uses a soft
leather-and-cloth rustle, and hill wind is quiet with a quick, soft onset. All
variations preload after the first user gesture to keep later clicks responsive. Audio starts only after an interaction; the Sound effects setting persists.
Rebuild WAVs with `python scripts/generate-sfx.py` (requires numpy and scipy).

River forests sit above the northern village approach and along the outer southern
edge, leaving the starting soldier and scout clear routes to nearby villages.
Current river saves use these revised forest positions without losing progress.

Recruitment plays a brief pair of coin taps, with no horn or spawned-unit cue.

Turn changes and undo are silent; their visual feedback remains.

Scouts and archers use the soft plains/grass sound for selection and movement.

Soldiers use the heavier footstep thud previously used by archers.

All cues have a lower playback level, rounded attacks, and gentler high frequencies.


## Weapon choices and resistances

Select an enemy, then tap a weapon to attack immediately. Each compact action shows
the damage dealt and taken. The magnifying glass reveals damage calculations and
enemy resistances without spending an action. Every unit still attacks once per
turn; attacking ends its movement.

| Unit | Melee | Ranged | Resistance / weakness |
| --- | --- | --- | --- |
| Commander | Sword: 4 Blade; Mace: 3 Impact | — | 25% less Blade damage |
| Soldier (8 gold) | Sword: 4 Blade; Mace: 3 Impact | — | 25% less Blade/Pierce; 25% more Impact |
| Archer (8 gold) | Knife: 2 Blade | Bow: 3 Pierce, range 2 | Normal damage |
| Scout (5 gold) | Knife: 2 Blade | Sling: 1 Impact, range 2 | Normal damage |

Melee answers melee; ranged answers ranged. A surviving defender replies once
with its strongest legal weapon in the same category, even if it already acted
this turn. A defeated defender cannot reply. Forests and hills block both bows
and slings through them; forest cover reduces all ranged damage by 1. Hill
cover reduces all damage by 1. Resistance changes base damage first, rounded to
the nearest integer, then cover and shallow-water exposure apply. Every hit deals
at least 1 damage; there are no hit-chance rolls.

The initial balance keeps soldiers strongest in close combat, archers stronger
than scouts in ranged exchanges, and scouts focused on mobility and villages.
Maces counter armor; swords hit unarmored units harder. The enemy evaluates
weapons and retaliation, including safer replies and avoiding lethal exchanges.
Existing saves keep their units, HP, gold, map, and progress and use the new rules.

Run `node tests/combat.mjs` for weapon/terrain rules and `python3 tests/combat.py`
for immediate attacks, optional damage inspection, exact outcomes, keyboard use,
and mobile layout.
