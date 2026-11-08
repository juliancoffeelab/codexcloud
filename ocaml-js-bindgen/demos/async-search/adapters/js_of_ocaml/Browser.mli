(** Browser operations used by this frontend. Host values stay in their native
    representation. *)

module Js = Js_of_ocaml.Js
module Dom = Js_of_ocaml.Dom
module Html = Js_of_ocaml.Dom_html

type element = Html.element Js.t
type event = Html.event Js.t
type native_string = Js.js_string Js.t
type worker
type worker_options = < _type : native_string Js.readonly_prop > Js.t
type message = { kind : native_string; value : native_string }

val to_host : string -> native_string
(** Convert an OCaml string to a host JavaScript string using the generated runtime. *)

val of_host : native_string -> string
(** Convert a host JavaScript string to an OCaml string. *)

val global : Html.window Js.t
val document_of : Html.window Js.t -> Html.document Js.t
val document_element : Html.document Js.t -> element
val body : Html.document Js.t -> Html.bodyElement Js.t

val find_id : Html.document Js.t -> native_string -> element option
(** Find an element by id, returning None when absent. *)

val create_element : Html.document Js.t -> native_string -> element
val set_text : element -> native_string -> unit
val replace_children : element -> unit
val append_child : #Dom.node Js.t -> #Dom.node Js.t -> unit
val get_value : element -> native_string
val set_attribute : element -> native_string -> native_string -> unit
val listen_raw : < .. > Js.t -> native_string -> (#Html.event Js.t -> 'a) -> unit
val new_worker : native_string -> worker_options -> worker
val terminate : worker -> unit
val post : worker -> message -> unit
val worker_message : worker -> (event -> unit) -> unit
val worker_error : worker -> (event -> unit) -> unit
val worker_message_error : worker -> (event -> unit) -> unit
val post_global : message -> unit
val close_worker : unit -> unit

val worker_options : unit -> worker_options
(** Create options for an ES module worker. *)

val message_valid : message -> int -> bool
(** Check a message against the supplied maximum payload length. *)

val data : event -> message option
(** Read an event payload, returning None if absent. Use message_valid before accessing
    its fields. *)

val error_text : 'a -> string
(** Convert a host rejection value to display text. *)

val reload : unit -> unit
(** Reload the current page. *)

val exception_text : exn -> string
(** Render an OCaml or host JavaScript exception for display. *)

val rejection_text : event -> string
val prevent_default : 'a #Dom.event Js.t -> unit
