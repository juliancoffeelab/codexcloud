val by_id : string -> Browser.element
(** Find a required element by its HTML id. Raises Failure if it is absent. *)

val create : string -> Browser.element
(** Create an unattached element with the given HTML tag. *)

val text : Browser.element -> string -> unit
(** Replace the element text content. *)

val clear : Browser.element -> unit
(** Remove all child nodes. *)

val append : Browser.element -> Browser.element -> unit
(** Append the second element to the first. *)

val value : Browser.element -> string
(** Read the current form control value. *)

val set_value : Browser.element -> string -> unit
(** Set the form control value. *)

val on_click : Browser.element -> (unit -> unit) -> unit
(** Register a click callback. *)

val class_name : Browser.element -> string -> unit
(** Replace the element CSS class list. *)

val button : string -> (unit -> unit) -> Browser.element
(** Create an unattached button with a label and click callback. *)

val ready : unit -> unit
(** Mark the document ready for interaction. *)

val failed : string -> unit
(** Append the failure message and a Reload button to the page. *)
