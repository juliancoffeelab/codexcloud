use super::*;

/// Initialization is a runtime operation, with no dependency on application codecs.
pub(super) fn generate(o: &Options) -> (String, String, String) {
    let platform = if matches!(o.backend, Backend::JsOfOcaml) {
        "Js_of_ocaml.Promise"
    } else {
        "Js.Promise"
    };
    let mut helpers = format!("type foreign_error = {platform}.error\ntype load_result\n");
    for (name, args, ret) in [
        (
            "load",
            vec!["string"],
            if matches!(o.backend, Backend::JsOfOcaml) {
                "load_result js_promise"
            } else {
                "load_result Js.Promise.t"
            },
        ),
        (
            "observe_load",
            vec![
                if matches!(o.backend, Backend::JsOfOcaml) {
                    "load_result js_promise"
                } else {
                    "load_result Js.Promise.t"
                },
                "(load_result -> unit)",
                "(foreign_error -> unit)",
            ],
            "unit",
        ),
        ("load_ok", vec!["load_result"], "bool"),
        ("load_kind", vec!["load_result"], "string"),
        ("load_url", vec!["load_result"], "string"),
        ("load_status", vec!["load_result"], "float"),
        ("load_expected", vec!["load_result"], "string"),
        ("load_actual", vec!["load_result"], "string"),
        ("load_cause", vec!["load_result"], "foreign_error"),
        ("foreign_detail", vec!["foreign_error"], "string"),
        ("module_failure_cause", vec!["unit"], "foreign_error"),
    ] {
        helpers.push_str(&ml::external(o, name, name, &args, ret));
    }
    let common = format!(
        r#"  module Foreign_error = struct
    type t = {platform}.error
    let describe = Internal.foreign_detail
  end
  module Failure = struct
    type t = {{ message : string; cause : Foreign_error.t }}
  end
  module Load_error = struct
    type t =
      | Fetch of {{ url : string; cause : Foreign_error.t }}
      | Http of {{ url : string; status : int }}
      | Compile of Foreign_error.t
      | Instantiate of Foreign_error.t
      | Schema_mismatch of {{ expected : string; actual : string }}
      | Abi_mismatch of {{ expected : string; actual : string }}
      | Mode_mismatch of {{ expected : string; actual : string }}
      | Poisoned of Failure.t
      | Unexpected of Foreign_error.t
    let message = function
      | Fetch {{ url; cause }} -> "Could not fetch Wasm at " ^ url ^ ": " ^ Foreign_error.describe cause
      | Http {{ url; status }} -> "Wasm download failed: HTTP " ^ string_of_int status ^ " at " ^ url
      | Compile cause -> "Wasm compilation failed: " ^ Foreign_error.describe cause
      | Instantiate cause -> "Wasm instantiation failed: " ^ Foreign_error.describe cause
      | Schema_mismatch {{ expected; actual }} -> "Schema fingerprint mismatch: expected " ^ expected ^ ", received " ^ actual
      | Abi_mismatch {{ expected; actual }} -> "ABI version mismatch: expected " ^ expected ^ ", received " ^ actual
      | Mode_mismatch {{ expected; actual }} -> "Paranoid mode mismatch: expected " ^ expected ^ ", received " ^ actual
      | Poisoned failure -> failure.message
      | Unexpected cause -> Foreign_error.describe cause
  end
  let failure () = if Internal.module_failed () then
    Some {{ Failure.message = Internal.module_failure (); cause = Internal.module_failure_cause () }} else None
  let decode_load raw =
    if Internal.load_ok raw then Ok () else Error (
      match Internal.load_kind raw with
      | "fetch" -> Load_error.Fetch {{ url = Internal.load_url raw; cause = Internal.load_cause raw }}
      | "http" -> Load_error.Http {{ url = Internal.load_url raw; status = int_of_float (Internal.load_status raw) }}
      | "compile" -> Load_error.Compile (Internal.load_cause raw)
      | "instantiate" -> Load_error.Instantiate (Internal.load_cause raw)
      | "schema" -> Load_error.Schema_mismatch {{ expected = Internal.load_expected raw; actual = Internal.load_actual raw }}
      | "abi" -> Load_error.Abi_mismatch {{ expected = Internal.load_expected raw; actual = Internal.load_actual raw }}
      | "mode" -> Load_error.Mode_mismatch {{ expected = Internal.load_expected raw; actual = Internal.load_actual raw }}
      | "poisoned" -> Load_error.Poisoned {{ Failure.message = Internal.load_actual raw; cause = Internal.load_cause raw }}
      | _ -> Load_error.Unexpected (Internal.load_cause raw))
"#
    );
    let load = if o.uses_lwt() {
        "  let load url =\n    let promise, resolver = Lwt.task () in\n    let complete value = if Lwt.is_sleeping promise then Lwt.wakeup_later resolver value in\n    Internal.observe_load (Internal.load url)\n      (fun raw -> complete (decode_load raw))\n      (fun cause -> complete (Error (Load_error.Unexpected cause)));\n    promise\n"
    } else if matches!(o.backend, Backend::JsOfOcaml) {
        "  let load url = Js_of_ocaml.Promise.make (fun ~resolve ~reject:_ ->\n    Internal.observe_load (Internal.load url)\n      (fun raw -> resolve (decode_load raw))\n      (fun cause -> resolve (Error (Load_error.Unexpected cause))))\n"
    } else {
        "  let load url = Js.Promise.make (fun ~resolve ~reject:_ ->\n    Internal.observe_load (Internal.load url)\n      (fun raw -> (resolve (decode_load raw) [@u]))\n      (fun cause -> (resolve (Error (Load_error.Unexpected cause)) [@u])))\n"
    };
    let wasm = format!("module Wasm = struct\n{common}{load}end");
    let interface = format!(
        r#"module Wasm : sig
  module Foreign_error : sig
    type t = {platform}.error
    val describe : t -> string
  end
  module Failure : sig
    type t = {{ message : string; cause : Foreign_error.t }}
  end
  module Load_error : sig
    type t =
      | Fetch of {{ url : string; cause : Foreign_error.t }}
      | Http of {{ url : string; status : int }}
      | Compile of Foreign_error.t
      | Instantiate of Foreign_error.t
      | Schema_mismatch of {{ expected : string; actual : string }}
      | Abi_mismatch of {{ expected : string; actual : string }}
      | Mode_mismatch of {{ expected : string; actual : string }}
      | Poisoned of Failure.t
      | Unexpected of Foreign_error.t
    val message : t -> string
  end
  val load : string -> (unit, Load_error.t) result {}
  (** Load and validate the module. Expected failures resolve to typed errors; foreign causes remain opaque. Concurrent loads share initialization and failed initialization can be retried. *)
  val failure : unit -> Failure.t option
  (** A terminal Wasm trap invalidates the module and its owners. Retains the original cause and requires a fresh realm. Binding validation errors raise separately. *)
end
"#,
        if o.uses_lwt() {
            "Lwt.t"
        } else {
            if matches!(o.backend, Backend::JsOfOcaml) {
                "Js_of_ocaml.Promise.t"
            } else {
                "Js.Promise.t"
            }
        }
    );
    (helpers, interface, wasm)
}
