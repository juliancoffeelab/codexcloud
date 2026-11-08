(* Typed exception fixtures; the JavaScript tests never construct OCaml exceptions. *)
type 'a described = { original : 'a; description : string }

exception Diagnostic of string

let samples () =
  Array.map
    (fun error -> Utf8.to_js (Fault.describe error))
    [|
      Failure "Précise λ🦀";
      Invalid_argument "bad argument";
      Match_failure ("sample.ml", 12, 3);
      Assert_failure ("sample.ml", 18, 4);
      Undefined_recursive_module ("sample.ml", 24, 5);
      Out_of_memory;
      Stack_overflow;
    |]

let generic () =
  let original = Diagnostic "retained diagnostic payload" in
  { original; description = Utf8.to_js (Fault.describe original) }

let foreign () =
  try Js.Exn.raiseError (Utf8.to_js "Browser λ🦀 failure")
  with error -> (
    let description = Utf8.to_js (Fault.describe error) in
    match Js.Exn.asJsExn error with
    | Some original -> { original; description }
    | None -> raise error)
