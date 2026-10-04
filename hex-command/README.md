# Hex Command

A standalone browser game about capturing villages, recruiting units, and
defeating the enemy commander on a hex battlefield, with three story campaigns.

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

Campaign artwork uses the same native-resolution unit cropping and style loading
as the original roster. Separate source sheets keep larger unit frames without
resampling the PNGs; all campaign assets load before a style switch completes.

The existing OpenAI Sites project is recorded in `.openai/hosting.json`; its
static directory is `dist`. Publish that entire directory, including the asset
sheets, through a Sites session with access to the project.

Site: https://hex-command-tfhlu4.lightdarkdaughter.chatgpt.site/

## Campaigns

Open **Settings → New campaign**. Three stories each have three linked missions:

| Campaign         | Your company            | Objectives                                                             |
| ---------------- | ----------------------- | ---------------------------------------------------------------------- |
| The Lantern Road | Nera’s human army       | Escort Ivo, reclaim villages, silence grave ruins and the Bellwarden   |
| The Green Accord | Alder’s woodland beasts | Reclaim groves, defend through dawn, escort a witness across the river |
| The Ashen Oath   | Orin’s unbound undead   | Escape a quarry, secure a refuge, break the Master’s binding stones    |

Portrait conversations introduce missions and react to optional side quests and
outcomes. Move between lines with the previous/next arrows or keyboard arrows;
read past lines in Settings. Conversations and
mission progress survive reloads. Gold-bordered map hexes mark objectives. The magnifying glass between undo and
settings opens the mission briefing, hints, objectives and optional side quests;
these details start collapsed so the battlefield remains visible. Enemy armies contest
control objectives and threaten escorts; defeating their leader does not replace
an escort or territory objective.
Hints are brief clues rather than instructions or a schedule of future enemies.

The Quiet Refuge is a defense and mediation mission. Its villages sit on Orin’s
side of the shelter; claiming both is no longer a victory requirement. Defend the
refuge through six enemy turns. Medicine from the northern grove gives six gold
and brings the villagers’ intervention after five defended turns. Losing the refuge
resets that count. Enemies withdraw when every available attack would kill them
through retaliation. They favor safe owned healing villages or open ground, with
a visible retreat arrow; cornered troops may still fight. A safe attack permits
reengagement, while previously withdrawing troops can recover in owned villages.
Any death of a withdrawing soldier or Dain ends mediation, including a death from
defensive retaliation.
It changes the objective to defeating the remaining patrol and removes
Dain’s alliance and living recruitment in the final mission. The remaining undead
company receives two companions instead. Refuge save upgrades move the villages
while preserving units, gold and ownership; defense tracking begins on the updated rules.

The Green Accord ends its opening battle when all three groves are reclaimed.
Lark then conducts a vigil in the central grove: protect her there through three
complete enemy turns. Leaving the grove or losing control resets the vigil.
Lark must survive every mission and returns as part of the story company, without
using a recall slot. Dain’s death persists across the campaign; Tamsin investigates
the woodcutters’ deaths independently if he cannot return. The final mission
requires both delivering Lark to the village meeting and securing the bridge fort
so the households can share the crossing. Mission results explain what happened
and why the company moves on.

After a victory, select up to four surviving recruits to bring to the next mission.
They heal completely, and up to eight spare gold carries forward. Each mission
also supplies its own starting company and treasury. Couriers, leaders and required companions have
fixed story roles rather than occupying recall slots. Losing a leader, required
courier or story companion ends the mission; Retry mission restores its original company, recall
and treasury. There is no arbitrary loss deadline. Hold objectives check ownership
at the beginning of a player turn, starting at the announced turn.

New terrain adds tactical choices: marsh costs 3 movement for heavy troops and 2
for others; ruins give 1 ranged cover without blocking shots; forts give 1 cover
against every attack; sacred groves give ranged cover, block shots through them,
and heal their owner’s occupant by 2 HP each turn. Forts, ruins and groves are
capturable but do not generate gold. Woodland units cross forest and grove for
1 movement. Only scouts also receive that discount on hills. Sealing marked
ruins stops their future grave waves permanently. Each optional supply quest
pays six gold once; move undo restores both its reward and conversation.

The nine new unit types have original teal/crimson artwork in Normal, Low-poly
and Pixel, alongside the four new terrain tiles. Sheets load for the selected style;
classic skirmishes retain their original three recruits and rules.

| New unit             | Gold |  HP | Move | Weapons                                           | Flat incoming modifiers        |
| -------------------- | ---: | --: | ---: | ------------------------------------------------- | ------------------------------ |
| Warden               |    9 |  12 |    3 | Spear 4 Pierce; Hammer 2 Impact                   | Blade −1, Pierce −1, Impact +1 |
| Adept                |   10 |   7 |    3 | Spark 3 Impact, range 2; Staff 1 Impact           | None                           |
| Skeleton             |    7 |   9 |    3 | Rust blade 3 Blade; Bone club 3 Impact            | Pierce −2, Impact +1           |
| Bone archer          |    8 |   7 |    3 | Bone bow 3 Pierce, range 2; Shard 1 Blade         | Pierce −1, Impact +1           |
| Wolf                 |    6 |   8 |    5 | Fangs 3 Blade                                     | Pierce +1                      |
| Boar                 |    9 |  13 |    3 | Tusks 4 Pierce                                    | None                           |
| Thorncaller          |    8 |   7 |    4 | Thorns 3 Pierce, range 2; Root staff 2 Impact     | None                           |
| Wildkeeper (leader)  |    — |  14 |    3 | Bough 3 Impact; Stones 2 Impact, range 2          | Impact −1                      |
| Necromancer (leader) |    — |  12 |    3 | Bone staff 3 Impact; Grave bolt 3 Impact, range 2 | Pierce −1, Impact +1           |

Balance follows counterplay rather than enemy stat boosts. A bow deals 1 damage
to an open-ground skeleton; a sword or mace deals 4. Skeleton clubs deal 4 to
armored soldiers but 3 to unarmored targets. Wardens’ spears deal 5 to wolves;
wolves compensate with speed and woodland routes. Boars survive front-line
exchanges but cannot reply to ranged attacks. Adepts punish bones and armor but
have low HP and a weak melee reply. Thorncallers trade raw damage for woodland
mobility. Every faction uses the same unit definitions and combat calculations.

Focused campaign checks cover all nine map layouts, saves, objectives, turn-boundary
holds, bounded reinforcement waves, grave suppression, one-time rewards, undo,
recall, retry, unit counters and session cancellation. Independent Luna 6 players
also play complete linked campaigns through the headless interface below. Their
failures inform map and pacing changes; the [playtest report](docs/campaign-playtests.md)
records the method, results and limitations. Browser interaction and visual parity
have not been automated, in accordance with `AGENTS.md`.

Existing campaign saves retain their deployed units. The refuge’s two village positions upgrade on reload; other saved terrain remains unchanged. Start a new
campaign, or retry a defeated mission, to use the revised starting layouts and
companies.

### Headless campaign playtesting

Agents can play the actual game without a browser or a replacement rules model:

```sh
node --import tsx scripts/play-campaign.ts /tmp/my-company.json '{"op":"start","campaign":"lantern-road"}'
node --import tsx scripts/play-campaign.ts /tmp/my-company.json '{"op":"options","units":["u1"]}'
node --import tsx scripts/play-campaign.ts /tmp/my-company.json '{"op":"act","actions":[],"endTurn":true}'
```

Each invocation reads and writes that trial file, returning JSON with the current
map, units, treasury, briefing, hints, conversations, objectives, side quests and
announced reinforcements, plus the outcome and any defined debrief after victory. Start and retry also
return unit definitions, terrain rules and the common action rules.
Coordinates are zero-based `col,row` on the game's odd-row-offset hex grid.
Terrain rules expose numeric base costs, or `"impassable"` for water; unit
`movementModifiers` apply signed changes to those base costs.
Start refuses to overwrite an existing trial; use next, retry, or a new file path.

| Operation | Parameters and behavior                                                                                                                                  |
| --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `start`   | `campaign`: `lantern-road`, `green-accord` or `ashen-oath`; starts mission one.                                                                          |
| `observe` | Optional `rules: true`; inspect the current battle.                                                                                                      |
| `options` | `units: [ids]`; legal moves, weapon forecasts and recruitment hexes. Optional `from: "col,row"` forecasts attacks from a reachable hex without moving.   |
| `threats` | `unit: id`, `tiles: [up to 12 current/reachable hexes]`; potential next-turn attacks from current enemies, using the same reachability and damage rules. |
| `act`     | `actions: [...]`, optional `endTurn: true`; move, attack and recruit, then execute the actual enemy AI if requested.                                     |
| `undo`    | Undo a move until an attack, recruitment or end turn clears the move history.                                                                            |
| `next`    | `recall: [up to four survivor IDs]`; advance after winning, with actual survivor and gold carryover.                                                     |
| `retry`   | Restart a defeated mission with its original recall and treasury.                                                                                        |
| `results` | Recorded mission outcomes, rounds and survivor counts.                                                                                                   |

Action objects use `op` and `unit`, plus `tile` for a move, `target` and optional
`weapon` for an attack, or `type` and `tile` for recruitment. Move before attacking:
an attack ends movement. New recruits act next turn. An invalid action stops the
batch and returns an error plus the authoritative observation; earlier successful
actions stay applied. Dialogue is acknowledged automatically for headless play,
and `conversation` returns the current mission's accumulated lines, including
introductions, events and the result, so agents can read them without modal clicks.

Inside `actions`, use these forms, replacing placeholders with values from
`options`:

```json
[
  { "op": "move", "unit": "UNIT_ID", "tile": "col,row" },
  { "op": "attack", "unit": "UNIT_ID", "target": "ENEMY_ID", "weapon": "WEAPON_ID" },
  { "op": "recruit", "unit": "LEADER_ID", "type": "RECRUIT_TYPE", "tile": "col,row" }
]
```

Threat inspection considers each enemy independently with refreshed actions. Its
damage sum is a conservative bound, rather than an AI prediction: movements can
conflict, and future recruitment, waves, healing and other planned moves are
excluded. It supplies mechanical information, not tactical recommendations.
The agent chooses every player action. The trial records successful actions,
undo, recall, retries and enemy-turn snapshots for review.
New trials also fingerprint their campaign definition and engine rules, rejecting
continuation if those rules change mid-run. Verify a recorded trial against its
matching source with `node --import tsx scripts/replay-campaign.ts /tmp/my-company.json`;
this replays the agent's decisions and checks every enemy response and final state,
without selecting any player actions.

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
has its own HP forecast and damage segments. The magnifying
glass opens numeric damage and reply equations; flat incoming damage modifiers stay in
the unit stats panel. Armor applies before cover and exposure; no rounding is needed.

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
