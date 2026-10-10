# Corruption Mage commander progression

The original `zombie-corruption-mage` is **L2**, as assigned by the user. Its
matrix and palette remain byte-for-byte unchanged; the exact baseline is saved
as `preserved-l2.matrix` and `preserved-l2.palette.json`.

- L1 (`zombie-corruption-mage-l1`): the same mage's plain hood, robe, exposed
  forearms and spellcasting arrangement, with a lean caster-influenced build
  and a simple bud staff.
- L2 (`zombie-corruption-mage`): the existing approved mage.
- L3 (`zombie-corruption-archmage`): the same mage's hood/robe and handedness,
  with carrier-influenced swelling, bone talismans and a branching staff crown.
  It does not duplicate the carrier's basket or outfit.

New faces point left, matching the approved L2. The caster and carrier style
anchors were mirrored for the template only, so all three references give the
same facing cue; their canonical art is unchanged. Their mirrored matrices
are retained as extraction evidence here. Exactly three matrix references and one fill box were
used per generation. These references are the original L2, thin caster and
plague carrier. Exact prompts, generation PNGs, extraction manifests, raw
matrices and guarded corrections are retained here. The initial wrong-tier
candidate generations were deleted at the user's request.

Cleaning follows the project [pixel-cleaning skill](../../../../.agents/skills/pixel-cleaning/SKILL.md).
L1 has 48 reviewed cleanup corrections and is placed seven pixels lower
to align the feet with L2. Its body is 36 pixels tall, six fewer than L2.
L3 has 182 cleanup corrections followed by 60 flame-shape edits: the large
mass is narrowed into distinct tongues without changing the hand. Both passes include trimming
unrequested faction-colored dangling shaft fragments.
No blanket outline filling or diagonal bridging was applied. Material edits
preserve alpha and outline; only explicitly logged fragment trims change them. Faction color belongs to the hood/cape, while
ankle wraps retain fixed brown material colors.

Stray and edge scripts were run before and after cleaning, with normal, debug
and marked images inspected internally. Remaining detector candidates include
intentional corpse shading, ragged edges, flame tips and the negative space
between body and staff. These counts are not proof that every candidate is
intentional; review notes retain that limitation. Full diagnostic images and
neighborhoods remain in ignored `tmp/corruption-commander/`.

Canonical editable assets live in `../../units/`. `sources.json` records exact
promoted matrix/palette hashes; later edits should update it after review.
The old expansion commander tier recipe is disabled so it cannot replace this
progression. Both zombie board helpers now read the chosen three-level commander
row from `../../lineups/zombies.json`. Gameplay stats are unchanged.

Refresh native faction previews with `python3 scripts/render-pixel.py --previews`,
and refresh the chosen race board with
`node art/pixel/reference/zombies/lineup.cjs` from the project root. Standard
`npm run art:build` / `npm run art:check` build and verify canonical assets.
