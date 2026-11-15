(* Persistence is decoded by Rust serde; historical conversions remain data-only JS. *)
let decode value =
  match Domain.checkpoint_from_json (Js.Json.stringify value) with
  | Ok checkpoint -> Ok checkpoint
  | Error error ->
      Error
        (match error with
        | Domain.Engine_error.Invalid_save { message } -> message
        | _ -> {js|Invalid saved battle|js})
