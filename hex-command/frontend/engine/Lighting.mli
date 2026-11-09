val key : Domain.battlefield_lighting -> string
(** Pure light transforms, shared by offline terrain, its rendering worker and SVG overlays. Rust
    supplies already-resolved light/fog conditions. *)

val matrix : float -> float -> Domain.battlefield_lighting -> float array
val daylight : float -> float -> float array

type pixels
(** Browser and Node Uint8ClampedArray storage, without representation exposure. *)

val shade : pixels -> int -> int -> int -> float -> Domain.battlefield_lighting -> unit
val paint : Terrain_painter.context -> Domain.battlefield_lighting -> unit
val fog : Terrain_painter.context -> Terrain_plan.tile array -> string array -> unit
val position : Domain.battlefield_light -> float * float

val changed_lights :
  Domain.battlefield_light array -> Domain.battlefield_light array -> Domain.battlefield_light array
