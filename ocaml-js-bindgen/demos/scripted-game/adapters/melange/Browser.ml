type element
type event
type native_string
type message = { kind : native_string; value : native_string }

external native : string -> native_string = "%identity"

external to_host : string -> native_string = "host_string"
[@@mel.scope "Ocamljs_scripted_game_demo"]

external of_host : native_string -> string = "native_string"
[@@mel.scope "Ocamljs_scripted_game_demo"]

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
external replace_children : element -> unit = "replaceChildren" [@@mel.send]
external append_child : element -> element -> unit = "appendChild" [@@mel.send]

external set_attribute : element -> native_string -> native_string -> unit
  = "setAttribute"
[@@mel.send]

external listen_raw : element -> native_string -> (event -> unit) -> unit
  = "addEventListener"
[@@mel.send]

external post_global : message -> unit = "postMessage"
external close_worker : unit -> unit = "close"
external stringify_raw : 'a -> native_string = "String"

let error_text value = of_host (stringify_raw value)

external location : element -> element = "location" [@@mel.get]
external reload_location : element -> unit = "reload" [@@mel.send]

let reload () = reload_location (location global)

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

external rejection_value : event -> 'a = "reason" [@@mel.get]

let rejection_text event = error_text (rejection_value event)

external prevent_default : event -> unit = "preventDefault" [@@mel.send]
