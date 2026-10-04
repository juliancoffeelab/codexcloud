# Bridge: existing stone crossing

Transcribed from the existing vector bridge in `src/presentation/art.ts`, using
the same water tile, horizontal stone deck, five joints and two light rails.

The source uses a 70 by 70 viewBox. Its deck at y23–45 becomes native rows
16–31; rails at y21/43 become rows 13–15/29–31. The original joints at
x10/22/34/46/58 become columns 7/15/23/32/40. The deck shadow occupies
rows 18–33. It is represented by an opaque dark-water color, with no alpha
noise. Preserve the top-down orientation and upper-left light.

The pixel renderer uses this 48 by 48 matrix. Normal and Low-poly keep their
existing vector bridge. No new generated bridge design is introduced.
