# Development and publication

Standing project instructions live in [AGENTS.md](../AGENTS.md), which also
indexes the task-specific skills. This document provides setup and command references.

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

## Browser engine builds

The Rust workspace separates the current catalog models and host-side authoring
loader from the battle engine. The build validates authored JSON and derives
static catalog tables from those same models. Browser factories materialize their
owned runtime catalog from the tables; they do not parse the authoring documents.
Native catalogs continue to read the selected data directory. Custom browser
catalog imports require the opt-in `runtime-catalog` feature.

`wasm:build:release` uses the dedicated `wasm-release` size profile, then a pinned
Binaryen pass deduplicates code and groups similar function bodies before removing
diagnostic names. Development builds retain names. Compare the WASM module and
its referenced JavaScript loader/helpers together when measuring download size.
The native simulation profile is independent of these browser build settings.

## AI and battle ownership

`rust/battle` owns mechanical battle state and command execution. The host owns
its opaque `BattleSession` and borrows one catalog. Evaluators receive a borrowed
`BattleView` for mechanical queries. They cannot turn that view into mutable
state or execution access. Authored campaign observations use `CampaignView`.

Forecasting is explicit: `ActionForecastSource` grants mechanical command
simulation; `PhaseForecasts` grants named phase projections and its matching
source view. A `PhaseSnapshot` grants observation only. Automatic recruitment is
runtime orchestration, including recruitment after an accepted keep move;
recruitment capacity forecasts use the shared estimate and never run a recruiter.

Each planner owns its algorithm and mutable session. Common runtime code handles
commands, budgets, boundaries and continuation. Strategic preparation is an
explicit adapter subscription. Shared decisions live in `ai/policies` under
named domains with explicit inputs. Any planners may deliberately share a policy;
changing one should make its consumers apparent at call sites. Add new strategic
behavior to a domain or adapter rather than extending the mechanical facade or
adding a silent common-driver default.

## Event runtime

The engine prepares an immutable mission together with its clock descriptors.
Each battle's private event progress binds that plan to numeric counter indices;
queries cannot assemble a counter layout separately. Clock updates run before
boolean condition evaluation and independently of short-circuiting. Missing
counters initialize on their first observation without counting that observation.

Evaluation takes ownership of shared progress, then returns it once all cascade
passes finish. A quiet forecast keeps its progress shared; mutation detaches only
when another battle still shares it. Active-index scans borrow immutable shared
lists, retirement changes mask bits, and compaction follows the scan. Predicate
and visibility contexts contain explicit facts, not a game back-reference;
current event progress is supplied separately to event predicates.

Saved event IDs, predicate numbers and progress remain the durable contract.
Prepared descriptors, numeric slots and index lists are runtime data, rebuilt
when entering or restoring a chapter. Restore prepares the mission before
resolving active-event and counter indices, including when no cache exists.

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
or `night`). Terrain emitters are authored on the owning tile as `light`, beside
its optional `fog` rule. A light inherits the tile's coordinates; it has optional
`dx`/`dy` offsets, positive `radius`, nonnegative `power`, optional RGB `color`,
and an optional `fixture` flag. Do not duplicate tile coordinates or keep a
separate map-level emitter list.

A light's optional `when` contains a shared `condition` and `remember` boolean.
A tile's `fog` uses that same condition wrapper and specifies what clears it.
Owner/proximity conditions can omit `tile` to use the owning tile ID. Proximity
uses hex distance (`radius: 0` means occupying the tile). `remember: false`
follows current state; `true` retains satisfaction under the stable `light:<tile>`
or `fog:<tile>` identity. Event completion is already remembered by campaign state.
Visibility queries cannot define lighting or fog conditions recursively.

Rust validates these definitions and resolves active lights to positioned render
sources, shared by initial catalog previews, devtools and live engine views.
The rendering worker paints soft fog; the UI hides covered enemies and Rust
rejects attacks on covered enemies. Remembered facts persist in `conditionMemory`;
resolved masks and rendered images do not. Missing lighting defaults to daylight.
Dynamic ritual death lights remain positioned runtime sources, separate from
terrain emitters. Color transforms and glow rendering remain presentation details.

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

All runtime map painting happens in one persistent module Worker using WebGL 2.
The main thread creates a visible canvas, transfers it once with
`transferControlToOffscreen`, and sends scene/lighting/fog updates. The worker
selects complete authored tile frames and renders terrain, lighting and fog into
that canvas. The browser composites the worker-owned canvas directly; runtime
never exports, transports or swaps an
ImageBitmap per update. Main-thread replies are small revision acknowledgements.
The same GPU context draws units, items, health bars, ownership badges and tile
outlines from a presentation-only snapshot. Complete native 64×64 unit/item images
are packed into nearest-sampled GPU atlas pages; batches share one vertex format.
Unit sprites sample the same colored lighting function as terrain, replacing
per-tile SVG color filters. Transparent SVG hexes retain click targets, keyboard
navigation and accessible labels. Drag panning and Control/Command-wheel or
plus/minus zoom transform this shared canvas/input surface while retaining native
terrain, lighting and fog caches.
Zero resets zoom.

Production prebuilds unlit terrain PNGs, independent of light state. The worker
uploads a prebuilt image directly to WebGL 2. Changed/custom maps draw complete
frames into a GPU target in authored order, retaining offline anchors, native
scale and forest brightness. Terrain textures are decoded with createImageBitmap;
no runtime Canvas 2D context or CPU pixel readback participates in map painting.

Separate GPU targets retain unlit terrain, the lit background and blurred fog.
Changed regions are composed directly into the visible, retained drawing buffer
(`preserveDrawingBuffer: true`). This on-demand canvas keeps its pixels between
updates, idle tasks and scrolling; there is no final full-map framebuffer copy.
Changed lights update their old/new influence bounds and intersecting fixtures.
Fog edits update the affected mask region, expanding each
Gaussian pass by its kernel reach. Geometry is cached, and reordered identical fog
sets skip painting. Terrain and day/night changes still require full updates.
Terrain lighting uses integer spatial cell lists, so each fragment checks only
lights whose influence reaches its cell. This preserves overlapping light colors
without requiring floating-point blending extensions. Selection, health and unit
updates redraw GPU overlays while retaining terrain, lighting and fog caches.
Replaced terrain loads are aborted; pending requests for identical terrain reuse
the existing load. Superseded jobs cannot draw after a replacement
request is known to the worker.

The worker animates movement, hit recoil, appearance/disappearance and action
feedback only while transitions are active. Animation frames restore their dirty
region from cached terrain/fog before drawing live visuals, then stop when idle.
Reduced-motion preferences disable transitions. Atlas pages reset on scene/session
changes; sprite fetch/decode failures use the existing diagnostic/retry surface.
Stationary units, indicators and ownership badges retain their geometry and GPU
buffers during animation. Contiguous stationary runs stay in painter order around
moving units; only animated runs rebuild/upload vertices each frame. Scene,
inspection, pulse and transition-membership changes invalidate these caches.
New fog coverage removes existing disappearance fades, including fog-only updates.

Workers, transferable OffscreenCanvas, worker WebGL 2 and createImageBitmap are
required browser capabilities. There is no main-thread map painter, separate fog
canvas, SVG terrain substitute, bitmap export path or compatibility renderer.
Image/script/worker/GPU failures show diagnostics and retry. Retry transfers a
fresh canvas because a canvas cannot be transferred twice. The previous display
and its worker remain until replacement is ready; unrecoverable GPU/context loss
can invalidate that display. Disposal terminates active and retired workers and
removes their canvases. Offline builders may use their own canvas implementation.

Startup queues required shader compilations together and overlaps them with image
fetch/decode. Shared vertex shaders are compiled once per context. Daylight uses a
small copy program; fog programs and render targets are lazy. Night fixture
programs are prepared before play so first activation does not link new shaders.
Prebuilt PNGs upload directly into the terrain texture, with straight alpha
normalized by the lighting/copy shader rather than a separate sprite pass. Those
shaders also convert top-first image rows to framebuffer coordinates.

The worker retains up to two decoded prebuilt images within a 4,194,304-pixel
budget. Larger images are used without retention. Eviction closes image bitmaps;
GPU textures and buffers remain scoped to the worker. Tile-frame decoding still
closes its temporary images after upload.

Before entry, map preparation can render into one detached, worker-owned canvas.
Acquisition moves that same canvas into the battlefield and rebinds callbacks;
it does not create another worker, context or texture upload. This spare renderer
expires after two minutes and is discarded on page hide. Existing live renderers
only prepare future shader programs, preserving the current display. Initial
application setup prepares the actual Rust-published view; map pickers prepare
catalog views. Preparation failures are speculative: actual entry retries using
the normal diagnostic/retry path. Preparation time remains real work, even when
performed before the user enters a map.

Use `npm run map:benchmark -- --headed` on a GPU-enabled machine to measure the
Stage 3 renderer; `--software` explicitly selects SwiftShader for diagnostics.
Normal diagnostic mode acknowledges drawing without a forced completion fence.
Completed mode waits for a GPU fence, including polling overhead. Per-pass GPU
timer queries are asynchronous and optional. API wall times can include driver
synchronization, are nested, and are not GPU execution times. Normal readiness,
completed GPU work, a main-thread animation-frame opportunity and physical display
scanout are distinct events. Browser benchmarks stay outside the default tests.
Game logic and physical display scanout are outside renderer timing. Unit overlays
are included when `--units` is supplied; no timed run reads CPU pixels. Software timings do not establish a hardware budget.

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

Optional GPU benchmark (outside the default tests):

```sh
npm run map:benchmark -- --lights=128 --units=64
# Diagnostic software GPU when hardware acceleration is unavailable:
npm run map:benchmark -- --software --lights=128 --units=64
```

The light/unit counts add a synthetic presentation workload on the Bell map;
they never change playable content. Completed acknowledgements include a GPU
fence; software results do not establish hardware frame rates.

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

Rust inspection publishes required, optional and survival objective status. Saved
`completed` IDs record permanent achievement history, published as `achieved`.
By default this history also completes an achievement objective. An ongoing
requirement instead authors `satisfiedWhen`, a pure current-state predicate;
`progress.done`, progress counts and markers then reflect current satisfaction,
not past achievement. Losing ownership or leaving a required destination can make
it unmet again without erasing history. Events use `{"objectiveSatisfied":"id"}`
to reference that exact rule for victory and completion. Unknown or cyclic
references fail catalog loading. Satisfaction is recomputed from the resolved
checkpoint, not saved as another cache or reconstructed from event retirement.

A remembered event with an objective records achievement when it fires;
`completeObjective` and `failObjective` can address another task by ID. Explicit
completion does not retire another event or weaken its victory requirements.
An ongoing task may subsequently fail while retaining its achievement history;
a completed permanent achievement cannot be downgraded by a later failure.

Every objective explicitly authors `visibleWhen`. Revelation persists even if
its original predicate ceases to match. `availableWhen` additionally controls
current availability; completed achievements and failed tasks have no remaining
planning work. Ongoing requirements remain available while satisfied so guard
duties can continue and lost ownership can become actionable again.
Both availability and `satisfiedWhen` accept state predicates
only: use flags, ownership, occupancy or `turnAtLeast`, not tick-only `turn`,
`killed`, `left`, chapter-entry signals, timed counters, or path traversal.
Proximity/occupancy state queries must use `through: false`.

Campaign runtime alone evaluates those conditions. Campaigner's capability exposes
only revealed, available objectives as resolved intent, selectors, destinations
and current satisfaction. Recruitment, transition-sensitive orders, territory,
terminal heuristics and ordinary targets consume this same permitted view.
It exposes no raw mission, scripts, predicates or full campaign catalog. Runtime
also resolves enemy-group activation/hold orders and arrival facts. An aggregate
readiness flag prevents terminal shortcuts when requirements are still undisclosed.

`playerMarker` is optional and defaults to no outline. Use `{"tiles":["tile-id"]}`
or `{"units":{"character":"character-id"}}`. Runtime filters both kinds through
current fog; `visibleInFog: true` is an explicit opt-in for a deliberately revealed
marker. Markers are suppressed for unavailable, failed or currently satisfied
objectives. AI intent and approach radii never generate markers. Both required
and optional tasks use the same policy.

Deadlines remain authored defeat events. Rust does not infer public schedules or
format countdowns from event conditions. Dialogue, briefing and hints disclose
urgency only at their authored discovery points. The briefing stays collapsible;
no separate inferred notice is displayed outside it.

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

## Live performance traces

Settings → Debug tools provides **Start recording trace**, **Stop recording**
and **Download trace**. Starting closes Settings so the battle can be exercised;
reopen Settings to stop/download. Downloading during capture first stops and
flushes worker data. Recordings stay in memory until reload; starting another
recording replaces the previous one. Nothing is uploaded automatically.

The bounded Chrome/Perfetto JSON trace includes action and engine timings,
browser callback intervals and long tasks, map-worker stages, render workload
counts, optional asynchronous GPU timer queries, renderer information and the
start/end battle state. Worker clock origins are normalized to the recording.
Recording stops after ten minutes or 250,000 events; the export reports dropped
events and worker flush failures. GPU disjoint/incomplete samples are identified;
GPU duration tracks use CPU submission timestamps for approximate placement.
Recording never inserts completion fences. Callback intervals are not physical
display frame rates, and API submission time is not GPU execution time.

**Start Mirror Night** in the same debug panel starts a single-battle test
scenario using Classic Mirror's terrain and armies. It also appears in the
campaign picker because the shared campaign event system owns persistent death
marks. Player proximity clears fog permanently within two hexes. Player-owned
villages emit light until ownership changes; both keeps have fixed lamps.
Every death creates its own persistent red-light mark, including repeated deaths
on one hex. The usual commander victory/loss conditions end the battle. No test
scenario conditions are implemented in frontend rendering.

### Sprite preparation and outline resolution

After the first battlefield frame, the worker fetches, decodes and uploads the
published factions' recruitment pools, visible archetypes in each active palette,
and their Rust-catalog advancement branches in the background. Currently published
promotion choices are also included. Breadth-first traversal prepares base types
before deeper tiers, deduplicates shared artwork and stops at 128 unique URLs per
prefetch list. This prepares images only; Rust still decides legal choices and
concealed actors are not queried. Three speculative requests run at once.
Actual use shares an in-flight request or an uploaded atlas
entry; background preparation never gates a ready frame. Scene changes abort old
requests, and a failed speculative load is retried/diagnosed when actually needed.
Every newer frame releases the superseded frame's sprite wait immediately, even
if a decoder has not settled. Requests no longer needed by the latest scene or
its prefetch pool are aborted; still-needed URLs share their existing request.
An immediate recruit can still wait if its image has not finished loading.
Trace image events identify the asset and required/prefetch purpose; atlas-upload
events distinguish decoded images from artwork ready to draw.
Objective outlines preserve the upstream opaque gold dashed foreground over a
dark backing, blue selected/reachable variants and green recruitment variant.

Native terrain, lighting and fog caches keep their original resolution and pixel
scale. The final canvas instead follows the displayed size and device pixel ratio,
with antialiasing for geometric overlays. Its display buffer is bounded to
4,194,304 pixels and the device's renderbuffer/viewport limits. Resize/zoom can
refresh that buffer without rebuilding the native caches. Texture sampling remains
nearest for pixel artwork. Hex strokes use analytic edge coverage in a separate cached GPU vertex buffer,
with upstream widths, opacity, colors,
continuous dash lengths and miter joins; there is no added gold hover outline.
