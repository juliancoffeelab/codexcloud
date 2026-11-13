type t

type readiness = {
  scope : string;
  ready : bool;
  completed : int;
  total : int;
  error : string Js.Nullable.t;
}

val create : unit -> t
val enabled : t -> bool
val set_enabled : t -> bool -> unit
val close : t -> unit
val prepare : t -> string -> Catalog.t -> Native.view -> unit
val speak : t -> string -> Domain.dialogue array -> int -> (string -> unit) -> unit -> unit

val watch_opening :
  t -> string -> Domain.dialogue array -> int -> (readiness -> unit) -> unit -> unit

val retry : t -> unit
