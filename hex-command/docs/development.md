# Development and publication

Development rules live in [AGENTS.md](../AGENTS.md).

## Setup

Use Node.js 22.12+, Rust stable with `rustfmt`, the `wasm32-unknown-unknown` target,
and Python 3 with Pillow for artwork. `wasm-pack` is pinned in npm dependencies.

```sh
rustup target add wasm32-unknown-unknown
npm ci
npm run dev
```

## Verification

```sh
npm run format                # Apply project formatting
npm run wasm:build            # Prepare bindings before a fresh test run
npm test                      # All test suites; warns after 20 seconds
npm run verify                # All checks and production build
npm run verify -- types tests # Selected checks
npm run build
npm run preview
```

Verification covers formatting, content, artwork, tests and the production build.
Independent checks continue after failures and report their statuses together.
The test duration target warns without terminating a successful run. Browser
automation is separate from the default checks.

## Content and artwork

A lineup is a faction. A faction can have sprites before its gameplay roster is
implemented. Character and terrain artwork use the permanent 64×64 canvas and
shared battlefield scale defined in the development rules.

```sh
npm run art:build
npm run art:check
```

Edit authored content and artwork rather than generated output. Keep temporary
experiments and screenshots in ignored `tmp/`.

## Simulations

Use the [combat-simulation skill](../.agents/skills/combat-simulation/SKILL.md)
for commands, reports and interpretation. [Army design](../.agents/skills/combat-simulation/design.md)
defines faction identities and unit-role tradeoffs. Evaluate actual battlefield
contributions, and distinguish observed outcomes from proposed causes.

The default arena roster is Plains, Forests, Crossing, Islands and Wood Edge.
Permanent arena-only layouts are registered in the manifest's `arenaScenarios`
and kept under `rust/data/simulation/maps/`; they do not enter the playable map
picker. A scoped `--landscapes` selection must not silently replace this roster.

Sprite palette entries use straight RGBA, including partial alpha for spectral
materials. Keep faction cloth readable and preserve material alpha in all faction
variants. Native layer composition uses source-over blending without resizing.

## Publication

Commit and push completed requested repository changes by default through the
feature branch and pull request, regardless of duration, unless the user requests
local-only work or a push hold.

Deploy only for an explicit request to update the live game or a task explicitly
intended to provide a live build for a human to playtest. Editing playable content,
running simulations, and pushing changes alone do not establish that intent.
Otherwise, finish with the pushed changes. Honor unpublished-release requests and
deployment holds. For an intended human playtest, finish appropriate validation
and commit/push first, then use the [deploy skill](../.agents/skills/deploy/SKILL.md)
and return the production URL. Reviews, documentation, tooling-only work, drafts,
renders, and simulations do not trigger deployment on their own.

An explicit deploy request invokes that operation immediately. Do not run
separate tests, formatting, inspections, Git commands, builds or previews. The
operation owns preparation from `HEAD`, necessary builds, Sites source synchronization, packaging,
credentials, saving the exact release and deployment, preserving the existing
audience. Exact saved releases are reused without rebuilding or saving again;
changed commits are built with their committed dependency lockfile. There are
no deployment-time test, clean-tree, branch-policy, HTML-reference or archive-audit
gates. Build/package errors and hosting failures remain genuine failures.
Return the URL only as a successful update when the operation reports
`deployed: true`. On failure, give the known site address with the error and
explicitly state that the requested update failed. Do not present an older
publication as success. A failure does not end the deployment task: before
investigating or repairing it, announce the failed stage/error, the next action
and why the live update is delayed. Fix only the observed blocker, commit/push
required changes and retry the deploy operation without requesting permission
again. Use its build to verify the repair; do not add a general validation cycle.
Continue until deployment succeeds or a concrete external prerequisite requires
user action, explaining that blocker and the known site's unchanged status.

Uncommitted implementation changes are left alone. Preparation neither checks
that the implementation working tree is clean nor stages, commits or pushes its
changes. Required implementation commits and pushes are completed before deployment.

Use the [publish skill](../.agents/skills/publish/SKILL.md) only when an
unpublished release is requested. Its `prepare` operation saves without deploying.

## Further guidance

- [Authored content](../rust/data/README.md)
- Artwork design: [unit lines](../.agents/skills/design-unit/SKILL.md),
  [terrain](../.agents/skills/design-terrain/SKILL.md).
- Shared artwork skills: [generation](../.agents/skills/pinkbox-generation/SKILL.md),
  [enhancement](../.agents/skills/pinkbox-enhancement/SKILL.md),
  [cleaning](../.agents/skills/pixel-cleaning/SKILL.md),
  [lineup previews](../.agents/skills/pixel-lineup/SKILL.md)

## Battlefield presentation

An optional `lighting` object in a scenario's `map.json` selects `time` (`day`
or `night`) and `lights`. Rust loads and validates this object with the map,
includes it in the content fingerprint, and exports it through the WASM catalog.
Runtime obtains these definitions from `MAPS`; generated terrain modules contain
only opaque image keys, URLs and image dimensions. Devtools may read source JSON
to bake images, but must not export scenario definitions into runtime modules.

Each light has integer `col`/`row`, optional `dx`/`dy`, positive `radius` and
nonnegative `power`. Its optional `when` rule contains a `condition` and a
`remember` boolean. Conditions include `owner`, `proximity`, `event`, `round`,
`noUnits`, `all` and `any`. Owner/proximity conditions can omit `tile` to use the
light or fog tile. Proximity uses hex distance (`radius: 0` means occupying the
tile), side, and optional `unitType` and `name` filters. Conditions are evaluated
by Rust. `remember: false` follows current state; `true` retains satisfaction as a
durable fact. Event completion is already remembered by campaign state.

A tile's optional `fog` uses the same rule format and specifies what clears it.
Rust exports the resolved lights and fogged tile IDs in each engine view and the
initial presentation in the catalog. The rendering worker paints soft fog; the UI hides
covered enemies; Rust rejects attacks on covered enemies. Remembered condition
facts persist in `conditionMemory`; resolved masks and rendered images do not.
Missing lighting defaults to daylight. Color transforms, palette colors and glow
rendering remain frontend presentation details.

Scenario events can use `trigger: {kind: "condition", condition: ...}`. An
optional `spawn` contains an ordered tile-ID `area` and units with stable `id`,
`type`, `side` and optional `name`/`role`. Activation fires once. Members appear
on free tiles they can traverse; unplaced members remain pending and retry on
subsequent gameplay updates even after the original condition becomes false.
Stable member markers prevent duplicate spawns after deaths or restoration.
`noUnits` includes activated pending members when determining whether a threat
has been cleared. Legacy visit/round/control triggers remain supported.

Every terrain definition explicitly declares a typed `role`, independent of its
artwork ID. Bell and ordinary keeps both use `keep`; castle courtyards use
`fortification`. Native income, healing, capture, sight blocking, recruitment and
AI objectives query roles. Race/unit terrain modifiers are keyed by role; resolved
movement and protection values remain per authored terrain variant. The UI reads
the same Rust-exported roles for recruitment, ownership, village counts, sounds
and terrain facts. Asset selection and frozen save conversions retain explicit
artwork IDs; gameplay rules must not branch on those IDs or list special variants.
Role behavior methods are exhaustive so adding a role requires choosing its rules.
An owned keep lets its leader recruit into free, passable adjacent hexes. Leaders
retain the existing army-size fight/rebuild policy; a recruiting keep does not
create a permanent stationing duty.

A group hold order can include a `when` condition. While it is true, the group
stays in its authored area. Once false, the group returns to ordinary combat
orders; it is neither frozen nor tethered to the former guard site. Captive
watches hold only while their named prisoners remain neutral captives.

Periodic rituals use `spawn.radius` around each consumed source, rather than a
fixed spawn area. An occupied mark and a captive death charge each record their
own source tile in durable group markers. Members choose the nearest free,
traversable tile within that radius. Blocked members stay pending at the same
source through save/restore; completed members never respawn. Empty marks produce
no group. Saved charge and group source references are validated on restoration.

All runtime map painting happens in one persistent module Worker using
OffscreenCanvas and createImageBitmap. It selects complete authored tile frames,
paints terrain in authored drawing order, applies lighting and fog, and transfers
one finished ImageBitmap. The main thread only swaps that bitmap using
ImageBitmapRenderingContext.transferFromImageBitmap. Units, items, health,
selection and interaction remain independent overlays; live sprite tinting does
not repaint the map.

Production prebuilds terrain PNGs with authored lighting and lantern fixtures.
The worker can load an exact matching PNG as its terrain input. Changed/custom
maps use the same worker to draw tile frames and apply lighting. Both paths then
compose fog into the finished background. The worker retains only the latest
terrain base, reusing it when only fog changes. Selection and ordinary moves
with unchanged terrain, lighting and fog request no new background.

Workers, OffscreenCanvas, createImageBitmap and bitmaprenderer are required
browser capabilities. There is no main-thread map painting, separate fog canvas,
SVG terrain substitute or compatibility renderer. Worker/script/image failures
show a diagnostic with a retry button. The previous completed background stays
visible during rendering and after failures. Replaced requests cannot publish
stale frames; obsolete bitmaps are closed and workers terminate on disposal.
Offline asset builders and draft renderers may paint through their own canvas
implementation; they are not browser runtime fallbacks.

Terrain inspection portraits use one explicitly registered image per terrain
type, shared across all floors, orientations, junctions, maps and cells. These
portrait choices never supply defaults for map rendering. On application entry,
`TerrainPreviewCache` preloads and decodes these frames; panels reuse the decoded
image nodes. This UI cache is separate from the worker's map textures and must
remain when map rendering changes. Pending or failed image loads show a loading
or retry state rather than an empty portrait.

Castle map cells declare a readable `walls` array. The offline library
bakes their paving and complete endpoint towers into each frame; the enclosed
bell owns all six towers and draws at higher z. No tile relies on neighboring
frames to supply missing tower pieces. Terrain painting draws complete images
without a polygon underfill; source artwork owns ground coverage.

Authored JSON can reject unknown fields through Serde. WASM object decoding uses
serde-wasm-bindgen's known-field iteration, so that annotation does not guarantee
rejection of extra JavaScript object keys. Geometry invariants and migration
conversion must not depend on such rejection.

The native dispatch transport publishes per-command state plus an explicit
terrain `keep`, `patch` or `replace`, tied to the prior revision. The browser
reconstructs the complete immutable view while preserving unchanged tile
identities. Board topology and rule costs are loaded on map entry, rather than
repeated with every action inspection. Undo retains immutable revisions.
Persistence still stores the full canonical checkpoint in the existing save
format; Rust serializes it directly instead of constructing another JavaScript
map solely for JSON serialization. Transport patches are never save data.

## Leadership and campaign groups

Leadership belongs to a unit instance's `role: "leader"`; unit definitions do not
classify types as leaders. Starting rosters and generated arena starts assign
roles explicitly. The same role controls loss conditions, AI protection,
recruitment and UI leadership indicators. Promotion preserves the role.

A stage's `enemyGroups` define independent participating groups. Unit placements
and spawn areas use `aiGroup` to select a group. A group's optional `when` uses
the shared condition rule: inactive members hold, active members engage opposing
units. Campaigner resolves participation and per-actor targets; mechanical
planning capabilities and command emission enforce the participation boundary.
No leader or keep is required. Untagged units retain ordinary campaign orders.

## Campaign equipment

Scripted events can grant `items` to recipients selected by side, instance role
or name. Each item has a stable ID, a description and granted attacks; item
weapon IDs must be distinct from ordinary unit attacks. The engine waits for a
matching recipient and grants each item once per recipient. Inventory is durable
unit state: it survives checkpoints, promotion, story carryover and recall.
Combat, forecasts, tactical planners and the UI share the unit's base attacks
plus its equipment attacks. Dropping equipment leaves it on the current tile without spending movement.
Move away, then move another unit onto the tile to collect it. Death also drops
all carried equipment. Items with conflicting inventory or weapon IDs stay on
the ground. Ground inventory is durable battle state, initialized for old saves
by the version-seven migration. Optional item `healing: {amount, radius}` grants
a carried aura at allied turn start; overlapping sources use the strongest heal.
Item artwork uses transparent 64×64 foreground props under
`pixel/art/terrain/props/items/<item-id>/`. The item ID selects artwork only;
Rust owns item effects and pickup/drop rules.

## Objective display

Rust inspection publishes required, optional and survival objective status.
Completed objectives retain their check marks after their deadlines. An unfinished
event objective fails when its exclusive round limit has passed, a cancelling
event has completed, or a required predecessor has become unavailable. Unfinished
objectives also fail when the chapter ends. Survival objectives distinguish living
or safely evacuated participants from casualties. The frontend renders pending
objectives with a dash, completed objectives with a green check, and failed
objectives with a red cross and an accessible failure label. These are derived
query results, not additional save fields.

Mission objective documents may also declare `loseConditions`, each with a
`label`, shared `condition` and defeat `message`. These conditions are evaluated
after actions and at turn boundaries and appear in the survival objectives.
For a chapter playable through round 30, use `{"kind":"round","round":31}`:
the loss occurs when round 31 begins, while an already completed victory remains
final. These rules are independent of the simulator's unresolved-round cap and
do not supply deadline hints or urgency scoring to AI planners.

Objectives may use `visibleWhen` with the shared Rust condition format. Hidden
objectives remain victory requirements, but are omitted from the briefing and map
markers until revealed; campaign planning also respects that discovery gate.
`hasUnits` tests for an existing live unit by side and optional name/type without
requiring a separate proximity visit. Encounter events can share their durable
marker across dialogue, pending spawns, lights, group activation and objective
visibility. Keep terrain can be claimed by moving onto it.

Rituals can declare a conditional `opening: {when, sites}`. At chapter entry, the
normal ritual effects consume only those source tiles, preserving their spent
terrain, losses and raised-unit identities in the generated state. Later regular
tolls act on the remaining sources. Opening conditions use persistent campaign
choices; no historical gameplay is reconstructed during save loading.

An optional positive `sourcesPerToll` limits source activations on a regular toll.
Active source tiles resolve in native map order, followed by queued death charges
within the remaining budget. Empty sites still spend their scheduled activation;
rescued captives do not cause the next site to fire early. Unprocessed charges
remain queued. Conditional openings use their explicit site list. Spent terrain
and existing activation/spawn marks preserve progress across saves; omitting the
limit retains simultaneous resolution.

Dialogue lines may declare an optional `sound` family. The dialogue view plays
that cue when the line is first presented through the gesture-unlocked sound
player; ordinary sound preferences still apply.
