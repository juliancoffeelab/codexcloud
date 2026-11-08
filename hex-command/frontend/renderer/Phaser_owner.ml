(* A prepared renderer owns at most one live Game. The scene implementation is
   supplied by the battlefield; preparation itself allocates no GPU resources. *)
type t = {
  mutable closed : bool;
  mutable runtime : Phaser_runtime.t option;
  mutable scene : Phaser_battlefield.battlefield option;
  audio : Phaser_sound.t;
}

let prepare audio = { closed = false; runtime = None; scene = None; audio }

let dispose t =
  if not t.closed then (
    t.closed <- true;
    let runtime = t.runtime in
    t.runtime <- None;
    t.scene <- None;
    Option.iter
      (fun runtime ->
        let _ = Phaser_runtime.dispose runtime in
        ())
      runtime)

let mount t config ~failed =
  if t.closed then invalid_arg "Phaser preparation was disposed";
  if t.runtime <> None then invalid_arg "Phaser preparation already has a game";
  let runtime = Phaser_runtime.create config ~failed in
  t.runtime <- Some runtime;
  Phaser_runtime.on_dispose runtime (fun () ->
      (* A prepared owner survives sequential battlefield mounts. A stale
           native destroy must not release a replacement runtime. *)
      match t.runtime with
      | Some current when current == runtime ->
          t.runtime <- None;
          t.scene <- None
      | _ -> ());
  runtime

let mount_attached t config failed attached =
  let runtime = mount t config ~failed in
  (* Attach application ownership before the deferred native boot can call create. *)
  (try attached runtime with error -> Phaser_runtime.fail runtime error);
  runtime

open Phaser_audio

type battlefield = { runtime : Phaser_runtime.t; scene : Phaser_battlefield.battlefield }

let mount_battlefield t (options : Battlefield_view.mount_options) =
  let scene =
    new Phaser_battlefield.battlefield
      ~audio:t.audio ~actions:options.actions ~describe:options.describe ~previews:options.previews
      ~progress:options.progress
  in
  let config =
    Phaser.Game.config ~parent:options.parent ~width:1 ~height:1 ~scene:[| scene#native |]
  in
  let runtime =
    mount_attached t config options.failed (fun runtime ->
        scene#attach runtime;
        let game = Phaser_runtime.game runtime in
        let permission () =
          let manager = game##sound in
          match Js.Nullable.toOption manager##context with
          | None -> Js.Promise.resolve (not manager##locked)
          | Some context ->
              if context_state context = "closed" then (
                manager##removeAll ();
                let replacement = create_context manager game in
                clear_context manager Js.Nullable.null;
                set_context manager replacement);
              let context = Option.get (Js.Nullable.toOption manager##context) in
              let resumed =
                if context_state context = "running" then Js.Promise.resolve ()
                else resume_context context
              in
              Js.Promise.then_
                (fun () ->
                  let ready =
                    (not (Phaser_runtime.unavailable runtime))
                    && Option.fold ~none:false
                         ~some:(fun context -> context_state context = "running")
                         (Js.Nullable.toOption manager##context)
                  in
                  if ready then (
                    unlocked manager true;
                    locked manager false);
                  Js.Promise.resolve ready)
                resumed
        in
        let detach = Phaser_sound.attach t.audio game permission in
        Phaser_runtime.on_dispose runtime detach)
  in
  if not (Phaser_runtime.unavailable runtime) then t.scene <- Some scene;
  { runtime; scene }

let present_battlefield t view reduced_motion = t.scene#present view reduced_motion
let pulse_battlefield t tile kind sound = t.scene#pulse ?sound tile kind
let retry_battlefield t kind = t.scene#retry_resources kind

let dispose_battlefield t =
  let _ = Phaser_runtime.dispose t.runtime in
  ()

let commander_battlefield t = t.scene#return_to_commander

let retry_audio (t : t) =
  match (t.runtime, t.scene) with
  | Some runtime, Some scene when not (Phaser_runtime.unavailable runtime) ->
      scene#retry_resources (Some Phaser_resource_plan.Audio)
  | _ ->
      Phaser_sound.report_readiness t.audio false
        (Some
           "Battlefield audio is unavailable because its renderer stopped. Reload the game to \
            restore audio.")
