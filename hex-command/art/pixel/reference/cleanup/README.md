# Reviewed material cleanup

Each race has its own reviewed matrix and palette corrections. The recipes cover
the active unit sources, including both factions; archived studies stay unchanged.
The [before/after review grid](review-before-after.png) shows one example from
each of the eight independent passes.

These are native pixel edits, not image filters. Each matrix record gives an exact
coordinate, expected old symbol, reviewed replacement and reason. Palette records
give the expected old and replacement RGBA values. Player and enemy corrections
are explicit so older faction conventions and intentional magical gem colors
remain intact.

The renderer applies these corrections after construction scripts regenerate
their source matrices. Application is idempotent: an already corrected pixel or
palette stays unchanged. A different source symbol, palette or canvas fails with
a stale-source error instead of applying a pose-specific correction to unrelated
artwork. `npm run art:check` verifies the corrections are already present.

Anatomy, weapons, canvas dimensions, transparency and outlines are preserved.
Only material ownership, internal stray color pixels and connected shading planes
are corrected. Natural beast markings and human magical branch colors are kept.

For future matrix edits, update the owning cleanup recipe alongside the source
so rebuilding retains the change. Recipe coordinates refer to the canonical
native matrix, not the larger imagegen source or its original extraction slot.

Rebuild and verify from the project root:

```sh
python3 tests/pixel_cleanup_test.py
npm run art:build
npm run art:check
npm run style
```
