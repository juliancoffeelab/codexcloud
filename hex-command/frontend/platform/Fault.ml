let throw_js : Js.Exn.t -> 'a = [%mel.raw {|error => { throw error; }|}]

let raise_original error =
  match Js.Exn.asJsExn error with Some original -> throw_js original | None -> raise error

external log : 'a -> unit = "error" [@@mel.scope "console"]

let location (file, line, character) width message =
  "File \"" ^ file ^ "\", line " ^ string_of_int line ^ ", characters " ^ string_of_int character
  ^ "-"
  ^ string_of_int (character + width)
  ^ ": " ^ message

let describe error =
  match Js.Exn.asJsExn error with
  | Some original ->
      log original;
      Utf8.of_js
        (Option.value
           ~default:(Option.value ~default:"JavaScript exception" (Js.Exn.name original))
           (Js.Exn.message original))
  | None -> (
      log error;
      match error with
      | Failure message | Invalid_argument message | Sys_error message -> message
      | Match_failure position -> location position 5 "Pattern matching failed"
      | Assert_failure position -> location position 6 "Assertion failed"
      | Undefined_recursive_module position -> location position 6 "Undefined recursive module"
      | Out_of_memory -> "Out of memory"
      | Stack_overflow -> "Stack overflow"
      | _ -> Printexc.exn_slot_name error)
