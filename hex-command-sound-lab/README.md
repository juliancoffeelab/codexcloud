# Hexcommand Sound Lab

A standalone soundboard for the complete vendored collection: 467 CC0 recordings
and 51 archived original-game samples. Browse every recording and its detected
takes, or use suggested movement, attack, impact, equipment and voice assignments
for 238 entries across all ten released/unreleased faction lineups, plus all 27 terrain types.

Live site: https://hexcommand-sound-lab.lightdarkdaughter.chatgpt.site

The site is private; sign in with its owning ChatGPT account. It adapts to phone
screens, and playback starts only after a tap. No audio downloads at startup.
No imports, external audio hosts, API keys, database or backend are needed.

## Run locally

```sh
python3 -m http.server 4173 --directory dist
```

Open http://localhost:4173. Any static HTTP server works. To use a phone on the
same network, open the host computer's LAN address on port 4173; its firewall
must permit the connection. Serve through HTTP or HTTPS, not a file URL.

All application source and listening assets are tracked in `dist/`. That folder
is authored static-site source. The project has no runtime dependency on other
repository projects, an execution environment, or external downloads.

## Phone map view

The default view gives most of the viewport to sounds. A compact, unnamed
three-row sprite field contains **every level** of the selected faction; Terrain
contains all 27 tiles. Pan sideways and tap a sprite to hear its selection cue.
There is no search, large unit description card, generic Play button or Detailed
view. Named weapon attacks occupy the primary sound rows. Selection alternatives
are collapsed below the attacks; the shared Move click is collapsed at the very
bottom. The three-way **Selection / Hurt / Main attack** control chooses what
unit taps play. Main attack means the first attack in the unit's authored list,
using its saved sound or suggested default; a unit without an assigned main
attack reports the gap instead of substituting another cue. The mode persists
locally and migrates the previous Hit on tap switch to Hurt. Terrain taps stay
unchanged and hide the unit-specific control. Hurt mode exposes collapsed hurt
alternatives; the other modes keep selection alternatives below the attacks.
There is no Extra sounds button or separate primary Hit row. Attack buttons use the game's exact damage-type icon mapping
and weapon name from its attack list, including pierce icons for bows rather
than a generic ranged-target replacement. Swipe each row to compare recordings;
Use saves its take. Suggested/Other/Saved groups remain distinct. Library/Sources
retain recordings, source notes, favorites and export. Unit names remain
accessible labels without occupying the map.

The board automatically prefetches and decodes selected action defaults, nearby comparison cards and
visible/neighboring map selection cues. Panning and switching entities refreshes
that queue. Audio downloads and decodes in a suspended context on load; nothing plays automatically.
The first ordinary tap on a sprite/action/recording resumes audio and plays that
sound. There is no enable/arming button. Action buttons use local snapshots of
the game's SVG icons; named clip buttons have a large speaker icon.
Ready means decoded in memory. Ready playback schedules a Web Audio source in
the tap handler without awaiting fetch/decode; unprepared recordings still load
on demand. Internal diagnostics measure scheduling time rather than physical speaker
latency. Browser, OS, speakers and Bluetooth add latency; exact phone timing is
not established by desktop Chromium checks.

This follows the live game's interactive AudioContext, gesture unlock, decoded
sample warming and source-node playback pattern. It does not change the game or
promise identical acoustic latency. The audition board uses local MP3 listening
copies and an LRU decoded cache targeting 48 MiB (one oversized recording may
exceed it alone), two background fetch/decode workers and canceled stale queues.
Foreground preparation can replace an old scene's cached sounds; optional
lookahead stops when the budget is full. Rapid taps replace playback, and late
loads cannot trigger an obsolete selection. Quiet authored edits keep their gain.

Cache behavior can be checked without dependencies:

```sh
node scripts/check-audio-cache.mjs
```

## Auditioning

- Factions includes Humans, Merfolk, Skeletons, Woodland, Zombies, Elves,
  Dwarves, Cursed, Beasts and Undead Animals. All 238 entries and 27 terrains are
  on the map, including every released/unreleased advancement.
- Game-defined attacks come from authored game data; artwork-only attacks are
  proposed from equipment/role. This does not release units in the game.
- Every action separates Suggested recordings from disjoint Other plausible
  matches. There is no arbitrary cutoff. All takes/full sources remain accessible.
  `dist/matching.js` keeps weapon mechanics, equipment/material, movement and
  voices separate. Animal calls never substitute for a weapon attack.
- Use persists the recording and selected take for that exact entity/action.
  The action button and tile selection follow personal choices. Saved exposes
  the assignment and Reset restores the default. Storage is local to the browser.
- Unsupported firearm shots stay gaps. Creature/terrain proxies remain labeled;
  no natural species recording is claimed for a designed texture. Optional
  source recordings remain available in the full library.
- Library includes all 518 original listening entries plus labeled derived
  auditions. It retains filters, all-recordings access, favorites, take expansion,
  original game recordings, source/licensing details, downloads and JSON export.
- Stop or Escape cancels playback. Match levels applies per-take gain toward the live-game median;
  quiet authored edits retain their attenuation. Weapon take boundaries
  are approximate; the complete source session remains selectable.

## Play the library in sequence

Library's **Play all N** queues every recording matching the current filters and
favorites, including entries beyond the 60-row display limit. With no filters this
is all 537 complete recordings (about 20:15). Each full source plays once; take
ranges are not added again. The queue is a snapshot, so changing filters while it
plays changes the next Play all selection, not the running order.

The button becomes Stop all, and the footer shows recording number/name. Either
Stop button or starting another sound cancels the queue; canceled late loads do
not resume it. Failed recordings are skipped and counted at completion. Only one
source is fetched/decoded ahead, using the existing bounded cache. Match levels
can change during the queue without restarting it. As with other playback, hiding
the tab stops playback; manual seeking switches to that single recording.

## Loudness checks and playback matching

Run `python3 scripts/check-loudness.py` to measure every decoded listening copy
and each take. Sources links the complete CSV/JSON reports. Measurements include
sample peak, RMS, strongest 50 ms RMS, leading quiet, projected playback levels
and flags for quiet ranges or clipping. They include app volume (default 55%)
and the selected gain profile, but cannot measure phone speakers or system volume.
Short effects are measured with windowed RMS rather than pretending that integrated
LUFS is reliable for every tiny clip. CSV rows distinguish full sessions and takes.
Use `--only "Axe Swing" --output tmp/axe.json` for one source; `--no-normalize`
models raw playback, and `--volume 100` models a different app volume.

Match levels now uses the **selected take**, not a single loud peak elsewhere
in its full recording. `loudness-reference.json` anchors the target to the median
strongest-50ms RMS of the live game's six existing melee/bow cues at its 0.29
master gain. At the board's reference 55% volume that output is about -26.3 dBFS.
Turning the visible volume slider still scales everything normally. Archived game
samples retain their relative game levels, including selection's extra 0.25 gain.
Authored quiet edits keep zero makeup gain. New-source take/session gains aim at
the reference with a -1.5 dBFS sample-peak ceiling and bounds of -18/+24 dB.
This is constant gain, not a compressor or a guarantee of equal perceived loudness.
Unchecking Match levels plays the original listening copy with only app volume.

`build-playback-levels.py` verifies the report's catalog/listening hashes and writes
`dist/playback-levels.json`. To regenerate after changing audio, run these in order:

```sh
python3 scripts/check-loudness.py --legacy-gains
python3 scripts/build-playback-levels.py
python3 scripts/check-loudness.py
python3 scripts/check-playback-levels.py
```

The full listening builder runs this measurement/gain/report pipeline and bundles
the reports, profiles and reference with the listening ZIP. No source recording or
authored processing recipe is changed by playback matching. Numerical checks cover
quiet/loud takes, transient headroom, silence, leading quiet, every published range,
retention of quiet edits and the original game's relative gains.

## Weapon-led selection

`dist/weapon-selection.js` names the specific exceptions to material defaults:
Human scouts/ranger/shadowblade and elven archers/two-knifers use dagger handling;
human and dwarven crossbowmen/arbalesters share archer nocking. Elven swordsmen
use Sabre Draw Fast; jumpers use Crossbow Bolt Passby as a projectile proxy. Dwarven
arquebusiers use a metal latch mechanism proxy. Slingers use leather pouch
handling, not a claimed sling recording. Existing weapon/material alternatives
remain available. These rules only change selection. Ordinary infantry,
miners/hammerers/polearm dwarves, netters/tide hunters, mounts and undead retain
their established defaults. No new recording downloads are required.

## Material starters

Selection and hit suggestions use `item_wood_03` for solid skeletons, belt
handling for level-one frontline infantry, chain effects for higher-tier
armored infantry, and `cloth3` for lighter humanoids. Other plausible equipment
and impact recordings remain available. Bow users have nocking candidates for
selection and recorded English Longbow Shoot for attacks. Crossbows use
Crossbow Shoot; flight and handling recordings remain alternatives.
Merfolk selection defaults to `clothBelt2` across the roster. Horses and mounted
units use the trimmed `0286` horse burst for selection and optional voice; deer/goat
mounts use it as a proxy. Skeletal mounts instead keep `item_wood_03`.
Movement stays the board click.
Creature selection palettes live in `dist/creature-picks.js`: compact mammals and
some aquatic beasts use grunt 1; heavy reptiles, crab/tortoise and zombie bears
use grunt 4. Treants and zombie Shambler/Ghoul retain grunt 4. Leathery fliers use
cloth 2, birds/gryphons cloth 3, snakes a quiet saturated clothBelt2, octopus a
soft gurgle, wasps use `alien_03`, spiders `alien_04`, and rays `alien_02`. Previous bugs,
grunt and creak options remain alternatives. These are audition
proxies, not authentic species recordings. Breath is restricted to zombie
choices; cloth 1 is excluded from unit action choices. Originals remain in the
full library. Wing/contact palettes apply to selection; vocal slots do not
label cloth or creaks as animal voices. Attacks remain separate.
Unit movement defaults to the three archived `select` board-click variations,
played once per completed move. Wings, cloth, gurgles and creature textures belong
to selection; terrain tap/contact preferences remain separate.
Skeletal selection, including hounds, riders, horses and bone dragons, defaults
to `item_wood_03`; spectral units keep their separate cues. Zombie selection and
optional voice default to `breath`. Cleaver, Plague carrier, Shambler and Ghoul
use `grunt_04`; fire-holding casters keep breath even on the larger sprites.
Personal Use assignments take precedence over these editorial suggestions.

## Current audition curation

`curation-notes.json` records the user's approved picks, review families and proxies
and is downloadable from Sources. Horse 0286 has a separate 0.205–0.610 s cut;
ghosts use Grunt 2 faded out at 0.360 s before the scratchy tail. These trim-only
edits are level-matched like other sources (`preserveLevel: false`); the new quiet
filtered/distorted Book 03 forest cue preserves its deliberately low level.

Axe/polearm and spear attacks now use isolated axe/katana and spear/sabre contacts.
Near-silent recorded Swing sessions remain in Library but are excluded from unit
suggestions. Bow releases prefer Scythian Recurve Shoot, with English Longbow
Shoot retained; crossbows also suggest Bolt Passby. Fire attacks use Spell Fire 07,
physical staff contacts Wood 05, and sling release Book Close as a proxy.

Skeletons have a collapsed **Death candidate** for Eat 1; the human knight has Metal
Pot 3. These do not add deaths to gameplay or alter Hurt. Move offers Wood 04 as a
trial alternative and retains the original board click. Mining/Plank/Plate, creaks
and other blade contacts remain review candidates. Existing originals and personal
choices remain available. A one-time backed-up migration rebinds saved raw 0286
mount cues to the trim, replaces old ghost cloth/leather choices, and retires
explicitly rejected saved Swing assignments. Old curated ruins/grove magic
assignments are rebound to the newly requested hill/magic routing; unrelated
personal choices are retained.

## Terrain audition edits

Terrain preferences are authored in `dist/terrain-picks.js`, separate from the
semantic matching rules. Suggested records and family alternatives can be
changed there without changing the original recordings. Shared door and water
presets keep related tiles consistent. Current starters include:

| Terrain | Tap cue |
| --- | --- |
| Road / bridge | `stones_01` |
| Plains | Soft, bass-cut `footstep_grass_003` |
| Keep | Quiet light-metal contact |
| Fort | Wood, approximately matched to Keep |
| Marsh | Existing water or a heavily softened burble |
| Ruins | Existing hill wind |
| Sacred grove | Former quiet ruins magic |
| Ferry / landing / water | Existing generated water |
| House / hill monastery / church | Quiet door contact |
| Village | Existing village cue |
| Hills | Existing wind cue |
| Mountains | Heavily quieted mining |
| Quarry cave | More present mining |
| Burned village / house | Quiet fire excerpt |
| Road sign | Existing cue |
| Bell keep | Bell |
| Castle courtyard | Existing cue |
| Burial ground | Skeleton starter `item_wood_03` |
| Ritual mark | Quiet `fout-03` excerpt |

Road and plains also use their chosen cues for surface contact. Other tile
movement suggestions remain distinct from tap/interaction sounds; a door cue
does not become a walking sound. Sources without a new preference keep their
existing candidates. Personal Use assignments retain priority.

`audio-edits.json` records source hashes, exact FFmpeg filter recipes and level
references. `build-soundboard.py` invokes `scripts/build-edits.py` to regenerate
these copies and include them in the catalog and download ZIP. Edits render
from verified vendor originals, not from compressed listening copies. Authored
excerpts and fades preserve the full unmodified source as an alternative.
Keep and Fort are calibrated to the archived keep cue's integrated RMS; quarry
mining is more present than mountain mining. These are first audition mixes,
not perceptual loudness mastering. Intentional attenuation is retained when
Match levels is enabled. Original recordings and catalog IDs stay untouched.

## Refreshing faction and terrain snapshots

All snapshot data/artwork is already tracked; running or deploying the site has
no cross-project dependency. To refresh it from an explicit game checkout:

```sh
python3 scripts/snapshot-workbench.py --game-dir /path/to/hex-command
```

Terrain snapshots copy the canonical complete frames selected by the game's
`terrain-preview.ts`, including floors and assembled props. The source frame keys,
PNG hashes and native sizes are recorded in
`dist/workbench/terrain-portraits/provenance.json`. To refresh just those portraits
without touching any unit snapshot:

```sh
python3 scripts/terrain_portraits.py --game-dir /path/to/hex-command
```

The game checkout must already have its generated pixel assets (run its own
artwork build there first). This copies canonical lineup membership, levels,
roles, gameplay IDs, attacks, terrain and unchanged unit sprites (64×64) and native terrain artwork into
`dist/workbench.json` and `dist/workbench/`. It reads but never modifies the
supplied checkout. Artwork-only attacks are marked proposed, and snapshot
metadata records the source revision. This is an audition inventory rather than
live gameplay integration. Read the generated proposals when reviewing a refresh.

## Originals and reproducible listening copies

[`vendor/sfx/`](vendor/sfx/README.md) contains the 467 unmodified originals,
CC0 licenses, source evidence, acquisition lock, hashes and validation. Normal
Git tracks them all; no LFS or external storage is required. The 51 archived game
samples remain separate and are not represented as CC0.

To rebuild listening assets with Python 3.11+, FFmpeg and FFprobe:

```sh
python3 scripts/build-soundboard.py
```

The builder reads only this project's locked originals and metadata. It converts
CC0 recordings to mono 44.1 kHz / 96 kbps MP3 for broad browser compatibility,
retains legacy catalog peak gains for compatibility, suggests takes from silence
detection, and generates the measured per-take matching profiles described above.
Original audio is never rewritten. A clean rebuild may differ bytewise between
FFmpeg versions; source hashes identify unchanged originals.

`dist/catalog.json` indexes all recordings with source hashes, durations,
categories, take boundaries and listening paths. `sources.json`, `coverage.json`,
`provenance.json` and original license/source texts ship with the site. The
listening-library ZIP contains every listening file plus those records. It is a
portable audition collection, not a lossless copy of the large original WAVs.
The decoded cache is bounded by memory; selected and nearby sounds prepare automatically.

Unit artwork is a copied snapshot of native 64×64 game sprites, without edits or
resizing. Roster and artwork snapshots introduce no cross-project runtime imports.

## Hosting and recovery

`.openai/hosting.json` preserves the existing Sites project ID and declares
`dist/` as its static directory. Reuse that ID for updates. Publication needs
the owning Sites account and a fresh short-lived Sites source credential;
credentials are never committed. Push the exact static source before publishing
an archive containing `.openai/hosting.json` and `dist/`.

The deployed site survives environment reloads, and a fresh repository checkout
can serve the complete soundboard without rebuilding or downloading assets.
Source originals under `vendor/` need not be included in the hosted static output.

Navigation handlers are scoped to the actual tabs; bubbling audio clicks never
reselect the current view or cancel playback. The first-tap browser regression
check waits for a recording to finish, not merely for its source node to start.
