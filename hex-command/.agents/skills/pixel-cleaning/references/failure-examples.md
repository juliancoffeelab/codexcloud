# Observed failures and the correct reasoning

These examples come from the elf L1 warden, jumper and swordsman review. Native
coordinates refer to those exact scratch matrices; do not reuse them on another
pose. The rules are generic, not coordinate fixtures or instructions to copy edits.

| Failure                                                                                                                                         | Required reasoning                                                                                                                                                            |
| ----------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Brown wood/leather leaked into the warden's red cloak. Filling it black created a seam instead of cloth.                                        | Inspect shaft, boot and cloak separately. Replace the leaked pixels with local faction cloth; preserve the real shaft and boot outlines.                                      |
| The jumper had a straight separator between legs with one missing pixel. Cleanup erased the separator twice.                                    | Internal boundaries are anatomy too. Fill the single missing outline cell; preserve the rest of the line.                                                                     |
| A black pixel outside jumper hair enclosed a transparent notch, producing a "shadow ring." Filling the hole enlarged the damage.                | Inspect exterior connectivity. Remove the unsupported exterior black pixel to reopen the notch; do not assume the hole must be hair.                                          |
| Cardinal diagonal-gap flags led to all diagonal outlines being bridged horizontally/vertically. Weapons became fat and poses changed.           | Diagonal outline contact is legitimate. Inspect eight-neighbor geometry; bridge only a genuine local defect.                                                                  |
| A swordsman sleeve had two skin-colored pixels; the neck had an ivory pixel. Nearby green pixels were painted black and counted as stray fixes. | Change the two actual sleeve pixels to ivory and neck to skin. Match arrows to coordinates and confirm all three defects independently of nearby edge repairs.                |
| A swordsman waist patch looked like a stray but was an accepted sash.                                                                           | Material ownership and garment structure override color-isolation counts. Keep approved intentional elements.                                                                 |
| A legal leather shade was labeled leather despite no leather nearby.                                                                            | Color names do not establish local ownership. Inspect the surrounding region and actual material continuity.                                                                  |
| Detector output was described as empty although no current marked image had been inspected.                                                     | Run the script on the actual candidate, open its output, inspect all relevant flags and the unmarked render. Report candidates, confirmed defects and uncertainty separately. |
| Weapon recoloring was substituted for missing outlines.                                                                                         | Fix the requested contour, keeping approved metal/wood colors and native weapon width.                                                                                        |
| The wrong earlier candidate was restored.                                                                                                       | Save exact matrices, palettes and hashes for each approved checkpoint. Restore files, not memories or a new reconstruction.                                                   |

## Audit of reused tools

`check_isolated_pixels.py` remains a cardinal candidate detector with optional
material shade groups. `check_broken_outlines.py` remains a geometry/ownership
candidate detector. Their limitations are explicit in the cleaning skill.
`pixel_review_image.py` provides original/marked views. The bundled review helper
adds per-rule images, all eight neighbors, material component sizes and enlarged
neighborhoods. It does not silently filter diagonal flags or erase uncertain ones.

Generation now uses a manifest with one explicit crop per output box, current
faction colors and source hashes. The historical bottom-right extractor used its
own saved palette; it remains valid for its saved inputs, not as the default for
new designs. Skeleton servant tier edits were direct matrix/palette work;
Lich commander tiers were independent pinkbox generations. Both preserve their
own evidence and validation rather than one universal "tier" algorithm.
