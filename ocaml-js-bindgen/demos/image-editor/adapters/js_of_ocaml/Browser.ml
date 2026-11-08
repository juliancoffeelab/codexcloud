module Js = Js_of_ocaml.Js
module Dom = Js_of_ocaml.Dom
module Html = Js_of_ocaml.Dom_html

type element = Html.element Js.t
type event = Html.event Js.t
type rect = Html.clientRect Js.t
type style = Html.cssStyleDeclaration Js.t
type native_string = Js.js_string Js.t
type events
type touches = Html.touchList Js.t
type touch = Html.touch Js.t
type listener_options = { passive : bool }

let runtime = Js.Unsafe.get Js.Unsafe.global "Ocamljs_image_editor_demo"

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
let append_child = Dom.appendChild

let get_value element =
  let control : < value : native_string Js.prop > Js.t =
    Js.Unsafe.coerce element
  in
  control##.value

let set_value element value =
  let control : < value : native_string Js.prop > Js.t =
    Js.Unsafe.coerce element
  in
  control##.value := value

let set_attribute (element : element) name value =
  element##setAttribute name value

let get_attribute (element : element) name =
  Js.Opt.to_option (element##getAttribute name)

let closest (element : element) selector =
  Js.Opt.to_option (element##closest selector)

let click (element : element) = element##click

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

let listen_options target name callback options =
  let _ =
    Dom.addEventListenerWithOptions target
      (Dom.Event.make (Js.to_string name))
      ~passive:(Js.bool options.passive)
      (Html.handler (fun event ->
           callback event;
           Js._true))
  in
  ()

let frame_raw callback =
  Html.window##requestAnimationFrame
    (Js.wrap_callback (fun time -> callback (Js.float_of_number time)))

let cancel_frame id = Html.window##cancelAnimationFrame id

let timeout_raw callback delay =
  Html.window##setTimeout (Js.wrap_callback callback) (Js.number_of_float delay)

let set_disabled (element : element) (disabled : bool) : unit =
  Js.Unsafe.set element "disabled" (inject (Js.bool disabled))

let disabled (element : element) : bool =
  Js.to_bool (Js.Unsafe.get element "disabled")

let set_hidden (element : element) (hidden : bool) : unit =
  Js.Unsafe.set element "hidden" (inject (Js.bool hidden))

let width element =
  let canvas : Html.canvasElement Js.t = Js.Unsafe.coerce element in
  float_of_int canvas##.width

let height element =
  let canvas : Html.canvasElement Js.t = Js.Unsafe.coerce element in
  float_of_int canvas##.height

let client_width (element : element) = float_of_int element##.clientWidth
let client_height (element : element) = float_of_int element##.clientHeight
let bounds (element : element) = element##getBoundingClientRect
let left (rect : rect) = Js.float_of_number rect##.left
let top (rect : rect) = Js.float_of_number rect##.top
let rect_width (rect : rect) = Js.float_of_number rect##.width
let rect_height (rect : rect) = Js.float_of_number rect##.height
let style (element : element) = element##.style

let set_style (style : style) name value =
  let _ = style##setProperty name value Js.undefined in
  ()

let capture (element : element) (id : int) : unit =
  call element "setPointerCapture" [| inject id |]

let target (event : event) =
  Js.Opt.get event##.target (fun () -> failwith "Event has no element target")

let client_x event =
  let mouse : Html.mouseEvent Js.t = Js.Unsafe.coerce event in
  Js.float_of_number mouse##.clientX

let client_y event =
  let mouse : Html.mouseEvent Js.t = Js.Unsafe.coerce event in
  Js.float_of_number mouse##.clientY

let pointer_id (event : event) : int = Js.Unsafe.get event "pointerId"

let button event =
  let mouse : Html.mouseEvent Js.t = Js.Unsafe.coerce event in
  mouse##.button

let pointer_type_raw (event : event) : native_string =
  Js.Unsafe.get event "pointerType"

let prevent_default = Dom.preventDefault
let coalesced_raw (event : event) : events = call event "getCoalescedEvents" [||]
let events_length (events : events) : int = Js.Unsafe.get events "length"

let event_at (events : events) (index : int) : event =
  Js.Unsafe.get events index

let touches event =
  let touch_event : Html.touchEvent Js.t = Js.Unsafe.coerce event in
  touch_event##.touches

let touches_length (touches : touches) = touches##.length

let touch_at (touches : touches) index =
  Js.Optdef.get (touches##item index) (fun () -> invalid_arg "Touch index")

let touch_x (touch : touch) = Js.float_of_number touch##.clientX
let touch_y (touch : touch) = Js.float_of_number touch##.clientY

let delta_y event =
  let wheel : Html.wheelEvent Js.t = Js.Unsafe.coerce event in
  Js.float_of_number wheel##.deltaY

let delta_mode event =
  let wheel : Html.wheelEvent Js.t = Js.Unsafe.coerce event in
  match wheel##.deltaMode with
  | Html.Delta_pixel -> 0
  | Html.Delta_line -> 1
  | Html.Delta_page -> 2

let key_raw event =
  let keyboard : Html.keyboardEvent Js.t = Js.Unsafe.coerce event in
  Js.Optdef.get keyboard##.key (fun () -> Js.string "")

let ctrl event =
  let keyboard : Html.keyboardEvent Js.t = Js.Unsafe.coerce event in
  Js.to_bool keyboard##.ctrlKey

let meta event =
  let keyboard : Html.keyboardEvent Js.t = Js.Unsafe.coerce event in
  Js.to_bool keyboard##.metaKey

let shift event =
  let keyboard : Html.keyboardEvent Js.t = Js.Unsafe.coerce event in
  Js.to_bool keyboard##.shiftKey

let listener_options passive = { passive }

let coalesced event =
  if Js.Optdef.test (Js.Unsafe.get event "getCoalescedEvents") then
    Some (coalesced_raw event)
  else None

let reload () = Html.window##.location##reload

let save_png (bytes : Bindings.Uint8ClampedArray.t) =
  let options = Js.Unsafe.obj [| ("type", inject (Js.string "image/png")) |] in
  let blob =
    Js.Unsafe.new_obj
      (Js.Unsafe.get Js.Unsafe.global "Blob")
      [| inject (Js.array [| bytes |]); inject options |]
  in
  let url_api = Js.Unsafe.get Js.Unsafe.global "URL" in
  let url : native_string = call url_api "createObjectURL" [| inject blob |] in
  let link = create_element (document_of global) (Js.string "a") in
  Js.Unsafe.set link "href" url;
  Js.Unsafe.set link "download" (Js.string "little-paint.png");
  click link;
  let _ =
    timeout_raw
      (fun () ->
        let _ = call url_api "revokeObjectURL" [| inject url |] in
        ())
      1000.
  in
  ()

let exception_text = Printexc.to_string
