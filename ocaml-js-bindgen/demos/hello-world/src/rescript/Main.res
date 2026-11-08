module B = Bindings

type element
@scope("document") external by_id: string => element = "getElementById"
@scope("document") external root: element = "documentElement"
@set external set_text: (element, string) => unit = "textContent"
@send external set_attribute: (element, string, string) => unit = "setAttribute"

@get external number: element => float = "valueAsNumber"
@set external on_input: (element, unit => unit) => unit = "oninput"

let start = () => {
  let a = by_id("a") and b = by_id("b") and output = by_id("result")
  let render = () => {
    let a = number(a) and b = number(b)
    let text = if a >= 0. && Js.Float.isFinite(a) && b >= 0. && Js.Float.isFinite(b) {
      let value = B.hypotenuse(a, b)
      if Js.Float.isFinite(value) {
        Js.Float.toString(value)
      } else {
        "Result is too large"
      }
    } else {
      "Enter two nonnegative, finite numbers"
    }
    set_text(output, text)
  }
  on_input(a, render)
  on_input(b, render)
  render()
  set_attribute(root, "data-ready", "true")
}

let () = ignore(
  Js.Promise.catch(
    _ => {
      set_text(by_id("result"), "Could not load Wasm")
      Js.Promise.resolve()
    },
    Js.Promise.then_(result => {
      switch result {
      | Ok() => start()
      | Error(error) => set_text(by_id("result"), B.Wasm.Load_error.message(error))
      }
      Js.Promise.resolve()
    }, B.Wasm.load("./backend.wasm")),
  ),
)
