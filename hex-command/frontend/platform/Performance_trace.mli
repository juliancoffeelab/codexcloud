type config = { id : int; origin : float }
type status = Empty | Recording | Stopping | Ready
type state = { status : status; events : int; dropped : int; reason : string option }

val state : unit -> state
val subscribe : (state -> unit) -> unit -> unit
val configuration : unit -> config option
val instant : ?args:Js.Json.t -> string -> unit
val measure : ?args:Js.Json.t -> string -> (unit -> 'a) -> 'a
val obj : (string * Js.Json.t) list -> Js.Json.t
val int : int -> Js.Json.t
val start : (unit -> Js.Json.t) -> unit
val stop : ?reason:string -> unit -> unit Js.Promise.t
val download : unit -> unit Js.Promise.t

external then_result :
  'a Js.Promise.t ->
  (('a -> 'b Js.Promise.t)[@u]) ->
  ((Foreign_error.t -> 'b Js.Promise.t)[@u]) ->
  'b Js.Promise.t = "then"
[@@mel.send]

external reject : Foreign_error.t -> 'a Js.Promise.t = "reject" [@@mel.scope "Promise"]
