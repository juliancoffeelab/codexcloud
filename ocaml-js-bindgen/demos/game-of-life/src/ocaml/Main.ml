module B = Bindings

let start () =
  let universe =
    match B.Universe.create 20. 14. with
    | Ok u -> u
    | Error _ -> failwith "Invalid grid"
  in
  List.iter
    (fun x ->
      let _ = B.Universe.toggle universe x 7. in
      ())
    [ 8.; 9.; 10. ];
  let grid = Ui.by_id "grid" in
  let rec render () =
    let s = B.Universe.snapshot universe in
    Ui.clear grid;
    Ui.text (Ui.by_id "generation")
      ("Generation " ^ string_of_int (int_of_float s.generation));
    Array.iteri
      (fun index alive ->
        let x = float_of_int (index mod 20) and y = float_of_int (index / 20) in
        let cell =
          Ui.button "" (fun () ->
              let _ = B.Universe.toggle universe x y in
              render ())
        in
        Ui.class_name cell (if alive then "cell alive" else "cell");
        Ui.set_attribute cell "aria-label"
          ("Cell "
          ^ string_of_int (int_of_float x)
          ^ ","
          ^ string_of_int (int_of_float y));
        Ui.append grid cell)
      s.cells
  in
  Ui.on_click (Ui.by_id "step") (fun () ->
      let _ = B.Universe.step universe in
      render ());
  render ()

let () =
  Async.observe_error
    (Bindings.Wasm.load "./backend.wasm")
    (function
      | Error error -> Ui.failed (Bindings.Wasm.Load_error.message error)
      | Ok () -> (
          try
            start ();
            Ui.ready ()
          with error -> Ui.failed (Browser.exception_text error)))
    Ui.failed
