open Browser_dom

let field id action = Js.Nullable.toOption (byId id) |> Option.iter action
let detail = Foreign_error.describe

external set_busy : Dom.element -> string -> unit = "ariaBusy" [@@mel.set]

let report reason =
  Boot_process.fail reason;
  field {js|startup|js} (fun node -> set_busy node {js|false|js});
  field {js|startup-loading|js} (fun node -> hidden node true);
  field {js|startup-failure|js} (fun node -> hidden node false);
  field {js|startup-message|js} (fun node ->
      set_text node {js|The game could not start. Reloading keeps your saved battle.|js});
  field {js|startup-error|js} (fun node ->
      hidden node false;
      open_ node true);
  field {js|startup-detail|js} (fun node -> set_text node reason)

let load : unit -> unit Js.Promise.t = [%mel.raw {|() => import("./Main.js")|}]

type wasm_module

let load_wasm : unit -> wasm_module Js.Promise.t =
  [%mel.raw {|() => import("@wasm/bindings.runtime.mjs")|}]

type wasm_instance

external initialize : wasm_module -> wasm_instance Js.Promise.t = "initialize" [@@mel.send]

let () =
  let wasm = Js.Promise.then_ initialize (load_wasm ()) in
  let _ =
    Js.Promise.catch
      (fun error ->
        report (detail error);
        Js.Promise.resolve ())
      (Js.Promise.then_ (fun _ -> Js.Promise.resolve ()) wasm)
  in
  let _ =
    Js.Promise.catch
      (fun error ->
        report (detail error);
        Js.Promise.resolve ())
      (load ())
  in
  ()
