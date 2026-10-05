# direwolf

Veteran of wolf. The approved four grounded legs, compact muzzle, attached fang,
eye and body planes remain. A connected charcoal `D/O` shoulder mane extends
to y22 at x21–31. Dark leather `L/l` reinforces the existing faction harness
along its right edge, y29–38. A connected gold `K/g` clasp sits at x28–30,y31–32.
`G` still means grey fur and is never gold. Every original fur color remains
unchanged; no gold or leather belongs in paws, fangs or muzzle. Both factions
share geometry, face screen right and receive upper-left light.

See [the progression specifications](../../../../design/progression-art.md).

# Wolf A: fresh native transcription

Drawn from a blank 48×48 field, against the fixed numbered Option A guide.
The wolf faces right in a three-quarter side stance. The near legs are the
light left hind leg and central front leg; the darker hind leg and forward
sloping right front leg are behind them. Four limbs originate in the two
large body masses, rather than being added at the belly edge.

## Component specification, revised with the matrix

- Body: long, low back, x15–31/y29–34; rump x14–22/y30–39;
  shoulder/chest x27–35/y30–40. The belly rises at x20–23/y35–36
  then descends toward the heavy chest. The far hind leg is visible below it.
- Tail: continuous rump attachment at x14–16/y32–35; descending sweep
  narrows to x4–10/y37 and separate points x3–5 and x7/y38. The lower
  notches x6/y38 and x8–12/y38 are transparent, separating tail and hind leg.
- Near hind leg: full thigh x14–20/y34–39, knee x10–14/y40–41,
  sloping shin x8–11/y42–44, low ankle x7–9/y45, paw x7–10/y46.
- Far hind leg: dark upper thigh x20–23/y37–40, knee x18–20/y41–42,
  back-bending ankle x19–22/y43–45, paw x22–24/y46.
- Near front leg: rooted in shoulder x29–33/y35–39, straight heavy upper
  leg x29–31/y40–42, lower leg x31–32/y43–44, paw x32–35/y45–46.
- Far front leg: rooted behind chest x34–36/y38–40, forward diagonal
  foreleg x36–41/y41–44, paw x40–43/y45–46. Its shoulder root must
  connect above the near limb, with a visible dark separation below it.
- Mane: angular upper tufts x29–36/y25–29, joining the shoulder and cheek
  at x32–37/y30–35. The mane is a raised mass above the low torso.
- Ears: nearer upright pale-rimmed point at x38–39/y27–30, farther small
  dark point x41/y28–31. Both join the cranial silhouette.
- Head: brow/skull x37–41/y31–34; one subdued grey eye glint x40/y34
  sits under the two-pixel dark brow x39–40/y33, between dark socket
  edges x39/41/y34, above F muzzle-shadow x40/y35. The dark bridge
  slopes through x42/y34–35. Grey muzzle x37–41/y35 slopes to
  x38–43/y36, ending in dark nose x44/y36 and x43/y37. O mouth
  x40–41/y37 cuts below the grey upper muzzle. One bone fang x42/y38
  hangs directly from the visible G muzzle edge x42/y37, itself joining G x42/y36.
  The fang must visibly join that upper jaw edge in the bitmap, not merely have
  an opaque black path to the head. Underjaw x35–39/y37 is transparent.
  Pale cheek points occupy x34–35/y35 and x34/x36/y36, with notches
  at x35/y36 and x37/y36; these break the former rectangular grey slab.
  The black nose terminates the right-facing muzzle; cheek/chin meet neck.
- Scarf: teal/red streamer x21–29/y27–30; descending collar follows
  x29–34/y30–37 then wraps under jaw at x34–36/y38. Preserve the
  mane behind the scarf and the cheek in front of it.

## Material ownership and light

`.` is hard transparency. `D/H` are the dark charcoal fur shadow/light group
at RGB (45,42,41)/(60,57,56); `F/G` form broad pale grey flank, belly, leg,
tail-tip and muzzle patches at RGB (84,81,78)/(142,139,133). Grey leg and
patch shadows use F rather than coat H, preserving their material separation.
`O` defines a one-pixel dark contour around the silhouette as well as the nose,
eye socket/brow, mouth, toe divisions and structural seams. The outline keeps
the pale legs and muzzle readable against the green ground without moving their
interior pixels or changing their attachment points. `I` is the single bone fang.
The eye uses G so its one native pixel
remains subdued; O socket edges and the F shadow below keep it distinct.
The baseline paw divisions are O at x8/10, x23, x33/35 and x41/43 on y46.
`T/t` is the scarf's light/shadow pair; `R/r` substitutes
only those symbols in the enemy matrix. No faction changes touch geometry.
Light is from the upper left: top back, ear rim, upper muzzle, outer legs and
tail tip receive connected pale/light regions, not scattered highlight noise.

## Verification requirements

Compare the fixed reference guide and actual numbered bitmap for every
component and all four limb attachments. Render matrices directly into native
PNGs and rebuild the native atlas after independent review. Keep source/PNG equality, 48×48,
binary alpha and faction geometry exact. This document records an intended
layout; it is not a substitute for inspecting the completed bitmap.
