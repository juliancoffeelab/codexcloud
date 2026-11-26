type recording
type t

val create : unit -> t
val clear : t -> unit
val begin_turn : ?learned_model:string -> t -> string -> Setup.t -> recording
val record : recording -> Domain.Command.t -> unit
val complete : recording -> unit
val download : ?browser_saves:Js.Json.t -> t -> string -> Setup.t -> Domain.state -> unit
