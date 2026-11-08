type cancellation = Session_replaced | Disposed
type outcome = Completed | Cancelled of cancellation | Failed of Foreign_error.cause
type token = private { id : int; combat : Domain.combat_animation option }
type t
type lease

val create : Domain.combat_animation option -> t
val token : t -> token
val finished : t -> outcome Js.Promise.t
val fail : t -> Foreign_error.cause -> unit
val cancel : t -> cancellation -> unit
val claim : token -> lease
val complete : lease -> unit
external owner : token -> Phaser_sound.owner = "%identity"
