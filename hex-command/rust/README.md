# Native simulation core

`hex_command_core` owns battle state, movement, combat, healing, XP, promotions,
recruitment, campaign events/carryover, objectives and classic/rush/mixed decisions.
`hex-command-sim` reads one arena or campaign configuration per stdin line and
writes its complete result as one JSON line. It stays alive between matches.
There is no browser or WASM integration in this port.

Run commands from the project root, with Node dependencies and Rust installed:

```sh
npm run ai:rust:build
npm run ai:simulate -- --mode arena --seeds 2 --out artifacts/native-arena.json
npm run ai:simulate -- --mode campaign --seeds 2 --out artifacts/native-campaigns.json
npm run ai:full -- --out artifacts/native-full.json
```

The existing Node CLI schedules matches, limits workers to the available CPU
quota, and writes indexed journals and aggregate reports. Each worker sends whole
matches to a persistent native process. All decisions and turns run inside Rust;
there are no calls back into the TypeScript game during a match. Reporting and
replay keep the existing schema, including every action, combat and endpoint state.

`npm run ai:rust:build` exports the authored TypeScript unit/race/terrain rules,
map layouts and campaigns into `data/rules.json`, then compiles the native program.
Do not edit that generated data by hand. The executable embeds the export's source
fingerprint; the CLI rejects it after rule, native source or exporter changes.
The TypeScript reference remains available through `npm run ai:typescript -- ...`.
Simulation batches still require an explicit user request.

Use `npm run ai:rust:benchmark -- --out artifacts/rust-sample.json` for a bounded
60-match sample: all races/maps/policies and four campaigns. It compares every
result field with the TypeScript reference, excluding JSON object key order, and
reports both timings. `--alternate` changes the seed, gives the arena enemy first
initiative, mixes opposing policies and uses pragmatic campaign diplomacy.
`--skip N --limit N` selects fewer matches while fixing a mismatch. Benchmarks are
separate from unit tests and never launch the full tournament.

`npm run check:rust` runs focused rule/transition tests. `npm run format` and
`npm run style` include Rust formatting; `npm run verify` includes the Rust checks.
The Cargo lockfile pins dependencies. Geometry, route fields and attack origins
are reused across forecasts; movement uses integer queues and candidate selection
retains stable ties without sorting the complete plan list.
