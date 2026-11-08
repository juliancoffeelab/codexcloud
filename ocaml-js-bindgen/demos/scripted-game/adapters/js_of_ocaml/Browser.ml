module Js = Js_of_ocaml.Js
module Dom = Js_of_ocaml.Dom
module Html = Js_of_ocaml.Dom_html

type element = Html.element Js.t
type event = Html.event Js.t
type native_string = Js.js_string Js.t
type message = { kind : native_string; value : native_string }

let runtime = Js.Unsafe.get Js.Unsafe.global "Ocamljs_scripted_game_demo"

let to_host value : native_string =
  Js.Unsafe.fun_call
    (Js.Unsafe.get runtime "host_string")
    [| Js.Unsafe.inject (Js.bytestring value) |]

let inject = Js.Unsafe.inject
let call object_ name args = Js.Unsafe.meth_call object_ name args
let global = Html.window
let document_of (window : Html.window Js.t) = window##.document

let document_element (document : Html.document Js.t) =
  document##.documentElement

let body (document : Html.document Js.t) = document##.body

let find_id (document : Html.document Js.t) id =
  Js.Opt.to_option (document##getElementById id)

let create_element (document : Html.document Js.t) tag =
  document##createElement tag

let set_text (element : element) text = element##.textContent := Js.some text

let replace_children (element : element) : unit =
  call element "replaceChildren" [||]

let append_child = Dom.appendChild

let set_attribute (element : element) name value =
  element##setAttribute name value

let listen_raw target name callback =
  let _ =
    Dom.addEventListener target
      (Dom.Event.make (Js.to_string name))
      (Html.handler (fun event ->
           callback event;
           Js._true))
      Js._false
  in
  ()

let post_global (message : message) : unit =
  Js.Unsafe.fun_call
    (Js.Unsafe.get Js.Unsafe.global "postMessage")
    [|
      inject
        (Js.Unsafe.obj
           [| ("kind", inject message.kind); ("value", inject message.value) |]);
    |]

let close_worker () : unit =
  Js.Unsafe.fun_call (Js.Unsafe.get Js.Unsafe.global "close") [||]

let error_text value =
  Js.to_string
    (Js.Unsafe.fun_call
       (Js.Unsafe.get Js.Unsafe.global "String")
       [| inject value |])

let reload () = Html.window##.location##reload
let exception_text = Printexc.to_string
let rejection_text event = error_text (Js.Unsafe.get event "reason")
let prevent_default = Dom.preventDefault
