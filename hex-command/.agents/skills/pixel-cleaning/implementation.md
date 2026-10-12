# Pixel cleaning implementation

## Review output

The review bundle contains faction previews, detector findings and enlarged
eight-neighbor contexts. The stray detector uses cardinal equal-color neighbors;
intentional eyes, highlights and diagonal details can therefore be flagged.
Optional material shade groups are input hints, not proof of material ownership.
Detectors can miss connected wrong-material patches, so a full review also
examines unflagged surfaces in the source image and matrix.

## Edit contract

The edit JSON requires `matrix_sha256`, `palette_sha256` and `edits`. Each edit
has native `x`, `y`, `from`, `to`, `material` and `reason`. Outline or transparency
changes also require `boundary_reviewed: true`. Hash guards reject stale inputs;
the helper preserves the baseline and applies only the specified pixel edits.

## Terrain joins

Road exits meet at the same position and width with a normal tangent through the
hex boundary. A curve can end off-center even when its connection name matches.
The artwork build checks boundary sections independently of the interior.
Inspect connected forks, crossroads and curves to assess actual continuity.
