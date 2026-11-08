module B = Bindings
module P = Js_of_ocaml.Promise

let ( let* ) pending fn = P.bind fn pending

(* Test-only: inspect exceptions raised by the compiled throws/callback fixtures.
   Matching the expected constructor and payload is the assertion; unmatched
   rejections are propagated. Never use this bridge for general JS errors. *)
external exception_of_any : Js_of_ocaml.Js.Unsafe.any -> exn = "%identity"

let () =
  let _ =
    let* result = B.fetch_cat "https://fixture/cat" in
    (match result with
    | Ok (cat : B.cat) -> assert (cat.id = "λ🦀")
    | Error _ -> assert false);
    let* result = B.fetch_cat "https://fixture/bad" in
    assert (result = Error B.Cat_error.Invalid_response);
    let* result = B.fetch_cat "https://fixture/failure" in
    assert (result = Error B.Cat_error.Network);
    let* () =
      P.catch
        (fun error ->
          match exception_of_any (P.error_to_any error) with
          | B.Fetch_cat_throw_error B.Cat_error.Invalid_response -> P.resolve ()
          | _ -> P.reject error)
        (P.bind
           (fun _ -> assert false)
           (B.fetch_cat_throw "https://fixture/bad"))
    in
    let module J = Js_of_ocaml.Js in
    let () : unit =
      J.Unsafe.meth_call (J.Unsafe.get J.Unsafe.global "OcamljsTest") "done" [||]
    in
    P.resolve ()
  in
  ()
