type element
type event
type rect
type style
type native_string
type events
type touches
type touch
type listener_options = { passive : bool }

external native : string -> native_string = "%identity"

external to_host : string -> native_string = "host_string"
[@@mel.scope "Ocamljs_image_editor_demo"]

external of_host : native_string -> string = "native_string"
[@@mel.scope "Ocamljs_image_editor_demo"]

external global : element = "globalThis"
external document_of : element -> element = "document" [@@mel.get]
external document_element : element -> element = "documentElement" [@@mel.get]
external body : element -> element = "body" [@@mel.get]

external find_id_raw : element -> native_string -> element Js.Nullable.t
  = "getElementById"
[@@mel.send]

let find_id document id = Js.Nullable.toOption (find_id_raw document id)

external create_element : element -> native_string -> element = "createElement"
[@@mel.send]

external set_text : element -> native_string -> unit = "textContent" [@@mel.set]
external append_child : element -> element -> unit = "appendChild" [@@mel.send]
external get_value : element -> native_string = "value" [@@mel.get]
external set_value : element -> native_string -> unit = "value" [@@mel.set]

external set_attribute : element -> native_string -> native_string -> unit
  = "setAttribute"
[@@mel.send]

external get_attribute_raw :
  element -> native_string -> native_string Js.Nullable.t = "getAttribute"
[@@mel.send]

let get_attribute element name =
  Js.Nullable.toOption (get_attribute_raw element name)

external closest_raw : element -> native_string -> element Js.Nullable.t
  = "closest"
[@@mel.send]

let closest element selector =
  Js.Nullable.toOption (closest_raw element selector)

external click : element -> unit = "click" [@@mel.send]

external listen_raw : element -> native_string -> (event -> unit) -> unit
  = "addEventListener"
[@@mel.send]

external listen_options :
  element -> native_string -> (event -> unit) -> listener_options -> unit
  = "addEventListener"
[@@mel.send]

external frame_raw : (float -> unit) -> int = "requestAnimationFrame"
external cancel_frame : int -> unit = "cancelAnimationFrame"
external timeout_raw : (unit -> unit) -> float -> int = "setTimeout"
external set_disabled : element -> bool -> unit = "disabled" [@@mel.set]
external disabled : element -> bool = "disabled" [@@mel.get]
external set_hidden : element -> bool -> unit = "hidden" [@@mel.set]
external width : element -> float = "width" [@@mel.get]
external height : element -> float = "height" [@@mel.get]
external client_width : element -> float = "clientWidth" [@@mel.get]
external client_height : element -> float = "clientHeight" [@@mel.get]
external bounds : element -> rect = "getBoundingClientRect" [@@mel.send]
external left : rect -> float = "left" [@@mel.get]
external top : rect -> float = "top" [@@mel.get]
external rect_width : rect -> float = "width" [@@mel.get]
external rect_height : rect -> float = "height" [@@mel.get]
external style : element -> style = "style" [@@mel.get]

external set_style : style -> native_string -> native_string -> unit
  = "setProperty"
[@@mel.send]

external capture : element -> int -> unit = "setPointerCapture" [@@mel.send]
external target : event -> element = "target" [@@mel.get]
external client_x : event -> float = "clientX" [@@mel.get]
external client_y : event -> float = "clientY" [@@mel.get]
external pointer_id : event -> int = "pointerId" [@@mel.get]
external button : event -> int = "button" [@@mel.get]
external pointer_type_raw : event -> native_string = "pointerType" [@@mel.get]
external prevent_default : event -> unit = "preventDefault" [@@mel.send]
external coalesced_raw : event -> events = "getCoalescedEvents" [@@mel.send]
external events_length : events -> int = "length" [@@mel.get]
external event_at : events -> int -> event = "" [@@mel.get_index]
external touches : event -> touches = "touches" [@@mel.get]
external touches_length : touches -> int = "length" [@@mel.get]
external touch_at : touches -> int -> touch = "item" [@@mel.send]
external touch_x : touch -> float = "clientX" [@@mel.get]
external touch_y : touch -> float = "clientY" [@@mel.get]
external delta_y : event -> float = "deltaY" [@@mel.get]
external delta_mode : event -> int = "deltaMode" [@@mel.get]
external key_raw : event -> native_string = "key" [@@mel.get]
external ctrl : event -> bool = "ctrlKey" [@@mel.get]
external meta : event -> bool = "metaKey" [@@mel.get]
external shift : event -> bool = "shiftKey" [@@mel.get]

let listener_options passive = { passive }

external coalesced_member : event -> (unit -> unit) Js.Undefined.t
  = "getCoalescedEvents"
[@@mel.get]

let coalesced event =
  match Js.Undefined.toOption (coalesced_member event) with
  | None -> None
  | Some _ -> Some (coalesced_raw event)

external stringify_raw : 'a -> native_string = "String"

let error_text value = of_host (stringify_raw value)

external location : element -> element = "location" [@@mel.get]
external reload_location : element -> unit = "reload" [@@mel.send]

let reload () = reload_location (location global)

type blob

external blob :
  Bindings.Uint8ClampedArray.t array -> native_string Js.Dict.t -> blob = "Blob"
[@@mel.new]

external object_url : blob -> native_string = "createObjectURL"
[@@mel.scope "URL"]

external revoke_url : native_string -> unit = "revokeObjectURL"
[@@mel.scope "URL"]

external href : element -> native_string -> unit = "href" [@@mel.set]
external download : element -> native_string -> unit = "download" [@@mel.set]

let save_png bytes =
  let options = Js.Dict.empty () in
  Js.Dict.set options "type" (to_host "image/png");
  let url = object_url (blob [| bytes |] options) in
  let link = create_element (document_of global) (to_host "a") in
  href link url;
  download link (to_host "little-paint.png");
  click link;
  let _ = timeout_raw (fun () -> revoke_url url) 1000. in
  ()

let exception_text error =
  match error with
  | Failure reason | Invalid_argument reason -> reason
  | _ -> (
      match Js.Exn.asJsExn error with
      | Some host -> (
          match Js.Exn.message host with
          | Some reason -> of_host (native reason)
          | None -> "Browser operation failed")
      | None -> "Application failed")
