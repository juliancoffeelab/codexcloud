module B = Bindings

let start = () => {
  let universe = switch B.Universe.create(20., 14.) {
  | Ok(u) => u
  | Error(_) => failwith("Invalid grid")
  }

  List.iter(x => ignore(B.Universe.toggle(universe, x, 7.)), list{8., 9., 10.})
  let grid = Ui.by_id("grid")
  let rec render = () => {
    let s = B.Universe.snapshot(universe)
    Ui.clear(grid)
    Ui.text(Ui.by_id("generation"), "Generation " ++ string_of_int(int_of_float(s.generation)))
    Array.iteri((index, alive) => {
      let x = float_of_int(mod(index, 20)) and y = float_of_int(index / 20)
      let cell = Ui.button("", () => {
        ignore(B.Universe.toggle(universe, x, y))
        render()
      })

      Ui.class_name(
        cell,
        if alive {
          "cell alive"
        } else {
          "cell"
        },
      )
      Ui.set_attribute(
        cell,
        "aria-label",
        "Cell " ++ (string_of_int(int_of_float(x)) ++ ("," ++ string_of_int(int_of_float(y)))),
      )
      Ui.append(grid, cell)
    }, s.cells)
  }

  Ui.on_click(Ui.by_id("step"), () => {
    ignore(B.Universe.step(universe))
    render()
  })
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
