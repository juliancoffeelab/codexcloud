# Procedural composition

Read for foreground density, layouts, shorelines and deterministic variation.

Compose foreground variation procedurally from approved pieces and legal
placements, at different levels of complexity:

- Grass and small stones: sparse, inexpensive variation with open walking space.
- Forests and hills: several pieces, clustered densities, varied silhouettes and
  coherent depth ordering. Mountains must read differently from hill stones.
- Fords: irregular stepping stones that communicate the crossing direction.
- Roads: constrained connected paths with correct exits.
- Villages/structures: fewer constrained arrangements with sensible entrances and
  approaches, rather than scattering building fragments.

Keep variation deterministic from map/tile identity. Currently the build emits
approved `layouts.json` arrangements and runtime selects among them; it does not
randomly place objects every frame or synthesize road artwork. Procedural
authoring may generate candidate layouts; persist and validate accepted
arrangements. More complex generators are separate implementation work when
needed by the requested terrain.

Place pieces at native integer coordinates without mirroring/resizing; keep
ordinary props inside the hex and draw overlaps in depth order. Complete padded
terrain frames retain their authored canvas, anchor and depth. Preview several
densities and seeds, retaining intentional clearings. Do not cut up a finished
building or mountain illustration to manufacture variation.

Do not manufacture shoreline ground by slicing a finished hex with an arbitrary
coordinate cutoff. This produces grass scraps and geometric wedges, not an
authored shore. Prefer an existing whole ground tile with foreground pieces;
design new shoreline artwork deliberately only when needed. Fitting inside the
hex is necessary but does not establish visual quality. Inspect actual joins on
both banks, not just isolated pieces or one cropped bank.
