(* Typed exception fixtures; the JavaScript tests never construct OCaml exceptions. *)
type 'a described = { original : 'a; description : string }

exception Diagnostic of string

let samples () =
  Array.map
    (fun error -> Fault.describe error)
    [|
      Failure {js|Précise λ🦀|js};
      Invalid_argument {js|bad argument|js};
      Match_failure ({js|sample.ml|js}, 12, 3);
      Assert_failure ({js|sample.ml|js}, 18, 4);
      Undefined_recursive_module ({js|sample.ml|js}, 24, 5);
      Out_of_memory;
      Stack_overflow;
    |]

let generic () =
  let original = Diagnostic {js|retained diagnostic payload|js} in
  { original; description = Fault.describe original }

let foreign () =
  try Js.Exn.raiseError {js|Browser λ🦀 failure|js}
  with error -> (
    let description = Fault.describe error in
    match Js.Exn.asJsExn error with
    | Some original -> { original; description }
    | None -> raise error)
