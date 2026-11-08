type element
type event

external global: element = "globalThis"
@get external document_of: element => element = "document"
@get external document_element: element => element = "documentElement"
@get external body: element => element = "body"
@send external find_id_raw: (element, string) => Js.Nullable.t<element> = "getElementById"
let find_id = (document, id) => Js.Nullable.toOption(find_id_raw(document, id))
@send external create_element: (element, string) => element = "createElement"
@set external set_text: (element, string) => unit = "textContent"
@send external replace_children: element => unit = "replaceChildren"
@send external append_child: (element, element) => unit = "appendChild"
@get external get_value: element => string = "value"
@send external set_attribute: (element, string, string) => unit = "setAttribute"
@send external listen_raw: (element, string, event => unit) => unit = "addEventListener"
external error_text: 'a => string = "String"
@get external location: element => element = "location"
@send external reload_location: element => unit = "reload"
let reload = () => reload_location(location(global))

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
