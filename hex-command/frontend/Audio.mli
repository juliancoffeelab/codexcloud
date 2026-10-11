(** Gesture-gated, optional audio. Playback never creates a context; cancellation
    invalidates pending samples and stops active voices without changing preference. *)
type t
val create : unit -> t
val enabled : t -> bool
val set_enabled : t -> bool -> unit
val cancel : t -> unit
val unlock : t -> trusted:bool -> bool Js.Promise.t
val play : t -> string -> bool Js.Promise.t
val interaction_family : ?attack:string -> ?unit_type:string -> ?terrain:string -> unit -> string
val play_interaction : t -> ?attack:string -> ?unit_type:string -> ?terrain:string -> unit -> bool Js.Promise.t

