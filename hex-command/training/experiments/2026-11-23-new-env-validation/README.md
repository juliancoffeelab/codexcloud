# Validated new targeted encounters

Three Sol agents implemented, tuned and completed native simulations of the
proposed commander, route and combat encounters. This directory records actual
results, not proposal-only feasibility. No training, sampler or playable catalog
was changed. The validated fixtures are isolated simulation data ready for a
separate integration decision.

## Fresh confirmation

Each row uses 20 matched fresh seeds per player, separate from its discovery
cohort. All commander and route conditions below use Classic opposition,
unrestricted diplomacy, fixed authored rosters, zero recruitment and clear vision
on both sides. Human units face skeleton enemies. Ordinary native references use
the same observations and legal actions; no custom oracle or frozen enemies.

| Encounter | Hour8 | Hour10 | Full | Classic | Recommendation |
|---|---:|---:|---:|---:|---|
|Commander Left Behind,25HP,turn22 deadline|0/20|2/20|20/20|18/20|Challenge candidate|
|Commander Left Behind,7HP+healer,turn13 deadline|0/20|6/20|13/20|18/20|Challenge candidate; teacherClassic|
|Scree Relay route-choice,blocked road|0/20|0/20|20/20|Not in confirmation|Challenge candidate|
|Commander Left Behind,7HP only,turn22 deadline|0/20|0/20|1/20|13/20|Harder diagnostic control; reference less reliable|
|Scree Relay,threat on bypass instead|0/20|20/20|20/20|Not in confirmation|Counterexample/evaluation control, not a new unsolved Hour10 task|

Scree's drafted ranger relay/handoff was removed because it failed reference
screening. The confirmed encounter is a visible-at-start fork with a living
captain exit requirement, not a demonstrated in-battle threat-change/handoff
exercise. The blocked version expires at the start of player turn8; Full arrives
on5. The commander scene is17×12; route scene18×11. All HP, deadline and healer
changes are recorded as distinct variants, not conflated as pure health controls.

The three commander variants comprise240 fresh games; two route variants120;
three combat conditions240. All600 confirmation games completed with no harness
errors or simulation caps. Authored deadline failures count as defeats, not
truncations. Tuning games are preserved separately. Fresh seeds share the final
layout: there is no unseen-layout or training-transfer claim.

## Observed behavior

Commander seed491231001: Hour10 diverts Nera alone around the southern bypass,
loses the three forward allies, reaches the exit with the required post still
enemy-owned, then dies. Full uses the same broad flank with surviving allies and
wins. This validates a separation/coordination failure, not simply idle behavior.
The wounded witnesses demonstrate successful reversal/regrouping; healing is
not established as universally required. All36 sampled detailed commander replays
match evaluator winner and terminal round, with required post ownership and
living commander arrival checked in sampled wins.

On the blocked route, Full enters the bypass and wins20/20. Both learned
checkpoints never enter that lane20/20; Hour10's ranger survives20/20, but the
captain expires before exit. With the dreadguard moved to the bypass, Hour10 and
Full use the direct lane and win20/20. The control guards against teaching
unconditional reversal. All120 route confirmation replays match the frozen
native evaluator at terminal winner and round.

## Rejected: Kiln Crossing

The original heavy roster was infeasible for references, so it was simplified and
tuned. The apparent discovery gap did not persist on fresh confirmation:

| Classic opposition; radius2 fog variants | Hour8 | Hour10 | Queued | Full |
|---|---:|---:|---:|---:|
|Player-only fog|14/20|11/20|13/20|9/20|
|Both sides fog|14/20|16/20|15/20|8/20|
|Full vision|9/20|9/20|9/20|2/20|

This is not a useful demonstrated learner weakness relative to the ordinary
references, so it is excluded from the shortlist. Twelve terminal-matched traces
are retained; observed Hour10 losses were commander deaths, and sampled cases
withdrew rather than charging. Do not call lost contact or premature charging the
proven mechanism. Juniper Works was not implemented or evaluated.

## Artifacts and reproduction

- `fixtures/`: exact confirmed commander and route stage documents, unchanged
  from their evaluated catalogs.
- `commander/validated-catalog.tar.gz`, `routes/validated-catalog.tar.gz`,
  `combat/validated-catalog.tar.gz`: full final catalogs for native replay.
- `confirmation-summary.json`: independently recalculated counts and all seeds.
- Per-agent summaries/protocols/builders/raw compressed results: discovery and
  fresh confirmation are distinguished. Earlier tuning catalogs are not all
  frozen here; final confirmation catalogs are.
- `routes/v8-confirmed-traces.tar.gz`, commander trace JSONL gzip and combat
  paired-mechanism trace gzip retain behavior evidence.
- `gallery.jpg` and `renders/`: native-artwork battlefield previews. Route
  previews add signpost art connection metadata to the render-only map copies;
  this does not alter the tested gameplay fixtures.
- `manifest.json`: archive file sizes and SHA256 hashes.

Models/native are the archived hour8 and hour10 planners and frozen native
binding from `tmp/rl/ppo-from-pretrain-5m-9h-20261123`. Exact model hashes and file
paths appear in per-agent protocol/summary files. The current engine fingerprint
matches the frozen evaluator; detailed replays were outcome/round checked.
No reward shaping, AI paralysis, new training or automatic integration was used.
Only the observed scopes are qualified: clear both-side vision versus Classic
for accepted fixtures. New fog/opponent conditions need their own qualification.
