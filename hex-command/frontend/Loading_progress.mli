type t = { step : int; value : float; detail : string }
(** One application-wide progression retained across the shell/HUD handoff. Values measure five work
    segments, not download percentage or time. *)

val current : unit -> t
val subscribe : (t -> unit) -> unit -> unit
val advance : ?fraction:float -> ?detail:string -> int -> unit
val begin_battlefield : unit -> unit
val finish : unit -> unit
val label : int -> string
