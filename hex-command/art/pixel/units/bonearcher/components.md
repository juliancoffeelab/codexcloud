# Bone archer — matched skeleton proportions

Native 48×48 matrices. The skull and shoulders sit four rows higher than the
previous sprite; the skull has one additional back column, and the near arm/rib
edge is slightly broader. Facial cavities, drawing fingers and bow grip retain
their existing design. Both complete thigh/shin chains connect the raised pelvis
to the original compact boots, grounded at row 47. The braced shooting pose remains.

Components: right-facing ivory skull, drawing arm and nock fingers, extended bow
arm and gripping fingers, rib groups with dark gaps, leather belt/bracers/boots,
faction scarf/cape and ragged waistcloth, curved wooden bow, taut string and arrow.

Materials: ivory bone B/b, dark cavities/outline O, leather L/l, wood W/w,
silver arrowhead S/s, gold belt G/g and faction cloth q/c/C (r/R/h for enemy).
Light comes from upper left. Use broad connected planes and purposeful bone gaps.

The enlarged bow runs from (34,6) to (34,40), with the string meeting the nock
at (24,23). The one-row arrow remains at the exact midpoint, y23, and its
five-cell steel head occupies rows 22–24. The skull occupies rows 11–18;
the belt is at rows 28–29 and the lengthened leg chains run through row 42.
Both ranks share these anatomical proportions, and both factions share geometry.

Edit player.txt and enemy.txt, then rebuild native PNGs and the game atlas with
npm run art:build. npm run art:check verifies the rendered assets against matrices.
