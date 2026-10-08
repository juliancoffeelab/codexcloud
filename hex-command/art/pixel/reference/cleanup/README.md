# Reviewed material cleanup

Each race has its own reviewed matrix and palette corrections. The recipes cover
the active unit sources, including both factions; archived studies stay unchanged.
The [before/after review grid](review-before-after.png) shows one example from
each of the eight independent passes.

The dwarf follow-up replaces scattered helmet highlights with connected steel
shading. Cuirasses follow the visible torso, with leather work aprons below the
waist rather than rectangular sheets of armour between the legs. Material fixes
keep beards, hands and wooden weapon shafts separate from plate.
See the [helmet and cuirass comparison](dwarf-refinement.png) and the
[regenerated/new dwarf tiers](dwarf-new-poses.png). Crossbow and pickaxe now use
new dwarf-sized imagegen poses; the pistol goat rider and axe-and-pick infantry
are additional families. Their material overlays preserve these new anatomies.

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

Anatomy, weapons, canvas dimensions, transparency and outer contours are preserved.
Only material ownership, internal stray color pixels and connected shading planes
are corrected. Natural beast markings and human magical branch colors are kept.

Reviewed L1 dwarf records may flag `removeInteriorOutline` for a specific stray
dark pixel inside a helmet, face or garment. Replay permits that removal only
when the entire 3×3 neighborhood is opaque; pixels touching transparency remain
protected. Pupils, facial features, weapon seams and grips are retained by the
pose review. L2/L3 artwork is left for a later derivation pass.
The [current L1 dwarf grid](dwarf-l1.png) covers all ten base poses; higher-tier
comparison sheets retain the earlier L2/L3 artwork until that derivation.

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
