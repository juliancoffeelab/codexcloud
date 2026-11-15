type root
(** Imported only after the catalog-free shell has committed and painted. *)

external render : root -> React.element -> unit = "render" [@@mel.send]

open Browser_dom

let release_both first second =
  let failure = ref None in
  (try first () with error -> failure := Some error);
  (try second () with error -> if !failure = None then failure := Some error);
  Option.iter (fun error -> Fault.raise_original error) !failure

let mount root host current failed =
  Js.Promise.then_
    (function
      | Error error -> Js.Promise.resolve (Error (Native.message error))
      | Ok factory ->
          if not (current ()) then (
            Native.dispose_factory factory;
            Js.Promise.resolve (Ok (fun () -> ())))
          else
            let () = Loading_progress.advance 2 in
            let audio = Phaser_sound.create () in
            let renderer = Phaser_boot.start audio in
            Js.Promise.catch
              (fun error ->
                Phaser_boot.close renderer;
                Native.dispose_factory factory;
                Save_json.reject error)
              (Js.Promise.then_
                 (fun session ->
                   if not (current ()) then (
                     Phaser_boot.close renderer;
                     Session.close session;
                     Native.dispose_factory factory;
                     Js.Promise.resolve (Ok (fun () -> ())))
                   else
                     try
                       let () = Loading_progress.advance 3 in
                       let disposed = ref false in
                       let _ = Session.prepare_audio session in
                       let tools_registered = ref false and unregister_tools = ref (fun () -> ()) in
                       let gesture event = Session.gesture session (trusted event) in
                       let artwork_failed event =
                         if (not !disposed) && current () then
                           let image = target event in
                           let tag = tag_name image in
                           if
                             (tag = {js|IMG|js} || tag = {js|image|js} || tag = {js|IMAGE|js})
                             && Js.Nullable.isNullable (closest image {js|.map-terrain|js})
                           then
                             Session.message session
                               {js|Artwork could not be loaded. Reload the game to retry.|js}
                       in
                       let close () =
                         if not !disposed then (
                           disposed := true;
                           remove_capture host {js|pointerdown|js} gesture true;
                           remove_capture host {js|keydown|js} gesture true;
                           remove_capture host {js|error|js} artwork_failed true;
                           release_both
                             (fun () -> Phaser_boot.close renderer)
                             (fun () ->
                               release_both !unregister_tools (fun () ->
                                   release_both
                                     (fun () -> Session.close session)
                                     (fun () -> Native.dispose_factory factory))))
                       in
                       try
                         listen_capture host {js|pointerdown|js} gesture true;
                         listen_capture host {js|keydown|js} gesture true;
                         listen_capture host {js|error|js} artwork_failed true;
                         let application () =
                           React.createElement App.make
                             [%mel.obj
                               {
                                 session;
                                 renderer;
                                 onMount =
                                   (fun () ->
                                     if (not !disposed) && current () then (
                                       Session.initialize session;
                                       if not !tools_registered then (
                                         tools_registered := true;
                                         unregister_tools := Tools.register session);
                                       notify (event {js|hex-command-ready|js}));
                                     close);
                               }]
                         in
                         render root
                           (React.createElement Error_boundary.make
                              [%mel.obj { onError = failed; render = application }]);
                         Js.Promise.resolve (Ok close)
                       with error ->
                         (try close () with _ -> ());
                         Fault.raise_original error
                     with error ->
                       (try Session.close session with _ -> ());
                       Fault.raise_original error)
                 (Session.create_with_audio audio factory)))
    (Native.create_factory ())
