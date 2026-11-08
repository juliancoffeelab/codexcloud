type root
type application

external create_root : Dom.element -> root = "createRoot" [@@mel.module "preact/compat/client"]
external render : root -> React.element -> unit = "render" [@@mel.send]
external unmount : root -> unit = "unmount" [@@mel.send]
external flush : (unit -> unit) -> unit = "flushSync" [@@mel.module "preact/compat"]

open Browser_dom
open Browser_time

external application_mount :
  application ->
  root ->
  Dom.element ->
  (unit -> bool) ->
  (Foreign_error.t -> unit) ->
  (unit -> unit, string) result Js.Promise.t = "mount"
[@@mel.send]

let failure_reason = Foreign_error.describe
let load : unit -> application Js.Promise.t = [%mel.raw {|() => import("./Application.js")|}]

let after_paint work =
  let timer = ref None in
  let frame = frame (fun () -> timer := Some (timeout_float work 0.)) in
  fun () ->
    cancel_frame frame;
    Option.iter clear_timeout !timer

let () =
  let target =
    match Js.Nullable.toOption (byId "app") with
    | Some target -> target
    | None -> Js.Exn.raiseError "Missing application mount."
  in
  let lifetime = ref None and closed = ref false in
  let failed error =
    microtask (fun () ->
        if not !closed then
          Option.iter (fun owner -> Startup.fail owner (failure_reason error)) !lifetime)
  in
  let root = create_root target in
  let uncaught event = Option.iter failed (Js.Undefined.toOption (event_error event)) in
  let rejected event = failed (event_reason event) in
  listen_error "error" uncaught;
  listen_error "unhandledrejection" rejected;
  let show = function
    | Startup.Ready -> ()
    | phase ->
        let failure = match phase with Startup.Failed message -> Some message | _ -> None in
        flush (fun () -> render root (React.createElement Shell.make [%mel.obj { failure }]));
        notify (event "hex-command-shell-ready")
  in
  let startup =
    Startup.create ~show ~schedule:after_paint ~load:(fun current ->
        Js.Promise.then_
          (fun application ->
            if current () then application_mount application root target current failed
            else Js.Promise.resolve (Ok (fun () -> ())))
          (load ()))
  in
  lifetime := Some startup;
  listen_pagehide (fun event ->
      (* A cached page resumes this owner on Back; only a final departure disposes it. *)
      if (not (page_persisted event)) && not !closed then (
        closed := true;
        remove_error "error" uncaught;
        remove_error "unhandledrejection" rejected;
        Startup.close startup;
        unmount root))
