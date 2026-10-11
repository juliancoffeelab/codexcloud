(** Best-effort feedback, triggered by commands after their view commits.
    Existing feedback is replaced and reduced-motion preferences are respected. *)
val animate_tile : string -> string -> unit
val animate_turn : unit -> unit

