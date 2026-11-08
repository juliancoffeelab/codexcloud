type element
type event
type rect
type style
type events
type touches
type touch
type listener_options = {passive: bool}

external global: element = "globalThis"
@get external document_of: element => element = "document"
@get external document_element: element => element = "documentElement"
@get external body: element => element = "body"
@send external find_id_raw: (element, string) => Js.Nullable.t<element> = "getElementById"
let find_id = (document, id) => Js.Nullable.toOption(find_id_raw(document, id))
@send external create_element: (element, string) => element = "createElement"
@set external set_text: (element, string) => unit = "textContent"
@send external append_child: (element, element) => unit = "appendChild"
@get external get_value: element => string = "value"
@set external set_value: (element, string) => unit = "value"
@send external set_attribute: (element, string, string) => unit = "setAttribute"
@send
external get_attribute_raw: (element, string) => Js.Nullable.t<string> = "getAttribute"
let get_attribute = (element, name) => Js.Nullable.toOption(get_attribute_raw(element, name))
@send external closest_raw: (element, string) => Js.Nullable.t<element> = "closest"
let closest = (element, selector) => Js.Nullable.toOption(closest_raw(element, selector))
@send external click: element => unit = "click"
@send external listen_raw: (element, string, event => unit) => unit = "addEventListener"
@send
external listen_options: (element, string, event => unit, listener_options) => unit =
  "addEventListener"
external frame_raw: (float => unit) => int = "requestAnimationFrame"
external cancel_frame: int => unit = "cancelAnimationFrame"
external timeout_raw: (unit => unit, float) => int = "setTimeout"
@set external set_disabled: (element, bool) => unit = "disabled"
@get external disabled: element => bool = "disabled"
@set external set_hidden: (element, bool) => unit = "hidden"
@get external width: element => float = "width"
@get external height: element => float = "height"
@get external client_width: element => float = "clientWidth"
@get external client_height: element => float = "clientHeight"
@send external bounds: element => rect = "getBoundingClientRect"
@get external left: rect => float = "left"
@get external top: rect => float = "top"
@get external rect_width: rect => float = "width"
@get external rect_height: rect => float = "height"
@get external style: element => style = "style"
@send external set_style: (style, string, string) => unit = "setProperty"
@send external capture: (element, int) => unit = "setPointerCapture"
@get external target: event => element = "target"
@get external client_x: event => float = "clientX"
@get external client_y: event => float = "clientY"
@get external pointer_id: event => int = "pointerId"
@get external button: event => int = "button"
@get external pointer_type_raw: event => string = "pointerType"
@send external prevent_default: event => unit = "preventDefault"
@send external coalesced_raw: event => events = "getCoalescedEvents"
@get external events_length: events => int = "length"
@get_index external event_at: (events, int) => event = ""
@get external touches: event => touches = "touches"
@get external touches_length: touches => int = "length"
@send external touch_at: (touches, int) => touch = "item"
@get external touch_x: touch => float = "clientX"
@get external touch_y: touch => float = "clientY"
@get external delta_y: event => float = "deltaY"
@get external delta_mode: event => int = "deltaMode"
@get external key_raw: event => string = "key"
@get external ctrl: event => bool = "ctrlKey"
@get external meta: event => bool = "metaKey"
@get external shift: event => bool = "shiftKey"
let listener_options = passive => {passive: passive}
@get external coalesced_member: event => Js.Undefined.t<unit => unit> = "getCoalescedEvents"
let coalesced = event =>
  switch Js.Undefined.toOption(coalesced_member(event)) {
  | None => None
  | Some(_) => Some(coalesced_raw(event))
  }
external error_text: 'a => string = "String"
@get external location: element => element = "location"
@send external reload_location: element => unit = "reload"
let reload = () => reload_location(location(global))

type blob
@new external blob: (array<Bindings.Uint8ClampedArray.t>, Js.Dict.t<string>) => blob = "Blob"
@scope("URL") external object_url: blob => string = "createObjectURL"
@scope("URL") external revoke_url: string => unit = "revokeObjectURL"
@set external href: (element, string) => unit = "href"
@set external download: (element, string) => unit = "download"
let save_png = bytes => {
  let options = Js.Dict.empty()
  Js.Dict.set(options, "type", "image/png")
  let url = object_url(blob([bytes], options))
  let link = create_element(document_of(global), "a")
  href(link, url)
  download(link, "little-paint.png")
  click(link)
  ignore(timeout_raw(() => revoke_url(url), 1000.))
}

let exception_text = error =>
  switch error {
  | Failure(reason) | Invalid_argument(reason) => reason
  | _ =>
    switch Js.Exn.asJsExn(error) {
    | Some(host) =>
      switch Js.Exn.message(host) {
      | Some(reason) => reason
      | None => "Browser operation failed"
      }
    | None => "Application failed"
    }
  }
