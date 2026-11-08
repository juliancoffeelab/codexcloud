module B = Bindings

let keys =
  [
    ("0", B.Key.Zero);
    ("1", B.Key.One);
    ("2", B.Key.Two);
    ("3", B.Key.Three);
    ("4", B.Key.Four);
    ("5", B.Key.Five);
    ("6", B.Key.Six);
    ("7", B.Key.Seven);
    ("8", B.Key.Eight);
    ("9", B.Key.Nine);
    (".", B.Key.Decimal);
    ("+", B.Key.Add);
    ("-", B.Key.Subtract);
    ("*", B.Key.Multiply);
    ("/", B.Key.Divide);
    ("Enter", B.Key.Equals);
    ("Escape", B.Key.Clear);
    ("Backspace", B.Key.Backspace);
    ("n", B.Key.Negate);
    ("%", B.Key.Percent);
  ]

let start () =
  let calculator = B.Calculator.create () in
  try
    let output = Ui.by_id "result" in
    let render () = Ui.text output (B.Calculator.display calculator) in
    let press key =
      B.Calculator.press calculator key;
      render ()
    in
    List.iter
      (fun (name, key) -> Ui.on_click (Ui.by_id ("key-" ^ name)) (fun () -> press key))
      keys;
    Ui.on_key (fun name ->
        let name = if name = "=" then "Enter" else name in
        match List.assoc_opt name keys with
        | Some key ->
            press key;
            true
        | None -> false);
    Ui.on_close (fun () -> B.Calculator.dispose calculator);
    render ()
  with error ->
    B.Calculator.dispose calculator;
    raise error

let () =
  Async.observe_error
    (B.Wasm.load "./backend.wasm")
    (function
      | Error error -> Ui.failed (B.Wasm.Load_error.message error)
      | Ok () ->
      try
        start ();
        Ui.ready ()
      with error -> Ui.failed (Browser.exception_text error))
    Ui.failed
