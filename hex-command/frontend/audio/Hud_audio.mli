type readiness = Audio_state.readiness = { ready : bool; error : string option }

val readiness : readiness Observable.reader

type t

val create : (unit -> bool) -> t
val unlock : t -> bool Js.Promise.t
val prepare : t -> unit Js.Promise.t
val play : t -> bool Js.Promise.t
val stop : t -> unit
val dispose : t -> unit
val gestures : string array
