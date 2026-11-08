module B = Bindings

type element

external by_id : string -> element = "getElementById" [@@mel.scope "document"]
external root : element = "documentElement" [@@mel.scope "document"]
external set_text : element -> string -> unit = "textContent" [@@mel.set]

external set_attribute : element -> string -> string -> unit = "setAttribute"
[@@mel.send]

external number : element -> float = "valueAsNumber" [@@mel.get]
external on_input : element -> (unit -> unit) -> unit = "oninput" [@@mel.set]

let start () =
  let a = by_id "a" and b = by_id "b" and output = by_id "result" in
  let render () =
    let a = number a and b = number b in
    let text =
      if a >= 0. && a < infinity && b >= 0. && b < infinity then
        let value = B.hypotenuse a b in
        if value < infinity then Js.Float.toString value
        else "Result is too large"
      else "Enter two nonnegative, finite numbers"
    in
    set_text output text
  in
  on_input a render;
  on_input b render;
  render ();
  set_attribute root "data-ready" "true"

let () =
  let module P = Js.Promise in
  let _ =
    B.Wasm.load "./backend.wasm"
    |> P.then_ (fun result ->
        (match result with
        | Ok () -> start ()
        | Error error ->
            set_text (by_id "result") (B.Wasm.Load_error.message error));
        P.resolve ())
    |> P.catch (fun _ ->
        set_text (by_id "result") "Could not load Wasm";
        P.resolve ())
  in
  ()
