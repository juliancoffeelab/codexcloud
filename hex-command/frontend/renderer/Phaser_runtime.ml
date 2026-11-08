open Browser_time

(* The 3.90 Game seam mirrors upstream phaser-runtime.ts. Application policy is
   OCaml; Phaser.Class supplies only JavaScript prototype inheritance. *)
type t = {
  game : Phaser.Game.t;
  notify : string -> unit;
  mutable render_dirty : bool;
  mutable disposed : bool;
  mutable failed : bool;
  mutable cleanup_errors : exn list;
  mutable disposal_callbacks : (unit -> unit) list;
}

let prototype = Phaser.Game.prototype Phaser.Game.constructor
let unavailable t = t.disposed || t.failed
let game t = t.game
let invalidate t = t.render_dirty <- true
let record_cleanup_error t error = t.cleanup_errors <- error :: t.cleanup_errors

let release_callbacks t =
  let callbacks = t.disposal_callbacks in
  t.disposal_callbacks <- [];
  List.iter (fun release -> try release () with error -> record_cleanup_error t error) callbacks

let on_dispose t release =
  if t.disposed then release () else t.disposal_callbacks <- release :: t.disposal_callbacks

let dispose t =
  if t.disposed then [||]
  else (
    t.disposed <- true;
    (* Application cleanup cannot depend on native runDestroy reaching its
       destroy event: an earlier scene shutdown may throw. *)
    release_callbacks t;
    let game = t.game in
    game##loop##stop ();
    prototype##destroy##call game true;
    let release f = try f () with error -> record_cleanup_error t error in
    let drop value set =
      Option.iter
        (fun device ->
          if Js.Nullable.isNullable device##target then set game##input Js.Nullable.null)
        (Js.Nullable.toOption value)
    in
    drop game##input##keyboard Phaser.Internal.drop_keyboard;
    drop game##input##mouse Phaser.Internal.drop_mouse;
    drop game##input##touch Phaser.Internal.drop_touch;
    let finally () =
      let _ = game##events##removeAllListeners () in
      if not (Js.Nullable.isNullable game##loop##raf) then
        release (fun () -> game##loop##destroy ());
      Option.iter
        (fun canvas ->
          release (fun () -> Phaser.Internal.remove_canvas canvas);
          release (fun () -> canvas##remove ()))
        (Js.Nullable.toOption game##canvas);
      Option.iter
        (fun container -> release (fun () -> container##remove ()))
        (Js.Nullable.toOption game##domContainer)
    in
    let work () =
      try prototype##runDestroy##call game
      with _ ->
        if not (Js.Nullable.isNullable game##scene##systemScene) then
          release (fun () -> game##scene##destroy ());
        let destroy_manager (manager : Phaser.manager) =
          if not (Js.Nullable.isNullable manager##game) then release (fun () -> manager##destroy ())
        in
        (* Input and scale have extra operations, so retain their precise types. *)
        if not (Js.Nullable.isNullable game##input##game) then
          release (fun () -> game##input##destroy ());
        if not (Js.Nullable.isNullable game##sound##game) then
          release (fun () -> game##sound##destroy ());
        if not (Js.Nullable.isNullable game##scale##game) then
          release (fun () -> game##scale##destroy ());
        destroy_manager game##anims;
        destroy_manager game##plugins;
        let textures = game##textures in
        if not (Js.Nullable.isNullable textures##game) then
          if not (Js.Nullable.isNullable textures##stamp) then
            release (fun () -> textures##destroy ())
          else (
            Array.iter
              (fun key ->
                release (fun () ->
                    let _ = textures##remove key in
                    ()))
              (textures##getTextureKeys ());
            let _ = textures##removeAllListeners () in
            Option.iter Phaser.Internal.remove_canvas (Js.Nullable.toOption textures##_tempCanvas));
        Option.iter
          (fun renderer -> release (fun () -> renderer##destroy ()))
          (Js.Nullable.toOption game##renderer)
    in
    (match work () with
    | () -> finally ()
    | exception error ->
        finally ();
        raise error);
    Array.of_list (List.rev t.cleanup_errors))

let fail t error =
  if not (unavailable t) then (
    t.failed <- true;
    t.game##loop##stop ();
    microtask (fun () ->
        let errors = try dispose t with error -> [| error |] in
        t.notify
          (String.concat "\n"
             (Foreign_error.exception_ error
             :: Array.to_list
                  (Array.map (fun error -> "Cleanup: " ^ Foreign_error.exception_ error) errors)))))

let guard t f = if not (unavailable t) then try f () with error -> fail t error

let step t time delta =
  guard t (fun () ->
      let game = t.game in
      if game##pendingDestroy then prototype##runDestroy##call game
      else if not game##isPaused then (
        let events = game##events in
        Phaser.Events.emit_step events "prestep" time delta;
        Phaser.Events.emit_step events "step" time delta;
        game##scene##update time delta;
        Phaser.Events.emit_step events "poststep" time delta;
        if not t.render_dirty then Phaser.Internal.processing game##scene false
        else (
          t.render_dirty <- false;
          let renderer =
            match Js.Nullable.toOption game##renderer with
            | Some renderer -> renderer
            | None -> invalid_arg "Phaser renderer is unavailable"
          in
          renderer##preRender ();
          Phaser.Events.emit_render events "prerender" renderer time delta;
          game##scene##render renderer;
          renderer##postRender ();
          Phaser.Events.emit_render events "postrender" renderer time delta)))

let create config ~failed =
  let owner = ref None in
  let with_owner f = Option.iter f !owner in
  let definition =
    [%mel.obj
      {
        boot =
          (fun[@mel.this] (_game : Phaser.game) ->
            microtask (fun () ->
                with_owner (fun t -> guard t (fun () -> prototype##boot##call t.game))));
        texturesReady =
          (fun[@mel.this] (_game : Phaser.game) ->
            with_owner (fun t -> guard t (fun () -> prototype##texturesReady##call t.game)));
        start =
          (fun[@mel.this] (_game : Phaser.game) ->
            with_owner (fun t -> guard t (fun () -> prototype##start##call t.game)));
        step =
          (fun[@mel.this] (_game : Phaser.game) time delta ->
            with_owner (fun t -> step t time delta));
        destroy =
          (fun[@mel.this] (_game : Phaser.game) (_canvas : bool) (_return : bool) ->
            with_owner (fun t ->
                let _ = dispose t in
                ()));
      }]
  in
  Phaser.Class.extends_game definition Phaser.Game.constructor;
  let game = Phaser.construct (Phaser.Class.game definition) [| config |] in
  let t =
    {
      game;
      notify = failed;
      render_dirty = true;
      disposed = false;
      failed = false;
      cleanup_errors = [];
      disposal_callbacks = [];
    }
  in
  owner := Some t;
  let _ = game##events##once "destroy" (fun[@u] () -> release_callbacks t) in
  let dirty = fun[@u] () -> invalidate t in
  List.iter
    (fun event ->
      let _ = game##events##on event dirty in
      ())
    [ "visible"; "focus"; "resume" ];
  let _ = game##scale##on "resize" dirty in
  t
