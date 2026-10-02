# Hex Command

A standalone browser game about capturing villages, recruiting units, and
defeating the enemy commander on a hex battlefield.

Install and run from this folder (Node.js 22.12+):

```sh
npm ci
npm run dev
```

The game uses Svelte 5 with strict TypeScript, built by Vite. It remains a static,
client-only game: no server or account is required. To build and serve the
production files:

```sh
npm run check
npm test
npm run build
python3 -m http.server 8000 --directory dist
```

Source lives in `src/`; `dist/` is generated and must not be edited.

- `src/game/` contains browser-independent rules, AI, typed domain models,
  save validation, and move undo. Save keys, format, battle rules, and deterministic
  enemy replay remain compatible with the original game.
- `src/controller/` owns the play session and actions, autosaves, turn sequencing,
  feedback, preferences, and browser tools. UI controls and tools use the same
  action methods; rendering does not save or mutate the battle.
- `src/ui/` contains Svelte components for the battlefield, panels, combat,
  controls, and dialogs. Pure `*-view.ts` helpers prepare display states, labels,
  and forecasts so templates describe the interface rather than calculate it.
  `src/App.svelte` assembles the interface.
- `src/presentation/` contains sprite, audio, motion, and icon helpers.
- `src/style.css` retains the original global styles. `public/assets/` contains
  the original artwork and audio, copied unchanged into production builds.

The **Art style** selector in the cog settings menu switches between Normal,
Low-poly, and Pixel. It remembers your preference in this browser and keeps the
current battle intact when switching.

Terrain uses dedicated flat pointy-top hex artwork, rendered with square crops
and uniform scaling, clipped to the board edges. Terrain sheets render with native
SVG images, including forest brightness and pixel-art scaling. No mesh warping is used. Unit sheets contain teal player and crimson enemy rows with natural skin tones.
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

## Formatting

Prettier and its Svelte plugin format source, tests, configuration, and this README.
Use `npm run format` to apply formatting and `npm run format:check` to verify it.
Generated output and original binary assets are excluded.

## Checks

`npm run check` checks application components and TypeScript tests with `strict`
and `noUncheckedIndexedAccess`. `npm test` runs small unit checks for rules,
publication, immutable views, undo, persistence, audio, and asynchronous cancellation.
The entire unit command has a 20-second limit, enforced by a shared runner budget.
Tests use controlled promises instead of waiting through gameplay animations.

There are no browser automation, screenshot, or migration/parity suites. These
constraints are recorded in `AGENTS.md` and apply to future changes as well.

Components receive an immutable `BattleView` and a command-only `BattleCommands`
facade. The controller owns the mutable engine, publishes undo and recruitment
availability with each view, and invalidates stale asynchronous results. Queries
on `GameView` read one detached, frozen revision and expose no engine commands.

The sticky top bar holds income, village count, coin treasury, End turn with its turn counter,
and settings. Settings contains New battle, the field manual, and
the battle journal. Selecting an attack target shows both units’ current health. Every weapon button
has its own labelled player/enemy HP forecast and damage segments. The magnifying
glass opens numeric damage and reply equations; unit damage multipliers stay in
the unit stats panel. Armor rounding happens before cover and exposure.

Battles autosave in this browser after player actions and at the start of each
player turn. Before an enemy turn, a checkpoint is saved for deterministic replay
if the page closes during its animation. Finished battles also persist. New battle
replaces the save; unsupported or invalid saves start fresh. No migration or
cross-device synchronization is performed.

Click a tile to inspect its terrain, including beneath friendly or enemy units.
Terrain cards explain movement, protection, village income and healing, and keep
ownership and recruitment. Inspection also works during enemy turns and after a battle.
With your commander on your own keep, click an adjacent empty land hex to
open its terrain recruitment menu above the tile description. Choose a unit to buy it directly onto that hex.
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
Rebuild WAVs in `public/assets/audio/` with `python scripts/generate-sfx.py` (requires numpy and scipy).

River forests sit above the northern village approach and along the outer southern
edge, leaving the starting soldier and scout clear routes to nearby villages.
Current river saves use these revised forest positions without losing progress.

Recruitment plays a brief pair of coin taps, with no horn or spawned-unit cue.

Turn changes and undo are silent; their visual feedback remains.

Scouts and archers use the soft plains/grass sound for selection and movement.

Soldiers use the heavier footstep thud previously used by archers.

All cues have a lower playback level, rounded attacks, and gentler high frequencies.

## Weapon choices and armor

Select an enemy, then tap a weapon to attack immediately. Each bordered action shows
the damage dealt and taken, plus its own HP forecasts. The magnifying glass reveals
exact numeric damage calculations without spending an action. Every unit still attacks once per
turn; attacking ends its movement.

| Unit             | Melee                          | Ranged                   | Flat armor / weakness             |
| ---------------- | ------------------------------ | ------------------------ | --------------------------------- |
| Commander        | Sword: 4 Blade; Mace: 3 Impact | —                        | −1 Blade damage                   |
| Soldier (8 gold) | Sword: 4 Blade; Mace: 3 Impact | —                        | −1 Blade/Pierce; +1 Impact damage |
| Archer (8 gold)  | Knife: 2 Blade                 | Bow: 3 Pierce, range 2   | None                              |
| Scout (5 gold)   | Knife: 2 Blade                 | Sling: 1 Impact, range 2 | None                              |

Melee answers melee; ranged answers ranged. A surviving defender replies once
with its strongest legal weapon in the same category, even if it already acted
this turn. A defeated defender cannot reply. Forests and hills block both bows
and slings through them; forest cover reduces all ranged damage by 1. Hill
cover reduces all damage by 1. Flat armor and weakness change base damage first,
then cover and shallow-water exposure apply. Clamp the final result to 0;
there is no rounding and no hit-chance roll. A fully blocked hit still spends
the attack, and a surviving defender can retaliate.

On plains, soldiers receive 3 damage from a sword, 4 from a mace, 2 from a bow,
1 from a knife, and 2 from a sling. On hills, a knife deals 0 damage to a soldier;
hills or forest reduce sling damage against soldiers to 1.

The initial balance keeps soldiers strongest in close combat, archers stronger
than scouts in most ranged exchanges, and scouts focused on mobility and villages.
Maces counter armor; swords hit unarmored units harder. The enemy evaluates
weapons and retaliation, including safer replies and avoiding lethal exchanges.
Existing saves keep their units, HP, gold, map, and progress and use the new rules.

Run `npm test` for the lightweight unit checks, or
`node --import tsx tests/combat.ts` for weapon and terrain rules alone.
