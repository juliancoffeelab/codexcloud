type t

(** Shared decoded presentation portraits. This cache never assembles terrain. *)

type image = Browser_image.t

val create : unit -> t
val ready : t -> Art.frame -> image option
val load : t -> Art.frame -> image Js.Promise.t
val retry : t -> Art.frame -> image Js.Promise.t
val bind : t -> (Art.frame -> image Js.Promise.t) -> unit -> unit
val adopt : t -> string -> image -> unit
val close : t -> unit
val mount : Dom.element -> image -> unit
