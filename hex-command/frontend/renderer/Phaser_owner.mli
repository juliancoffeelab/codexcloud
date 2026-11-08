type t

val prepare : Phaser_sound.t -> t
val mount : t -> Phaser.Game.config -> failed:(string -> unit) -> Phaser_runtime.t
val dispose : t -> unit

val mount_attached :
  t -> Phaser.Game.config -> (string -> unit) -> (Phaser_runtime.t -> unit) -> Phaser_runtime.t

type battlefield

val mount_battlefield : t -> Battlefield_view.mount_options -> battlefield
val present_battlefield : battlefield -> Battlefield_view.view -> bool -> unit
val pulse_battlefield : battlefield -> string -> string -> Phaser_sound.interaction option -> unit
val retry_battlefield : battlefield -> Phaser_resource_plan.kind option -> unit
val dispose_battlefield : battlefield -> unit
val commander_battlefield : battlefield -> unit
val retry_audio : t -> unit
