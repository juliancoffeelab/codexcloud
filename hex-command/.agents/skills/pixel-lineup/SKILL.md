---
name: pixel-lineup
description: Use for previewing or comparing Hex Command faction lineups and unit upgrade trees or grids.
---

# Pixel lineup

A lineup is a faction. This skill previews its units and progression tree.

Run from the game root with Python 3 and Pillow. Read the requested tree in
`pixel/art/units/lineups/<race>.json`.
Render current matrices directly; do not rebuild artwork or replay recipes.

```sh
python3 .agents/skills/pixel-lineup/scripts/preview.py --list
python3 .agents/skills/pixel-lineup/scripts/preview.py \
  --race humans --layout tree --side enemy --output tmp/humans-lineup
```

Prefer `--layout tree` for faction lineups: show each shared L1 once, centered
beside its L2 branches, with arrows connecting L1 → L2 → L3. Commander tiers
precede the unit families. Use `--layout grid` for the old row-per-branch view.
All frames stay 64×64 at one integer nearest-neighbor scale, including cavalry.
Do not crop, resize individual units, shift their baseline or pad them again.
Use the requested faction, otherwise the last used faction or enemy. Show both
only when requested. Use `--help` for display options. Tree pagination keeps each
family and its connectors together, so a large family can exceed the requested
row limit. Hiding boxes and names leaves tree arrows visible. Outputs stay in `tmp/`.

Open the generated PNGs and show them to the user. Keep the JSON source/hash
report internal unless requested. This tool never writes source matrices,
palettes, trees or game assets.
