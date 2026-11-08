module B = Bindings

let keys = list{
  ("0", B.Key.Zero),
  ("1", B.Key.One),
  ("2", B.Key.Two),
  ("3", B.Key.Three),
  ("4", B.Key.Four),
  ("5", B.Key.Five),
  ("6", B.Key.Six),
  ("7", B.Key.Seven),
  ("8", B.Key.Eight),
  ("9", B.Key.Nine),
  (".", B.Key.Decimal),
  ("+", B.Key.Add),
  ("-", B.Key.Subtract),
  ("*", B.Key.Multiply),
  ("/", B.Key.Divide),
  ("Enter", B.Key.Equals),
  ("Escape", B.Key.Clear),
  ("Backspace", B.Key.Backspace),
  ("n", B.Key.Negate),
  ("%", B.Key.Percent),
}

let start = () => {
  let calculator = B.Calculator.create()
  try {
    let output = Ui.by_id("result")
    let render = () => Ui.text(output, B.Calculator.display(calculator))
    let press = key => {
      B.Calculator.press(calculator, key)
      render()
    }
    List.iter(((name, key)) => Ui.on_click(Ui.by_id("key-" ++ name), () => press(key)), keys)
    Ui.on_key(name => {
      let name = if name == "=" {
        "Enter"
      } else {
        name
      }
      switch List.assoc_opt(name, keys) {
      | Some(key) =>
        press(key)
        true
      | None => false
      }
    })
    Ui.on_close(() => B.Calculator.dispose(calculator))
    render()
  } catch {
  | error =>
    B.Calculator.dispose(calculator)
    raise(error)
  }
}

let () = Async.observe_error(
  B.Wasm.load("./backend.wasm"),
  loadResult =>
    switch loadResult {
    | Error(error) => Ui.failed(B.Wasm.Load_error.message(error))
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
