type 'a t
type 'a reader

val create : 'a -> 'a t
val reader : 'a t -> 'a reader
val get : 'a reader -> 'a
val subscribe : 'a reader -> ('a -> unit) -> unit -> unit
val set : 'a t -> 'a -> unit
