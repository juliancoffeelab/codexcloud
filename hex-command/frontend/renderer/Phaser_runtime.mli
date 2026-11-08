type t

val create : Phaser.Game.config -> failed:(string -> unit) -> t
val game : t -> Phaser.Game.t
val invalidate : t -> unit
val unavailable : t -> bool
val record_cleanup_error : t -> exn -> unit
val on_dispose : t -> (unit -> unit) -> unit
val fail : t -> exn -> unit
val dispose : t -> exn array
