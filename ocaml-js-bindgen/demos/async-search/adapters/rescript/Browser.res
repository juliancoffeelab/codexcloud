type element
type event
type worker
type worker_options = {@as("type") kind: string}
type message = {kind: string, value: string}

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
@new external new_worker: (string, worker_options) => worker = "Worker"
@send external terminate: worker => unit = "terminate"
@send external post: (worker, message) => unit = "postMessage"
@set external worker_message: (worker, event => unit) => unit = "onmessage"
@set external worker_error: (worker, event => unit) => unit = "onerror"
@set external worker_message_error: (worker, event => unit) => unit = "onmessageerror"
external post_global: message => unit = "postMessage"
external close_worker: unit => unit = "close"
let worker_options = () => {kind: "module"}
let message_valid = (message, limit) =>
  Js.typeof(message.kind) == "string" &&
  Js.typeof(message.value) == "string" &&
  String.length(message.kind) <= 32 &&
  String.length(message.value) <= limit
@get external data_raw: event => Js.Nullable.t<message> = "data"
let data = event => Js.Nullable.toOption(data_raw(event))
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

@get external rejection_value: event => 'a = "reason"
let rejection_text = event => error_text(rejection_value(event))

@send external prevent_default: event => unit = "preventDefault"
