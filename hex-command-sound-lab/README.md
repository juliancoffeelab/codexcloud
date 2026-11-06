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

## Auditioning

- Factions includes Humans, Merfolk, Skeletons, Woodland, Zombies, Elves,
  Dwarves, Cursed, Beasts and Undead Animals. Every advancement in each artwork
  lineup is included. The dropdown is the complete roster; sprite buttons are
  quick picks. Search includes names, roles and attacks.
- Game-defined attacks are copied from authored game data. Artwork-only units
  carry explicitly proposed weapon actions inferred from equipment and role.
  This does not register or release units in the playable game.
- Every action separates **Suggested** recordings from **Other plausible
  matches**. The latter contains the complete rule-matched set, without a
  first-five cutoff, excluding the suggestions already shown. Expand it to
  compare every matching recording and every take. Matching is semantic and
  editable in `dist/matching.js`: weapon mechanics, target material, equipment,
  movement type and vocal type each have separate rules. Animal calls never
  enter weapon attacks merely because they share a faction.
- Main actions are Tap/selection (equipment or brief creature feedback),
  Movement, each named Attack, and Hit/taking damage. The hypothetical-extras
  toggle adds suitable voice/reaction, fall/death, reload, capture or ambience
  proposals where the collection supplies candidates. A firearm shot remains
  visibly unsupported instead of receiving an unrelated sound.
- Terrain includes every authored terrain type, with tile tap and surface
  contact suggestions. Optional contacts/capture/ambience are marked
  hypothetical. Water, wings, bone contacts and animal proxies have caveats.
- Play auditions the chosen take; each recording's dropdown includes all takes
  and the full source session. **Use** persists a recording/take for that exact
  faction/unit/action or terrain/action. Your saved choice appears on the action
  card and drives Play action sequence. Clear returns to the first suggestion.
- Stars persist a global favorite list. Export includes both favorite recordings
  and contextual assignments with original paths/hashes, source, creator,
  license and take boundaries. Browser storage is local to that device.
- Original game mode remains available for archived-family comparison. It does
  not imply that unreleased units had historical sound assignments.
- Full library retains all 518 original listening entries plus labeled processed auditions. Filter by source, category, name or
  recording type; Show every recording removes the initial result-page limit.
- Multi-take weapon boundaries are approximate. Full recordings preserve the
  entire session. Stop or Escape cancels playback and pending sequences; the
  position slider seeks. Match levels applies optional peak gain, not mastering.

## Material starters

Selection and hit suggestions use `item_wood_03` for solid skeletons, belt
handling for level-one frontline infantry, chain effects for higher-tier
armored infantry, and `cloth3` for lighter humanoids. Other plausible equipment
and impact recordings remain available. Bow users have nocking candidates for
selection and recorded English Longbow Shoot for attacks. Crossbows use
Crossbow Shoot; flight and handling recordings remain alternatives.
Merfolk selection defaults to `clothBelt2` across the roster.
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
Only six decoded recordings are cached; playback requests are made on demand.

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
