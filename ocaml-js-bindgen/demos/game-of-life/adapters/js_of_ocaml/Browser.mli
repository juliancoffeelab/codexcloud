(** Browser operations used by this frontend. Host values stay in their native
    representation. *)

module Js = Js_of_ocaml.Js
module Dom = Js_of_ocaml.Dom
module Html = Js_of_ocaml.Dom_html

type element = Html.element Js.t
type event = Html.event Js.t
type native_string = Js.js_string Js.t

val to_host : string -> native_string
(** Convert an OCaml string to a host JavaScript string using the generated runtime. *)

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
val set_class : element -> native_string -> unit
val set_attribute : element -> native_string -> native_string -> unit
val listen_raw : < .. > Js.t -> native_string -> (#Html.event Js.t -> 'a) -> unit

(** Convert a host rejection value to display text. *)

val reload : unit -> unit
(** Reload the current page. *)

val exception_text : exn -> string
(** Render an OCaml or host JavaScript exception for display. *)
