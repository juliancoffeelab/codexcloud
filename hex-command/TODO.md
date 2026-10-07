# TODO

## Design

- [ ] Review inconsistent low-poly artwork with the user before replacing designs.
- [ ] Clarify Dain's allegiance in the skeleton campaign, preserving the
      immortality/service exchange and the undecided future conversation.
- [ ] Clarify why lighting the Lantern Road lanterns forces the necromancer away.
- [ ] Review remaining optional quest rewards with the user.

## Balance and simulations

- [ ] Finish the balance pass with matched campaign and arena trials, preserving
      failed attempts and recording actual commands.
- [ ] Run updated comparisons when explicitly requested. Keep campaign and arena
      rosters, scenarios and results separate.
- [ ] Review veteran boar attacks separately after the rookie balance changes.

## Merfolk campaign

- [ ] Consider terrain-aware shoreline sprites.

## Data loading and unit information

- [ ] Make native Rust load game data from disk by default at the start of each run.
- [ ] Keep WASM as the exception: bundle its game data.
- [ ] Make TypeScript ask WASM for game data.
- [ ] Move attacks, resistances and other mechanical stats out of unit descriptions
      into their structured data and UI fields. Add a separate list for unit/race
      terrain buffs; consider a two-column table for terrain effects.

## Tests

- [ ] Move gameplay rule tests into Rust: movement, combat, recruitment, economy,
      healing, progression, campaign objectives and native save validation.
      Preserve behavioral coverage and remove tests that only repeat JSON values.
      Keep JS migrations/storage, controller, undo, UI publication, rendering and
      audio tests in Vitest, plus a small set of real WASM integration tests.

## Mobile rendering

- [ ] Investigate and fix mobile stray lines and portrait leakage that remain
      after replacing atlas cropping with isolated frame images.
