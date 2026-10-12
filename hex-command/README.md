# Hex Command

A client-only hex strategy game with recruitment, terrain combat, unit progression
and four story campaigns. Svelte and TypeScript provide the interface; Rust/WASM
runs battles and enemy planning. The game uses pixel artwork and requires no
account or server.

## Development

Use Node.js 22.12+, Rust stable with `rustfmt`, the `wasm32-unknown-unknown` target,
and Python 3 with Pillow for artwork. `wasm-pack` is pinned in npm dependencies.

```sh
rustup target add wasm32-unknown-unknown
npm ci
npm run dev
```

Development prepares WASM bindings and artwork. Common commands:

```sh
npm run format                # Apply project formatting
npm run wasm:build            # Prepare bindings before a fresh test run
npm test                      # All test suites; shared 20-second budget
npm run verify                # All checks and production build
npm run verify -- types tests # Selected checks
npm run build
npm run preview
```

## Playing

Capture villages for income and recruit while your leader occupies a keep.
Ordinary recruits act on the next turn. Use terrain and damage types to counter
the enemy; combat grants experience and eligible units can advance. Campaigns
provide their own objectives and let survivors be recalled between chapters.
Saves and sound preferences stay in the browser. The developer map displays
every terrain and both sides of every playable unit rank.

## References

- [Development, project layout and publication](docs/development.md)
- [Rust API and build guide](rust/README.md)
- [Authored content](rust/data/README.md)
- [Save formats and migrations](docs/save-formats.md)
- [Combat simulations and analysis](.agents/skills/combat-simulation/implementation.md)
- [AI policy implementation](rust/src/ai/README.md)
- [Artwork rendering](src/presentation/README.md) and [terrain sources](docs/terrain-artwork.md)
- Artwork skills: [generation](.agents/skills/pinkbox-generation/SKILL.md),
  [enhancement](.agents/skills/pinkbox-enhancement/SKILL.md),
  [cleaning](.agents/skills/pixel-cleaning/SKILL.md),
  [lineup previews](.agents/skills/pixel-lineup/SKILL.md)
