(* This module stays Phaser-free at runtime: native engine/HUD readiness is
   independent of the lazy renderer download and preparation. *)
type module_
type prepared

external prepare : module_ -> Phaser_sound.t -> prepared = "prepare" [@@mel.send]
external dispose : module_ -> prepared -> unit = "dispose" [@@mel.send]

let load : unit -> module_ Js.Promise.t = [%mel.raw {|() => import("./Phaser_owner.js")|}]

type error = Cancelled | Failed of Foreign_error.cause
type loaded = { module_ : module_; prepared : prepared }

type t = {
  mutable closed : bool;
  mutable loaded : loaded option;
  pending : (loaded, error) result Js.Promise.t;
}

let start audio =
  let owner = ref None in
  let pending =
    Js.Promise.catch
      (fun error -> Js.Promise.resolve (Error (Failed (Foreign_error.Rejection error))))
      (Js.Promise.then_
         (fun module_ ->
           let loaded = { module_; prepared = prepare module_ audio } in
           match !owner with
           | Some t when not t.closed ->
               t.loaded <- Some loaded;
               Js.Promise.resolve (Ok loaded)
           | _ ->
               dispose module_ loaded.prepared;
               Js.Promise.resolve (Error Cancelled))
         (load ()))
  in
  let t = { closed = false; loaded = None; pending } in
  owner := Some t;
  t

let close t =
  if not t.closed then (
    t.closed <- true;
    Option.iter (fun loaded -> dispose loaded.module_ loaded.prepared) t.loaded;
    t.loaded <- None)

type native_battlefield
type battlefield = { owner_module : module_; native : native_battlefield }

external mount_scene : module_ -> prepared -> Battlefield_view.mount_options -> native_battlefield
  = "mount_battlefield"
[@@mel.send]

external present_scene : module_ -> native_battlefield -> Battlefield_view.view -> bool -> unit
  = "present_battlefield"
[@@mel.send]

external pulse_scene :
  module_ -> native_battlefield -> string -> string -> Phaser_sound.interaction option -> unit
  = "pulse_battlefield"
[@@mel.send]

external retry_scene : module_ -> native_battlefield -> Phaser_resource_plan.kind option -> unit
  = "retry_battlefield"
[@@mel.send]

external dispose_scene : module_ -> native_battlefield -> unit = "dispose_battlefield" [@@mel.send]

let mount_battlefield t options =
  Js.Promise.then_
    (fun outcome ->
      Js.Promise.resolve
        (if t.closed then Error Cancelled
         else
           match outcome with
           | Error error -> Error error
           | Ok loaded -> (
               try
                 Ok
                   {
                     owner_module = loaded.module_;
                     native = mount_scene loaded.module_ loaded.prepared options;
                   }
               with error -> Error (Failed (Foreign_error.Exception error)))))
    t.pending

let present t view reduced_motion = present_scene t.owner_module t.native view reduced_motion
let pulse t tile kind sound = pulse_scene t.owner_module t.native tile kind sound
let retry_resources t = retry_scene t.owner_module t.native None
let dispose_battlefield t = dispose_scene t.owner_module t.native

external retry_audio_prepared : module_ -> prepared -> unit = "retry_audio" [@@mel.send]

external commander_scene : module_ -> native_battlefield -> unit = "commander_battlefield"
[@@mel.send]

let commander t = commander_scene t.owner_module t.native

let retry_audio t =
  Js.Promise.then_
    (fun outcome ->
      (match outcome with
      | Ok loaded when not t.closed -> retry_audio_prepared loaded.module_ loaded.prepared
      | _ -> ());
      Js.Promise.resolve ())
    t.pending
