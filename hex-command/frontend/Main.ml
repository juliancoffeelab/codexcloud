type root
type application
external by_id : string -> Dom.element Js.Nullable.t = "getElementById" [@@mel.scope "document"]
external create_root : Dom.element -> <onUncaughtError : (Js.Json.t -> Js.Json.t -> unit)> Js.t -> root = "createRoot" [@@mel.module "react-dom/client"]
external render : root -> React.element -> unit = "render" [@@mel.send]
external unmount : root -> unit = "unmount" [@@mel.send]
external flush : (unit -> unit) -> unit = "flushSync" [@@mel.module "react-dom"]
external event : string -> Dom.event = "Event" [@@mel.new]
external notify : Dom.event -> unit = "dispatchEvent" [@@mel.scope "window"]
external listen : string -> (unit -> unit) -> unit = "addEventListener" [@@mel.scope "window"]
external frame : (unit -> unit) -> int = "requestAnimationFrame"
external cancel_frame : int -> unit = "cancelAnimationFrame"
external timeout : (unit -> unit) -> float -> int = "setTimeout"
external cancel_timeout : int -> unit = "clearTimeout"
external application_mount : application -> root -> Dom.element -> (unit -> bool) -> (unit -> unit) Js.Promise.t = "mount" [@@mel.send]
external microtask : (unit -> unit) -> unit = "queueMicrotask"
external stringify : Js.Json.t -> string = "String"
let failure_reason error =
  Option.bind (Js.Json.decodeObject error) (fun object_ ->
      Option.bind (Js.Dict.get object_ "message") Js.Json.decodeString)
  |> Option.value ~default:(stringify error)
let load : unit -> application Js.Promise.t = [%mel.raw {|() => import("./Application.js")|}]
let after_paint work =
  let timer = ref None in
  let frame = frame (fun () -> timer := Some (timeout work 0.)) in
  fun () -> cancel_frame frame; Option.iter cancel_timeout !timer
let () =
  let target = match Js.Nullable.toOption (by_id "app") with
    | Some target -> target | None -> Js.Exn.raiseError "Missing application mount." in
  let lifetime = ref None and closed = ref false in
  let failed error _ = microtask (fun () -> if not !closed then
                                     Option.iter (fun owner -> Startup.fail owner (failure_reason error)) !lifetime) in
  let root = create_root target ([%mel.obj {onUncaughtError=failed}]) in
  let show = function
    | Startup.Ready -> ()
    | phase ->
      let failure = match phase with Startup.Failed message -> Some message | _ -> None in
      flush (fun () -> render root (React.createElement Shell.make ([%mel.obj {failure}])));
      notify (event "hex-command-shell-ready") in
  let startup = Startup.create ~show ~schedule:after_paint ~load:(fun current ->
      Js.Promise.then_ (fun application ->
          if current () then application_mount application root target current
          else Js.Promise.resolve (fun () -> ())) (load ())) in
  lifetime := Some startup;
  listen "pagehide" (fun () -> if not !closed then (
      closed := true; Startup.close startup; unmount root))
