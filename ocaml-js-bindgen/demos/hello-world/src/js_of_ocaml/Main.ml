module B = Bindings
module Js = Js_of_ocaml.Js
module Html = Js_of_ocaml.Dom_html

let start () =
  let a = Option.get (Html.getElementById_coerce "a" Html.CoerceTo.input)
  and b = Option.get (Html.getElementById_coerce "b" Html.CoerceTo.input)
  and output = Html.getElementById_exn "result" in
  let render () =
    let a = Js.float_of_number a##.valueAsNumber
    and b = Js.float_of_number b##.valueAsNumber in
    let text =
      if a >= 0. && a < infinity && b >= 0. && b < infinity then
        let value = B.hypotenuse a b in
        if value < infinity then
          Js.to_string (Js.number_of_float value)##toString
        else "Result is too large"
      else "Enter two nonnegative, finite numbers"
    in
    output##.textContent := Js.some (Js.string text)
  in
  a##.oninput :=
    Html.handler (fun _ ->
        render ();
        Js._true);
  b##.oninput :=
    Html.handler (fun _ ->
        render ();
        Js._true);
  render ();
  Html.document##.documentElement##setAttribute
    (Js.string "data-ready") (Js.string "true")

let () =
  let module P = Js_of_ocaml.Promise in
  let _ =
    P.catch
      (fun _ ->
        let output = Html.getElementById_exn "result" in
        output##.textContent := Js.some (Js.string "Could not load Wasm");
        P.resolve ())
      (P.map
         (function
           | Ok () -> start ()
           | Error error -> failwith (B.Wasm.Load_error.message error))
         (B.Wasm.load "./backend.wasm"))
  in
  ()
