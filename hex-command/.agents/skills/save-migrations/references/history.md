# Save migration history

Version-to-version save conversions. Consult the relevant
section when changing or investigating a historical conversion.

## Version-one migration

Both previous formats were version one. TS snapshots have a top-level `version`
and store ownership without terrain. Native envelopes have `engine: "rust-wasm"`
and a version inside `saved`. The v1-to-v2 step distinguishes these formats and
removes their format markers. Historical layouts, defaults and campaign
conversions live exclusively under `packages/save-migrations/src/legacy/`.

TS saves tagged `campaignDeploymentFormat: "named-waves-v1"` already use authored
wave IDs. Untagged published saves from `3956f5c` and `7a30040` translate numeric
markers through each wave's explicit `legacyIndex`. Revision-one Lantern village
and Ashen finale saves already contained subsequently delayed reinforcements;
the importer marks those waves deployed without changing wounded troops, gold
or IDs. Revision-two saves retain pending waves.

The unpublished intermediate draft `1acc1dd` reused a revision with a different
Ashen numeric order. Its format cannot be distinguished reliably from an older
published save, so that ambiguous draft is not supported. Wave identity is never
inferred from surviving troop health.

## Version-two migration

The v2-to-v3 step removes the need for historical layouts in the live catalog.
Revision-one refuge checkpoints retain their own tiles, ownership and
units, while their revision tag is changed to the current catalog entry with the
same geometry. Normal Rust restoration validates the
result. The catalog accepts only current authored documents; old layouts, marker
indices and dialogue are stored under `packages/save-migrations/src/legacy/data/`.

## Version-three migration

The v3-to-v4 step grows the developer map's published 6×20 and 13×23 layouts to
the current 32×32 revision. Existing tiles, ownership, units, battle progress
and seed stay intact; only missing tiles are added from the current map.
It never resets units or respawns defeated showcase troops.
Malformed or incomplete historical boards still fail validation. Other maps
receive only the envelope-version update.

These historical intermediate conversions may carry an opaque `planner` field
through the migration chain. The v4-to-v5 step discards it before native
restoration. Carrying those bytes does not permit interpreting or resuming them.

## Version-four migration

The v4-to-v5 step drops the entire historical `planner` field, including all
queued actions, pending commands and continuation bookkeeping. It preserves
resolved battle state and the pre-turn replay marker. Migration never executes
AI actions or deserializes the historical planner. Normal gameplay creates a
fresh plan when needed.

Older native saves may contain resolved state halfway through an enemy phase.
They have no pre-turn state to roll back to: completed actions remain, while
unexecuted actions are discarded. Their converted data is persisted immediately
after validation, without playing any remaining enemy actions.
This restores a historical format, not permission to save partial phases.
New saves must persist only the boundaries of the entire enemy turn, even when
that turn contains many units and commands.

## Version-five migration

The v5-to-v6 step turns historical implicit type-based leadership into explicit
`role: "leader"` metadata on unit instances. It uses frozen pre-conversion type
IDs stored with the migrations and preserves every existing explicit role.
Current unit definitions have no leadership flag. Rust, AI, UI and current saves
use only the unit's role; ordinary units of the same type do not become leaders.

## Version-six migration

The v6-to-v7 step makes faction data explicit. A frozen conversion fixture supplies
The Silent Crossing's wildlife controller and assigns its existing animals to that
faction, preserving identity, wounds, equipment and progress. Other maps retain
explicit faction data or receive an empty list. Current snapshots always serialize
`factions`, including an empty list, and Rust rejects omitted fields or missing
required authored factions instead of backfilling from the current map.

Older conversion steps read migration-owned metadata frozen from deployed source
`14371387185340e64c5b68706faa409ebe16fdf8`. That fixture is historical conversion
data; do not regenerate it when current maps, unit rules or campaigns change.

## Version-seven migration

The v7-to-v8 step adds the required `groundItems: []` battlefield inventory.
Version-seven checkpoints had carried equipment but no ground items. Conversion
preserves all existing units, equipment and campaign progress. Current snapshots
serialize ground items explicitly, including an empty list; Rust does not invent
missing state during restoration.

## Version-eight migration

The v8-to-v9 step converts Toll Road revision-three checkpoints to revision four
using migration-owned historical encounter identities. It removes uncontacted
preplaced Sella, records an already-completed meeting’s spawn as resolved, and
preserves a joined Sella and every other existing patrol unit. Existing camp
ownership is retained. The old proximity light memory is discarded, swamp entry
memory is translated to the shared encounter marker, and the original mire
caster receives its authored name without changing identity or progression.
Other chapters receive only the envelope update. No runtime restoration backfill
or live catalog lookup performs this conversion.

## Version-nine migration

The v9-to-v10 step writes previously implicit terrain orientations explicitly:
road, signpost, bridge and ford use `e+w`, villages use no connections, and landing/ferry
use `w`. Existing explicit connections are preserved. These are frozen historical
constants inside the migration. Current data, validation and rendering never
supply a missing connection mask.

## Version-ten migration

The v10-to-v11 step moves the published Bell chapter's revision-one/two battle
onto the valley using a frozen, injective coordinate mapping. It preserves living
units, casualties, wounds, experience, equipment, gold, round and resolved outcomes;
it never respawns the historical roster. The placeholder companion Meri becomes
Oren, including her existing progression. Sealed stones remain spent at their
mapped ritual sites. New fog and faction data are migration-owned frozen values.
Existing Mirror castle experiments return to the original terrain, and developer
castle cells receive their frozen complete-frame selections while retaining owners.
Current valley checkpoints and other maps retain their resolved state. This
conversion does not inspect neighbors or compose artwork.

## Version-eleven migration

The v11-to-v12 step corrects the valley opening. It removes the two unearned
starters only from unfinished turn-one battles where their published IDs, types,
positions and untouched stats still match. Recalled troops and units that acted,
earned experience, took damage or received equipment remain. Obsolete opening
dialogue is removed with its cursor adjusted; a present Oren supplies the monastery
lead once if the ritual has not started. Every constant belongs to this historical
conversion; it never queries current content or changes terrain.

## Version-twelve migration

The v12-to-v13 step corrects the published church reward to one enchanted mail
shirt. It removes the unintended chapel helm from ground inventory, active units,
veterans, story veterans and evacuated units in Lantern Road saves. Existing mail
keeps its mechanics and receives the corrected name and description; queued church
discovery text describes one item. Other battle progress remains intact. Constants
are frozen inside the conversion and do not query current content.

## Version-thirteen migration

The v13-to-v14 step installs three keep-side villages in ongoing valley battles,
using frozen coordinates and tile fields. Only the published plains sites change;
fog and all unrelated battle state are preserved. The villages begin enemy-owned,
except when a player unit already occupies one. Existing village ownership is not
reset. A player occupant on the newly capturable bell keep claims that tile too.
The obsolete courtyard garrison tag is removed so these units use ordinary AI.
Other maps pass through unchanged.

## Version-fourteen migration

The v14-to-v15 step assigns the living enemy Tollkeeper in Stage 1 to his
conditional hold group, replacing the old permanent leader post. It preserves
units, turns, completed events and all other battle progress. Other maps pass
through unchanged. The conversion uses only frozen assignment constants.

## Version-fifteen migration

The v15-to-v16 step corrects the three published keep-side villages in the Bell
valley from green-plain to mountain-rock. Only their floor changes; ownership,
fog, lights, units, turn, victory and all other progress are preserved. Coordinates
and prior values are frozen in the conversion. Other maps pass through unchanged.

## Version-sixteen migration

The v16-to-v17 step replaces castle connection/corner codes with explicit walls
and gates. It uses the frozen clockwise direction order and published gate rule,
removes the obsolete fields, and preserves ownership, fog, units and all battle
progress. It does not import current content or execute gameplay.

## Version-seventeen migration

The v17-to-v18 step retires dedicated castle gate terrain and its aperture fields.
Gate cells become ordinary castle courtyards with their existing wall directions;
the former gate edge stays open. All castle cells lose the obsolete gates array.
Ownership, fog, units, equipment, turns, victory and other battle progress remain
unchanged. The conversion uses only frozen version-seventeen field contracts.

## Version-eighteen migration

The v18-to-v19 step installs the northern hills and relocated Tollkeeper keep in
revision-four Villages saves, using a frozen tile delta. It advances that map to
revision five and preserves unit positions, health, equipment, gold, turns,
completed events and outcomes. Keep ownership transfers from the former keep,
unless a unit already occupies the new keep. Other maps pass through unchanged.

## Version-nineteen migration

The v19-to-v20 step bends the eastern Villages road northeast and moves its
departure from the sign at (18,5) to the northern pass at (17,0), advancing map
revision five to six. Frozen tile deltas widen and ease the approach to the pass
while preserving ownership, units, positions, health, gold, fog, completed events
and outcomes. Other maps pass through unchanged. Departure objectives and script
conditions use the northern pass through the current scenario definitions.

## Version-twenty migration

The v20-to-v21 step converts phase economies to faction economies. Existing
player/enemy balances and settlement owners remain unchanged; declared factions
receive empty independent treasuries. A frozen Villages revision-six tile delta
adds the marsh keep and funding villages and advances the map to revision seven.
Existing owners take precedence, followed by occupants on newly owned tiles.
A living Edrin gains the leader role and keep order; his faction receives its
explicit recruitment quotes. No units are spawned, revived or relocated, and no
gold is invented. The obsolete `group:swamp-dead` activation marker is retired
because that encounter is now unconditional. Health, items, progress, turns and
outcomes remain intact.

## Version-twenty-one migration

The v21-to-v22 step removes two added Villages settlements, restoring forest at
(13,7) and marsh at (16,10), and advances revision seven to eight. The conversion
uses frozen coordinates and terrain values. It preserves ownership, units, gold,
turns, equipment and campaign progress; existing armies are not disbanded and
treasuries are not reset to the new opening budgets. Other maps pass through.

## Version 23: tile-owned terrain lighting

Version 22 map emitters move onto their owning tiles using frozen light definitions
under `legacy/data/tile-lights-v22.json`. Remembered light keys change from list
index/coordinates to tile identity. Gameplay state and fog discovery are preserved;
rendered light positions are derived from tile coordinates.

## Version 24: explicit base factions

Faction rosters replace implicit player/enemy armies. The conversion appends frozen
base controller definitions only where absent, preserving saved units, treasuries,
existing faction definitions and campaign progress. Campaign recruitment continues
to use its resolved authored pools and completed recruitment events.

## Version 25: explicit diplomacy

The old engine treated player/enemy as hostile when neither faction declared
their relation. This conversion records that historical hostility on the existing
faction definitions. Explicit neutral, allied or hostile declarations on either
side remain unchanged. Units, gold and campaign progress are preserved.

## Version 26: campaign character identities

Frozen campaign registries map historical unit names and dialogue aliases to
character IDs. Units, veterans, evacuated units, companion-loss markers and saved
selector conditions retain their identity through conversion. Named dialogue
uses character IDs; current captions come from Rust inspection. Generic narrator
labels and existing prose/history remain unchanged. No gameplay commands run.

## Version 27: stable tile identities

Version 26 addressed board cells by coordinate strings. Version 27 gives each
current authored cell a stable ID independent of its position. The frozen
`legacy/data/tiles-v26.json` table converts saved tiles, unit positions, dropped
items, faction-transfer and atmospheric conditions, defended
site counters, remembered conditions, evacuated units and ritual/event markers.
It preserves coordinates, ownership, health and progress; conversion never reads
the current catalog.

## Version 28: general authored events

Frozen `legacy/data/events-v27.json` recipes convert Lantern Road's remembered
events, choices, holds, partial spawns and captive death charges into explicit
flags and active event state. Each death charge becomes its own structured mark.
Pending spawn members retain their placement area and item IDs; completed
members do not replay. Captive roles and obsolete faction transfer rules are
converted with the same frozen recipe. Other campaigns keep their old event
representation. Units, health, treasury and positions remain unchanged.

## Version 29: stage-stamped flag history

Flat campaign choices become per-flag histories of `{state, stage, turn}`. Frozen
`legacy/data/flags-v28.json` identifies known producing stages. The conversion
preserves current truth values, timestamps flags attributed to the current stage
at the saved round, and uses turn zero for older/unknown incoming facts. Previous
values and exact historical write turns were not stored and cannot be recovered.
Retry discards the current stage's positive-turn changes while retaining incoming
facts. Units, treasury, event state and positions remain unchanged.

## Version 30: unified campaign events and story

Frozen `legacy/data/events-v29.json` recipes convert the remaining campaigns'
remembered events, waves, ownership counters, vigil/refuge progress, named deaths
and recruitment unlocks into active events and explicit flags. Legacy diplomacy
mode names become `diplomatic` and `unrestricted`. Current-stage deaths are
removed from the incoming companion checkpoint so retry restores that chapter's
starting company. Completed objectives stay complete; impossible active objectives
receive explicit failure state. Units, health, positions and treasury are preserved.

## Version 31 on the objective branch: objective history

Version 30 derived objective completion from event retirement and visibility from
live conditions. The frozen chapter recipes in `objectives-v30.json` convert those
facts once into completed and revealed objective IDs. Existing failed IDs remain
unchanged. Current runtime saves own these facts independently of event activity.

Version 31 completed IDs retain achievement history. Ongoing objective satisfaction
is a runtime query over the restored world, shared with victory predicates; it is
not another saved field and does not require a format conversion. Restoring a
previously captured but currently lost fort preserves history without satisfying
the ownership requirement.

## Versions 31–33 on the saved-progress branch

This separately deployed family used version 31 for optional progress identity
and full-state undo, version 32 for compact undo changes, and version 33 for
adopted positions. Its native objective state still used the version-30 shape.
These numbers overlap the objective branch’s version 31; the envelope number
alone does not identify whether objective history has already been converted.

## Version 34: combined objective history and saved progress

The saved-progress 30→31→32→33 sequence retains its published meaning. The final
33→34 conversion recognizes objective history by its complete `completed` and
`revealed` arrays. Existing arrays are preserved; neither current flags nor event
retirement can replace already-recorded history. If both fields are absent, the
frozen `objectives-v30.json` and `objective-events-v31.json` recipes perform the
objective branch’s conversion once. Partial/malformed history is rejected.
No current catalog or gameplay commands participate in conversion.

Both version-31 families, later saved-progress checkpoints and earlier versions
therefore converge on one format. Native validation follows conversion. Run and
checkpoint identities, setup, seed, units, treasury and flag history are retained.
Older undo is discarded. Historical payloads migrate through the same loading
entry point as flash save; the history index remains unchanged.

## Version 35: current flash save and turn-only history

The 34→35 conversion makes setup and save identities explicit, supplies frozen
combat RNG and unit-experience defaults where absent, and discards old undo.
A current imported-position marker becomes flash save metadata; it is not a turn
start. Normal restoration neither fills old fields nor clamps saved hit points.

History index version 1→2 is a separate explicit data conversion. It keeps actual
turn-start entries and retires archived flash save and imported-position entries.
Their payloads are deleted after committing the converted index. Current flash save
lives at its separate fixed key and is not deleted. Missing metadata on an old
current save does not create historical records.

## Version 36 and history index 3: manual checkpoints

The explicit 35→36 and history-index 2→3 steps preserve existing checkpoint facts
and metadata. The target model adds explicitly requested manual saves with their
own identities and move undo. Automatic turn snapshots still contain no undo;
current flash save remains separate. No old representation is accepted by runtime
readers or writers. Manual saves are exempt from age-based automatic retention,
but explicit individual or campaign deletion has no exemptions.

## Version 37 and history index 4: one snapshot-only history format

The explicit 36→37 conversion removes the historical manual snapshot variant.
Both automatic and explicit checkpoints use turn snapshots with no undo field.
A required manual boolean identifies explicit saves for retention and display;
it does not change the payload contract. Current flash save keeps its undo. The
conversion drops its obsolete `parentId`, including preserved wire metadata,
because ancestry links belong to historical turn snapshots.
Index 3→4 makes the same metadata conversion. Older shapes are interpreted only
inside the version conversions; normal code accepts only the target model.

## Storage version 2: IndexedDB

`ocaml/Storage_migrations.ml` converts the localStorage flash save and history into current
flash save, metadata and snapshot records. It runs envelope/index migrations without
changing native facts or creating missing history. Native validation precedes the
single import transaction. The initialization marker commits with all records;
localStorage remains untouched until success. Initialized IndexedDB wins over
later localStorage writes, including writes from tabs running an older build.
Storage conversion changes the former `"recovery"` discriminator to `"flash"`.
Conversions from earlier envelopes also produce the current flash-save shape.
The native save envelope remains version 37; the IndexedDB database schema starts
at version 1, independently of save-envelope and storage-source versions.

## Version 38: coordinate addresses and named landmarks

The 37→38 conversion replaces opaque cell IDs with addresses derived from the
saved cells' own coordinates. Frozen map-scoped data adds optional landmark names.
Unit positions, ground items, fog/light memory, event marks, pending spawn areas
and backward changes retain their positions. The two Ashen seal event/flag IDs
receive readable names while preserving completion and flag histories. Only
declared reference fields change; prose, equipment strings and save identities
remain opaque. Undo reconstructs historical states and applies the same typed
conversion before regenerating backward differences. Surface
facts, treasury, units, turn, RNG and checkpoint metadata remain unchanged.

## Version 39: structured geometric addresses

The 38→39 conversion changes resolved unit, item and mark positions and pending
spawn areas into `{col,row}` objects. Shared conditions in fog, lights and faction transfers use explicit
`{location}` or `{hex}` references. Frozen version-38 and version-39 types define
the conversion. Backward data patches reconstruct historical states, which use
the same typed conversion before producing new backward differences. This covers
single-field patches, whole records and removals without guessing types from paths. Text,
cell presentation keys, IDs, flag histories and all other resolved facts retain
their meaning. IndexedDB storage and history-index versions are unchanged.

## Version-forty migration

The v39-to-v40 step makes explored terrain a faction fact. It moves recognized
`fog:col,row` entries from shared condition memory into each formerly viewing
human faction's `vision.explored` on maps with fog, and replaces player selectors inside terrain
fog predicates with the exact `observer` selector. Lights, faction transfers and
other conditions keep their previous selector meanings. Mirror Night's AI
factions gain independent fog with no invented exploration history. Other enemy
factions retain their authored absence of fog. Exploration is never shared just
because factions act in the same phase or are allied.

The conversion preserves units, inventories, economy, campaign outcomes, PRNG and
seed. It reconstructs and converts every undo state before generating replacement
patches, so undo does not restore shared fog or erase unrelated facts. Both the
frozen transition and the independent native/browser output contracts are typed.
Frozen type names are pinned in the reader metadata so a new, shorter reference
path cannot rename declarations used by earlier conversions.

## Version 41: shared perception and turn phases

The 40→41 conversion makes vision-group membership explicit on factions, moves
exploration into those groups, and converts tile fog into bounded concealment
facts. Each formerly independent faction starts in its own group; the conversion
does not infer sharing from diplomacy or scheduling. Recognized observer
proximity predicates retain their reveal radius. The authored monastery's
`fog-cleared OR proximity` expression resolves already-cleared fog immediately;
unsupported historical custom fog expressions fail conversion explicitly rather
than silently changing their meaning.

Units retain an explicit faction and gain saved combat/contact reveals and
perception effects. Scheduling values become `player`/`ai`, battle results become
`playerVictory`/`playerDefeat`, and the action-budget field becomes `attackSpent`.
Inventories, identities, HP, ownership, treasuries, combat RNG, campaign flags and
checkpoint metadata retain their resolved facts. The same typed conversion
reconstructs and converts every undo state before regenerating backward patches.
The full v41 wire schema and readers are frozen inside the migration package;
independent application/native compile-time contracts check its output.

## Version 42: battle-owned player AI selection

The 41→42 conversion adds required `setup.player`: `null` for manual control, or
one of the three browser policy identifiers. Existing battles migrate to manual
control. AI control belongs to each saved battle's setup, survives reload and
campaign chapter transitions, and is independent of the camera preference.
Native checkpoint types remain the frozen v41 graph; checkpoint and undo data
pass through unchanged. The browser contract comparison also checks this field.

## Version 43: cave terrain roles

The 42→43 step extends the frozen terrain-role enum with `cave-wall`, introduced
by the cave terrain integration. Older role values widen without changing their
wire values. Typed conversion visits affected unit effects, carried and ground
items, evacuated units and veteran inventories. All other resolved facts,
including undo patches and saved control settings, pass through unchanged.
The independent native contract is frozen as v43; v41/v42 inputs remain frozen.

### Version 44

Concealment rules carry explicit physical requirements, shared by terrain and
personal sources. The isolated v43 conversion records the old illumination
condition on darkness rules and moves personal terrain eligibility into
`requires.terrains`. Undo states pass through the same typed conversion and are
re-diffed. Other battlefield facts and browser preferences are retained. Unit
appearance is derived and is not part of the save.

### Version 45

Campaign opponents use named faction IDs. The frozen 44→45 conversion renames
the former `enemy` faction according to its saved map, including unit membership,
treasuries, diplomacy, settlement ownership, transfer targets, pending spawns,
recruitment overrides, evacuated units and exact ownership conditions. Vision
groups, palettes, phases and relation selectors retain their previous meanings.
Undo states receive the same conversion and are re-diffed. The native wire layout
is unchanged and reuses the frozen v44 checkpoint graph.
