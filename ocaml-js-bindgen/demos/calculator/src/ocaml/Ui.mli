val by_id : string -> Browser.element
(** Find a required element by its HTML id. Raises Failure if it is absent. *)

val text : Browser.element -> string -> unit
(** Replace the element text content. *)

val value : Browser.element -> string
(** Read the current form control value. *)

val on_click : Browser.element -> (unit -> unit) -> unit
(** Register a click callback. *)

val ready : unit -> unit
(** Mark the document ready for interaction. *)

val failed : string -> unit
(** Append the failure message and a Reload button to the page. *)

val on_key : (string -> bool) -> unit
(** Handle unmodified keyboard input. Return true to prevent the browser default. *)

val on_close : (unit -> unit) -> unit
(** Run when the page leaves, except when preserved in the browser back/forward cache.
*)
