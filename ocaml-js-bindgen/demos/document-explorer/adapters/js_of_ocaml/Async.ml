module P = Js_of_ocaml.Promise

let observe_error promise success failure =
  let _ =
    P.then_
      ~on_error:(fun error ->
        let module J = Js_of_ocaml.Js in
        let reason =
          J.Unsafe.fun_call
            (J.Unsafe.get J.Unsafe.global "String")
            [| P.error_to_any error |]
        in
        failure (J.to_string reason);
        P.resolve ())
      (fun value ->
        success value;
        P.resolve ())
      promise
  in
  ()

let observe promise success failure =
  observe_error promise success (fun _ -> failure ())
