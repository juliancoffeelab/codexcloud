module B = Bindings

let wall_name = wall =>
  switch wall {
  | B.Wall.Left => "left"
  | B.Wall.Right => "right"
  }

let start = () => {
  let w = B.World.create()
  let pending = ref(0) and running = ref(true)
  let message = s => Ui.text(Ui.by_id("message"), s)
  B.World.set_handler(w, (event: B.collision) => {
    pending := pending.contents + 10
    try {
      ignore(B.World.snapshot(w))
      message("Unexpected reentry")
    } catch {
    | _ =>
      message(
        "OCaml collision hook: " ++ (wall_name(event.wall) ++ "; reentry rejected; reward queued"),
      )
    }
  })
  let render = () => {
    let s = B.World.snapshot(w)
    Ui.text(Ui.by_id("score"), "Score " ++ string_of_int(int_of_float(s.score)))
    Ui.text(Ui.by_id("tick-count"), "Tick " ++ string_of_int(int_of_float(s.tick)))
    let board = Ui.by_id("world")
    Ui.clear(board)
    for x in 0 to 7 {
      let cell = Ui.create("span")
      Ui.text(
        cell,
        if float_of_int(x) == s.x {
          "●"
        } else {
          "·"
        },
      )
      Ui.append(board, cell)
    }
  }

  Ui.on_click(Ui.by_id("step"), () =>
    if running.contents {
      ignore(B.World.tick(w))
      if pending.contents != 0 {
        B.World.award(w, float_of_int(pending.contents))
        pending := 0
      }
      render()
    }
  )
  Ui.on_click(Ui.by_id("stop"), () =>
    if running.contents {
      B.World.clear_handler(w)
      running := false
      B.World.dispose(w)
      message("Handler and world released")
    }
  )
  render()
}

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
