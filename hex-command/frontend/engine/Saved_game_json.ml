(* Persistence is decoded by Rust serde; historical conversions remain data-only JS. *)
let decode value =
  match Domain.checkpoint_from_json (Utf8.of_js (Js.Json.stringify value)) with
  | Ok checkpoint -> Ok checkpoint
  | Error error ->
      Error
        (match error with
        | Domain.Engine_error.Invalid_save { message } -> message
        | _ -> "Invalid saved battle")
