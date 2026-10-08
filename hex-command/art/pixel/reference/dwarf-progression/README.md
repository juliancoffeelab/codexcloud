# Deferred dwarf armour progression

This leather → mixed metal → silver pass was rejected: its hand-assigned masks
proved colour coverage but did not prove that the artwork had clean part borders.
The rejected L2/L3 files are saved under `tiers/`, with their original native
inputs under `bases/`. They remain historical studies.
`build.py` is read-only and `--check` checks those 18 archived L2/L3 designs only.
It deliberately refuses to write art.

L1 has been restored to its earlier equipment and cleaned separately. The
[dwarf base cleanup](../dwarf-base-cleanup/README.md) owns all nine non-rune L1
pose matrices. Five extra base poses are branch references; the
[current upgrade recipe](../dwarf-upgrades/README.md) owns active L2/L3 artwork.
The rune family remains untouched for the future Thundersmith /
Enchanter design. Do not use this study's masks to recolour those cleaned bases.

The [branch board](lineup-player.png) contains the restored bases beside the
historical advanced tiers. It is a comparison, not an approved progression.
The [chosen trees](../../lineups/dwarves.json) define the current artwork branches;
gameplay definitions are unchanged.

```sh
python3 art/pixel/reference/dwarf-progression/build.py --check
python3 art/pixel/reference/dwarf-base-cleanup/build.py --check
```

The original `sources.json`, `bases/` and armour masks are kept for provenance.
`coverage.json` explicitly records the limitation of the historical mask check.
