# Skeleton and wolf design references

The letters are embedded in the original generated sheets; the saved filenames
retain the same labels.

- `option-A.png`: selected wolf reference, on the right.
- `option-B.png`: selected skeleton warrior reference, on the left. Its rabid
  wolf, on the right, is reserved for possible future use.
- `option-D.png`: alternate wolf reference, on the right, also reserved for
  possible future use.

These are design references. Native sprites are authored as editable 48×48
matrices in `art/pixel/units/`; generated sheets are not downscaled into sprites.

`skeleton-B-numbered.png` and `wolf-A-numbered.png` preserve the fixed target
guides used during transcription and independent review. Columns and rows are
numbered 0–47. The grids describe target placement; the original generated
references contain finer, uneven blocks. Each cell in a native sprite's grid
corresponds to one actual pixel in its editable matrix.

The skeleton guide maps source square `(0, 90, 720, 810)` onto the 48×48 field.
The wolf guide maps source square `(768, -144, 1728, 816)` onto that field,
with the label and unrelated content omitted from its blank upper region.
These transforms remain fixed across revisions.
