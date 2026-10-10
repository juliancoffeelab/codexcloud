# Elf caster branches

The cleaned caster advances into Grove healer or Thorn caster, each with L2/L3
artwork. Both have covered legs, ivory cloth and leather boots. Healers carry
living flowering branches; Thorn casters carry dark wood with violet flowers.
Faction colors belong to connected cloaks and collars.

The four canonical matrices and palettes live in `art/pixel/units/`. The reviewed
elf manifest protects them from obsolete expansion tier masks. Each evidence
directory retains its prompt, three-reference pinkbox manifest, raw generation,
extracted matrix and explicit correction logs. Later native edits replace the
awkward casting palms and L3 staff heads, align the shafts through their grips,
remove detached remnants, and clean the healer's ivory trouser strip.

Both detectors, ordinary renders and toxic faction previews were inspected.
Final matrices have no exposed-material border flags. Remaining heuristic flags
include diagonal contours, small flowers and intentional highlights; this is
not a claim that every remaining candidate is a false positive.

Further edits use the [pixel-cleaning skill](../../../../.agents/skills/pixel-cleaning/SKILL.md)
and [enhancement skill](../../../../.agents/skills/pinkbox-enhancement/SKILL.md).
Update `../elf-reviewed/sources.json` after reviewed edits. Rebuild with
`npm run art:build`; verify with `npm run art:check` and `npm run style`.
Gameplay rules and stats are unchanged.
