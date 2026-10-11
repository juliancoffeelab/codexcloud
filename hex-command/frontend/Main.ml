type root
external by_id : string -> Dom.element Js.Nullable.t = "getElementById" [@@mel.scope "document"]
external create_root : Dom.element -> root = "createRoot" [@@mel.module "react-dom/client"]
external render : root -> React.element -> unit = "render" [@@mel.send]
external unmount : root -> unit = "unmount" [@@mel.send]
external event : string -> Dom.event = "Event" [@@mel.new]
external notify : Dom.event -> unit = "dispatchEvent" [@@mel.scope "window"]
external listen : string -> (Dom.event -> unit) -> unit = "addEventListener" [@@mel.scope "window"]
external remove : string -> (Dom.event -> unit) -> unit = "removeEventListener" [@@mel.scope "window"]
external trusted : Dom.event -> bool = "isTrusted" [@@mel.get]
let () =
  let target = match Js.Nullable.toOption (by_id "app") with
    | Some target -> target | None -> Js.Exn.raiseError "Missing application mount." in
  ignore (Js.Promise.then_ (fun factory ->
      try
        let session = Session.create factory in
        let root = create_root target in
        let disposed = ref false in
        let gesture event = Session.gesture session (trusted event) in
        let dispose () = if not !disposed then (
            disposed := true;
            remove "pointerdown" gesture; remove "keydown" gesture;
            unmount root; Session.close session; Native.dispose_factory factory) in
        listen "pagehide" (fun _ -> dispose ());
        listen "pointerdown" gesture; listen "keydown" gesture;
        render root (App.make ([%mel.obj {session; onMount=(fun () ->
            Session.initialize session;
            notify (event "hex-command-ready"); dispose)}]));
        Js.Promise.resolve ()
      with exception_ -> Native.dispose_factory factory; raise exception_) (Native.create_factory ()))
