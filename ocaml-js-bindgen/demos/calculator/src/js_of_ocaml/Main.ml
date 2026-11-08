module B = Bindings
module Js = Js_of_ocaml.Js
module Dom = Js_of_ocaml.Dom
module Html = Js_of_ocaml.Dom_html

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
    let output = Html.getElementById_exn "result" in
    let render () =
      output##.textContent :=
        Js.some (Js.string (B.Calculator.display calculator))
    in
    let press key =
      B.Calculator.press calculator key;
      render ()
    in
    List.iter
      (fun (name, key) ->
        let button = Html.getElementById_exn ("key-" ^ name) in
        button##.onclick :=
          Html.handler (fun _ ->
              press key;
              Js._false))
      keys;
    Html.document##.onkeydown :=
      Html.handler (fun event ->
          if Js.to_bool event##.ctrlKey || Js.to_bool event##.metaKey then
            Js._true
          else
            let name =
              Js.to_string (Js.Optdef.get event##.key (fun () -> Js.string ""))
            in
            let name = if name = "=" then "Enter" else name in
            match List.assoc_opt name keys with
            | Some key ->
                press key;
                Js._false
            | None -> Js._true);
    let _ =
      Dom.addEventListener Html.window
        (Dom.Event.make "pagehide")
        (Html.handler (fun event ->
             if not (Js.to_bool (Js.Unsafe.get event "persisted")) then
               B.Calculator.dispose calculator;
             Js._true))
        Js._false
    in
    render ();
    Html.document##.documentElement##setAttribute
      (Js.string "data-ready") (Js.string "true")
  with error ->
    B.Calculator.dispose calculator;
    raise error

let failed error =
  let message = Html.createPre Html.document in
  message##.textContent :=
    Js.some
      (Js.string ("Could not start calculator: " ^ Printexc.to_string error));
  let reload = Html.createButton Html.document in
  reload##.textContent := Js.some (Js.string "Reload");
  reload##.onclick :=
    Html.handler (fun _ ->
        Html.window##.location##reload;
        Js._false);
  Dom.appendChild Html.document##.body message;
  Dom.appendChild Html.document##.body reload

let () =
  let module P = Js_of_ocaml.Promise in
  let _ =
    P.catch
      (fun error ->
        let reason =
          Js.Unsafe.fun_call
            (Js.Unsafe.get Js.Unsafe.global "String")
            [| P.error_to_any error |]
        in
        failed (Failure (Js.to_string reason));
        P.resolve ())
      (P.map
         (function
           | Ok () -> start ()
           | Error error -> failwith (B.Wasm.Load_error.message error))
         (B.Wasm.load "./backend.wasm"))
  in
  ()
