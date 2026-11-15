let throw_js : Js.Exn.t -> 'a = [%mel.raw {|error => { throw error; }|}]

let raise_original error =
  match Js.Exn.asJsExn error with Some original -> throw_js original | None -> raise error

external log : 'a -> unit = "error" [@@mel.scope "console"]

let location (file, line, character) width message =
  let js_text_part_3 = string_of_int line in
  let js_text_part_5 = string_of_int character in
  let js_text_part_7 = string_of_int (character + width) in
  {j|File "$(file)", line $(js_text_part_3), characters $(js_text_part_5)-$(js_text_part_7): $(message)|j}

let describe error =
  match Js.Exn.asJsExn error with
  | Some original ->
      log original;
      Option.value
        ~default:(Option.value ~default:{js|JavaScript exception|js} (Js.Exn.name original))
        (Js.Exn.message original)
  | None -> (
      log error;
      match error with
      | Failure message | Invalid_argument message | Sys_error message -> message
      | Match_failure position -> location position 5 {js|Pattern matching failed|js}
      | Assert_failure position -> location position 6 {js|Assertion failed|js}
      | Undefined_recursive_module position ->
          location position 6 {js|Undefined recursive module|js}
      | Out_of_memory -> {js|Out of memory|js}
      | Stack_overflow -> {js|Stack overflow|js}
      | _ -> Printexc.exn_slot_name error)
