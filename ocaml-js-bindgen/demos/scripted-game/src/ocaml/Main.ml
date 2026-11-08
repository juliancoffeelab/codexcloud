module B = Bindings

let wall_name = function B.Wall.Left -> "left" | B.Wall.Right -> "right"

let start () =
  let w = B.World.create () in
  let pending = ref 0 and running = ref true in
  let message s = Ui.text (Ui.by_id "message") s in
  B.World.set_handler w (fun (event : B.collision) ->
      pending := !pending + 10;
      try
        let _ = B.World.snapshot w in
        message "Unexpected reentry"
      with _ ->
        message
          ("OCaml collision hook: " ^ wall_name event.wall
         ^ "; reentry rejected; reward queued"));
  let render () =
    let s = B.World.snapshot w in
    Ui.text (Ui.by_id "score") ("Score " ^ string_of_int (int_of_float s.score));
    Ui.text (Ui.by_id "tick-count")
      ("Tick " ^ string_of_int (int_of_float s.tick));
    let board = Ui.by_id "world" in
    Ui.clear board;
    for x = 0 to 7 do
      let cell = Ui.create "span" in
      Ui.text cell (if float_of_int x = s.x then "●" else "·");
      Ui.append board cell
    done
  in
  Ui.on_click (Ui.by_id "step") (fun () ->
      if !running then (
        let _ = B.World.tick w in
        if !pending <> 0 then (
          B.World.award w (float_of_int !pending);
          pending := 0);
        render ()));
  Ui.on_click (Ui.by_id "stop") (fun () ->
      if !running then (
        B.World.clear_handler w;
        running := false;
        B.World.dispose w;
        message "Handler and world released"));
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
