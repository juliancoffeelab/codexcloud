type tile_event = { tile : string; kind : string; sound : Phaser_sound.interaction option }

val animate_tile : ?sound:Phaser_sound.interaction -> string -> string -> unit

(** Best-effort feedback, triggered by commands after their view commits. Existing feedback is
    replaced and reduced-motion preferences are respected. *)

val animate_turn : unit -> unit
val reduced_motion : unit -> bool
val subscribe_reduced : (bool -> unit) -> unit -> unit

type event

val detail : event -> tile_event
val listen : Dom.element -> (event -> unit) -> unit
val unlisten : Dom.element -> (event -> unit) -> unit
