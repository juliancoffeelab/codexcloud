# Original dwarf inputs

`bases/` retains the nine original native matrices, palettes and rendered images
from commit `b559200283b3279239a3225cfd2d20617e777d74`. The
[base cleanup recipe](../dwarf-base-cleanup/README.md) uses these inputs to guard
silhouettes, eyes, gripping hands and reviewed cleanup changes.

The rejected hand-assigned armour masks, 18 intermediate L2/L3 studies, comparison
boards, coverage report and obsolete builder have been removed. Previous versions
remain in Git history. Their old coverage check could not verify real part borders;
the current recipes check independent fill components detected from the cleaned art.

The [current branch upgrades](../dwarf-upgrades/README.md) and
[rune line](../rune-line/README.md) own all chosen advanced dwarf tiers.
The [lineup definition](../../lineups/dwarves.json) selects five L1 starters and
ten L2 branches, each with an L3. Gameplay definitions are unchanged.

Do not remove `bases/`: these files are required rebuild inputs.
