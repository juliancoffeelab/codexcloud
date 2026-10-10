# Reviewed elf sources

The reviewed artwork lives in `art/pixel/units/`, rather than scratch folders.
`sources.json` records the promoted matrices, palettes and native canvases.
These sources include warden and jumper L1–L3, the sword and dual-blade L1–L3,
the cleaned caster L1, shadow-stalker L2/L3, and the deer/shadow rider L1–L3.

Canonical `sprite.txt` and `palette.json` files are authoritative. The expansion
tier builder skips these unit IDs; obsolete pose-specific elf cleanup records
have been retired for them. Historical generated images remain design evidence,
not replacement inputs for these approved sources. New reviewed edits must update
the hashes in `sources.json`.

The [caster branches](../elf-caster-branches/README.md) add Grove healer and
Thorn caster L2/L3 with covered legs and flowering staffs. The jumper and Thorn
caster have separate catalog entries and atlas frames; gameplay advancement
rules are unchanged.

Use the project [pixel-cleaning skill](../../../../.agents/skills/pixel-cleaning/SKILL.md)
for further corrections, and [enhancement skill](../../../../.agents/skills/pinkbox-enhancement/SKILL.md)
for tiers. Rebuild and verify from the project root:

```sh
python3 art/pixel/reference/elf-reviewed/check.py
npm run art:build
npm run art:check
npm run style
```

## Roster roots and branches

[elves.json](../../lineups/elves.json) records the five agreed L1 roots: Spearman,
Two knifer, Archer, Deer rider and Grove caster. The blade root supplies starting
flanking capability and advances to Duelist (two blades) or Swordsman (two-handed
sword). Spearman branches to Warden or Jumper; the caster branches to Healer or
Thorn caster. Branches begin at L2. Leap is a proposed jumper ability, not an
implemented gameplay rule.

The old swordsman and jumper L1 matrices remain canonical reference artwork,
marked `reference` in the artwork catalog. They are omitted as roster roots.
Lineup previews repeat each shared L1 root on its branch rows, keeping tiers
aligned. Elves have no live gameplay roster yet; this manifest is their authored
roster plan.
