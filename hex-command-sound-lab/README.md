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
view. Select, Move, each named weapon attack and Hit have independent rows below
the map. Each row's action button plays the saved assignment or default; its
named recording buttons play that exact clip/take. Swipe each row sideways to
compare. Suggested and Other recordings stay separate, and saved assignments
have a Saved group. Each card retains takes, full source and Use. Extra sounds
adds the hypothetical voice/death/reload rows. Library/Sources retain recordings,
source notes, favorites and export. Unit names remain accessible labels without
occupying the map. This is a game-style sound audition interface, not a battle
simulation or a replica of gameplay stats.

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
- Extra sounds adds hypothetical voice/reaction, fall/death, reload, capture or
  ambience rows. Unsupported firearm shots stay gaps. These proposals and
  creature/terrain proxies are not claims of natural species recordings.
- Library includes all 518 original listening entries plus labeled derived
  auditions. It retains filters, all-recordings access, favorites, take expansion,
  original game recordings, source/licensing details, downloads and JSON export.
- Stop or Escape cancels playback. Match levels is optional peak gain, not
  mastering; quiet authored edits retain their attenuation. Weapon take boundaries
  are approximate; the complete source session remains selectable.

## Material starters

Selection and hit suggestions use `item_wood_03` for solid skeletons, belt
handling for level-one frontline infantry, chain effects for higher-tier
armored infantry, and `cloth3` for lighter humanoids. Other plausible equipment
and impact recordings remain available. Bow users have nocking candidates for
selection and recorded English Longbow Shoot for attacks. Crossbows use
Crossbow Shoot; flight and handling recordings remain alternatives.
Merfolk selection defaults to `clothBelt2` across the roster. Horses and mounted
units use natural horse vocal `0286` for selection and optional voice; deer/goat
mounts use it as a proxy. Skeletal mounts instead keep `item_wood_03`.
Movement stays the board click.
Creature selection palettes live in `dist/creature-picks.js`: compact mammals and
several aquatic beasts use grunt 1; heavy reptiles, crab/tortoise and zombie bears
use grunt 4. Treants and zombie Shambler/Ghoul retain grunt 4. Leathery fliers use
cloth 2, birds/gryphons cloth 3, snakes a quiet saturated clothBelt2, octopus a
soft gurgle, spiders `bug_01` with the quiet creak as an alternative. Wasps offer designed insect
clips and arrow fletching as unconfirmed buzz candidates. These are audition
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
| Ruins | Existing hill wind or very quiet magic |
| Sacred grove | Soft magic |
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
records optional peak gain, and suggests weapon takes from silence detection.
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
