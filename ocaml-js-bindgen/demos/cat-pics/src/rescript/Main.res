module B = Bindings

let status = text => Ui.text(Ui.by_id("status"), text)
let load = async () => {
  status("Loading…")
  try {
    switch await B.fetch_cat(Ui.value(Ui.by_id("api-url"))) {
    | Ok(cat) =>
      Ui.set_attribute(Ui.by_id("cat"), "src", cat.url)
      Ui.set_attribute(Ui.by_id("cat"), "alt", "Cat " ++ cat.id)
      status("Cat " ++ cat.id)
    | Error(B.Cat_error.Network) => status("Network request failed")
    | Error(B.Cat_error.Too_large) => status("Response is too large")
    | Error(B.Cat_error.Invalid_response) => status("Invalid cat response")
    }
  } catch {
  | _ => status("Unexpected binding failure")
  }
}
let start = () => Ui.on_click(Ui.by_id("fetch"), () => load()->ignore)

let () = Async.observe_error(
  Bindings.Wasm.load("./backend.wasm"),
  loadResult =>
    switch loadResult {
    | Error(error) => Ui.failed(Bindings.Wasm.Load_error.message(error))
    | Ok() =>
      try {
        start()
        Ui.ready()
      } catch {
      | error => Ui.failed(Browser.exception_text(error))
      }
    },
  Ui.failed,
)
