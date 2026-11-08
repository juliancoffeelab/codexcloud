type t
type error = Cancelled | Failed of Foreign_error.cause

val start : Phaser_sound.t -> t
val close : t -> unit

type battlefield

val mount_battlefield :
  t -> Battlefield_view.mount_options -> (battlefield, error) result Js.Promise.t

val present : battlefield -> Battlefield_view.view -> bool -> unit
val pulse : battlefield -> string -> string -> Phaser_sound.interaction option -> unit
val retry_resources : battlefield -> unit
val dispose_battlefield : battlefield -> unit
val commander : battlefield -> unit
val retry_audio : t -> unit Js.Promise.t
