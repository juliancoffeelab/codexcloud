(** HTML fallback owns failures before the React shell mounts. It imports no game
    data, React, or WASM; Main loads after these handlers are installed. *)
type element
type event
external by_id : string -> element Js.Nullable.t = "getElementById" [@@mel.scope "document"]
external text : element -> string -> unit = "textContent" [@@mel.set]
external hidden : element -> bool -> unit = "hidden" [@@mel.set]
external open_ : element -> bool -> unit = "open" [@@mel.set]
external remove_element : element -> unit = "remove" [@@mel.send]
external listen : string -> (event -> unit) -> bool -> unit = "addEventListener" [@@mel.scope "window"]
external remove : string -> (event -> unit) -> bool -> unit = "removeEventListener" [@@mel.scope "window"]
external event_error : event -> Js.Json.t Js.Undefined.t = "error" [@@mel.get]
external event_reason : event -> Js.Json.t = "reason" [@@mel.get]
external event_message : event -> string Js.Undefined.t = "message" [@@mel.get]
external error_stack : Js.Json.t -> string Js.Undefined.t = "stack" [@@mel.get]
external error_message : Js.Json.t -> string Js.Undefined.t = "message" [@@mel.get]
external rejection_json : Js.Promise.error -> Js.Json.t = "%identity"
external stringify : Js.Json.t -> string = "String"
let field id action = Js.Nullable.toOption (by_id id) |> Option.iter action
let detail error = match Js.Json.decodeObject error with
  | None -> stringify error
  | Some _ -> Option.value ~default:(Option.value ~default:(stringify error) (Js.Undefined.toOption (error_message error)))
                (Js.Undefined.toOption (error_stack error))
let report reason =
  field "startup-message" (fun node -> text node "The game could not start. Reloading keeps your saved battle.");
  field "startup-error" (fun node -> hidden node false; open_ node true);
  field "startup-detail" (fun node -> text node reason)
let load : unit -> unit Js.Promise.t = [%mel.raw {|() => import("./Main.js")|}]
let () =
  let failed event = match Js.Undefined.toOption (event_error event) with
    | Some error -> report (detail error)
    | None -> Option.iter report (Js.Undefined.toOption (event_message event)) in
  let rejected event = report (detail (event_reason event)) in
  let rec ready _ =
    field "startup" remove_element;
    remove "error" failed true; remove "unhandledrejection" rejected false;
    remove "hex-command-shell-ready" ready false in
  listen "error" failed true; listen "unhandledrejection" rejected false;
  listen "hex-command-shell-ready" ready false;
  ignore (Js.Promise.catch (fun error -> report (detail (rejection_json error)); Js.Promise.resolve ()) (load ()))
