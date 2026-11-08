module Js = Js_of_ocaml.Js
module Dom = Js_of_ocaml.Dom
module Html = Js_of_ocaml.Dom_html

type element = Html.element Js.t
type event = Html.event Js.t
type native_string = Js.js_string Js.t
type worker
type worker_options = < _type : native_string Js.readonly_prop > Js.t
type message = { kind : native_string; value : native_string }

let runtime = Js.Unsafe.get Js.Unsafe.global "Ocamljs_async_search_demo"

let to_host value : native_string =
  Js.Unsafe.fun_call
    (Js.Unsafe.get runtime "host_string")
    [| Js.Unsafe.inject (Js.bytestring value) |]

let of_host = Js.to_string
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

let get_value element =
  let control : < value : native_string Js.prop > Js.t =
    Js.Unsafe.coerce element
  in
  control##.value

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

let new_worker (url : native_string) (options : worker_options) : worker =
  Js.Unsafe.new_obj
    (Js.Unsafe.get Js.Unsafe.global "Worker")
    [| inject url; inject options |]

let terminate (worker : worker) : unit = call worker "terminate" [||]

let post (worker : worker) (message : message) : unit =
  call worker "postMessage"
    [|
      inject
        (Js.Unsafe.obj
           [| ("kind", inject message.kind); ("value", inject message.value) |]);
    |]

let worker_message (worker : worker) (callback : event -> unit) : unit =
  Js.Unsafe.set worker "onmessage" (inject (Js.wrap_callback callback))

let worker_error (worker : worker) (callback : event -> unit) : unit =
  Js.Unsafe.set worker "onerror" (inject (Js.wrap_callback callback))

let worker_message_error (worker : worker) (callback : event -> unit) : unit =
  Js.Unsafe.set worker "onmessageerror" (inject (Js.wrap_callback callback))

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

let worker_options () : worker_options =
  object%js
    val _type = Js.string "module"
  end

let message_valid message limit =
  Js.to_string (Js.typeof message.kind) = "string"
  && Js.to_string (Js.typeof message.value) = "string"
  && (Js.Unsafe.get message.kind "length" : int) <= 32
  && (Js.Unsafe.get message.value "length" : int) <= limit

let data event : message option =
  let value = Js.Unsafe.get event "data" in
  if Js.to_string (Js.typeof value) = "undefined" || not (Js.Opt.test value)
  then None
  else
    Some
      { kind = Js.Unsafe.get value "kind"; value = Js.Unsafe.get value "value" }

let error_text value =
  Js.to_string
    (Js.Unsafe.fun_call
       (Js.Unsafe.get Js.Unsafe.global "String")
       [| inject value |])

let reload () = Html.window##.location##reload
let exception_text = Printexc.to_string
let rejection_text event = error_text (Js.Unsafe.get event "reason")
let prevent_default = Dom.preventDefault
