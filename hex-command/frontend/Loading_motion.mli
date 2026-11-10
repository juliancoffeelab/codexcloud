val advance : step:int -> value:float -> unit
val restart : step:int -> value:float -> unit

val value : unit -> float
(** Presentation progress, capped inside the active stage and retained across mounts. *)

val reduced_motion : unit -> bool

val animate : Dom.element -> unit -> unit
(** Start a browser-owned animation; returned cleanup cancels it. No per-frame JS work. *)
