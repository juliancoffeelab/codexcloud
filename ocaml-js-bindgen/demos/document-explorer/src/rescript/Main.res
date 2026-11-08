let start = () =>
  App.start((blob, success, failure) =>
    try Async.observe(Bindings.Document.of_blob(blob), success, failure) catch {
    | _ => failure()
    }
  )

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
