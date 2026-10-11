(** Imported only after the catalog-free shell has committed and painted. *)
type root
external render : root -> React.element -> unit = "render" [@@mel.send]
external listen : Dom.element -> string -> (Dom.event -> unit) -> bool -> unit = "addEventListener" [@@mel.send]
external remove : Dom.element -> string -> (Dom.event -> unit) -> bool -> unit = "removeEventListener" [@@mel.send]
external trusted : Dom.event -> bool = "isTrusted" [@@mel.get]
external event : string -> Dom.event = "Event" [@@mel.new]
external notify : Dom.event -> unit = "dispatchEvent" [@@mel.scope "window"]
external target : Dom.event -> Dom.element = "target" [@@mel.get]
external tag_name : Dom.element -> string = "tagName" [@@mel.get]
external closest : Dom.element -> string -> Dom.element Js.Nullable.t = "closest" [@@mel.send]
let release_both first second =
  let failure = ref None in
  (try first () with error -> failure := Some error);
  (try second () with error -> if !failure = None then failure := Some error);
  Option.iter (fun error -> Fault.raise_original error) !failure
let mount root host current =
  Js.Promise.then_ (fun factory ->
      if not (current ()) then (Native.dispose_factory factory; Js.Promise.resolve (fun () -> ()))
      else try
          let session = Session.create factory in
          let disposed = ref false in
          let gesture event = Session.gesture session (trusted event) in
          let failed event =
            if not !disposed && current () then (
              let image = target event in
              let tag = tag_name image in
              if (tag="IMG" || tag="image" || tag="IMAGE") &&
                 Js.Nullable.isNullable (closest image ".map-terrain") then
                Session.message session "Artwork could not be loaded. Reload the game to retry.") in
          let close () = if not !disposed then (
              disposed := true;
              remove host "pointerdown" gesture true; remove host "keydown" gesture true;
              remove host "error" failed true;
              release_both (fun () -> Session.close session) (fun () -> Native.dispose_factory factory)) in
          try
            listen host "pointerdown" gesture true; listen host "keydown" gesture true;
            listen host "error" failed true;
            render root (React.createElement App.make ([%mel.obj {session; onMount=(fun () ->
                if current () then (Session.initialize session; notify (event "hex-command-ready")); close)}]));
            Js.Promise.resolve close
          with error -> (try close () with _ -> ()); Fault.raise_original error
        with error -> (try Native.dispose_factory factory with _ -> ()); Fault.raise_original error) (Native.create_factory ())
