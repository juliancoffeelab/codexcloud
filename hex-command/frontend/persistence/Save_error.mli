type kind = Full | Blocked | Internal
type t = { kind : kind; detail : string; cause : Foreign_error.cause option }

exception Failed of t

val internal : string -> t
val blocked : string -> t
val kind_name : kind -> string
val of_error : 'a -> t
val detail : 'a -> string
val exception_ : t -> exn
val reject : t -> 'a Js.Promise.t
val reject_internal : string -> 'a Js.Promise.t
val reject_error : 'a -> 'b Js.Promise.t
val attempt : (unit -> 'a Js.Promise.t) -> 'a Js.Promise.t
val catch : 'a Js.Promise.t -> 'a Js.Promise.t
val raise_internal : string -> 'a
val promise : ('a, t) result -> 'a Js.Promise.t
val await : ('a, t) result Js.Promise.t -> 'a Js.Promise.t
