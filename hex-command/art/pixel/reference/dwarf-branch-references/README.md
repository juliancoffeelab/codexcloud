# Dwarf branch pose references

Five extra poses are preserved here as construction references: axe + pick,
hammerer, poleaxe, arquebusier and pistol goat rider. They feed the corresponding
L2 branches and L3 upgrades; they are not extra L1 starters in the active catalog.
Their cleaned matrices are rebuilt by `dwarf-base-cleanup/build.py` from its
authoritative `bases/*/player.mask` files.

`catalog.json` records their original study IDs and parent starters, so construction
scripts and galleries resolve them here. The unused miner L2/L3 studies and
rejected older progression tiers have been removed; previous designs remain in
Git history. Original baseline inputs required by the cleanup checks remain in
`../dwarf-progression/bases/`.

See the [actual five trees](../dwarf-upgrades/README.md) and
[lineup definition](../../lineups/dwarves.json). The rune tree is complete.
