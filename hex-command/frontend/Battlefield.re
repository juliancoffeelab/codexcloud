open Ui;

[@react.component]
let make =
    (
      ~view: Session.model,
      ~actions: Session.actions,
      ~renderer: Phaser_boot.t,
      ~previews: Terrain_preview.t,
      ~progress: option(Battlefield_view.progress),
      ~onProgress,
      ~onRendererFailed,
      ~prepareReload,
      ~showCommanderArrow=false,
    ) => {
  let host = React.useRef(Js.Nullable.null);
  let runtime = React.useRef(None);
  let latest = React.useRef(view);
  latest.current = view;
  let commands = React.useRef(actions);
  commands.current = actions;
  let (description, setDescription) = React.useState(() => "Battlefield loading");
  let (failure, setFailure) = React.useState(() => None);
  let (reducedMotion, setReducedMotion) = React.useState(Motion.reduced_motion);
  let reducedLatest = React.useRef(reducedMotion);
  reducedLatest.current = reducedMotion;
  let (hint, setHint) = React.useState(() => None);
  let (busy, setBusy) = React.useState(() => false);
  let (reloadError, setReloadError) = React.useState(() => None);
  let (copied, setCopied) = React.useState(() => false);
  let failedOnce = React.useRef(false);
  let fail = message =>
    if (!failedOnce.current) {
      failedOnce.current = true;
      onRendererFailed(message);
      setFailure(_ => Some(message));
      Option.iter(Phaser_boot.dispose_battlefield, runtime.current);
      runtime.current = None;
    };
  let reloadFrom = source => {
    setBusy(_ => true);
    setReloadError(_ => None);
    let _ =
      prepareReload(source)
      |> Js.Promise.then_(() => {
           reload();
           Js.Promise.resolve();
         })
      |> Js.Promise.catch(cause => {
           setReloadError(_ => Some(Foreign_error.describe(cause)));
           setBusy(_ => false);
           Js.Promise.resolve();
         });
    ();
  };
  let copyDiagnostics = () => {
    let message =
      String.concat("\n\n", List.filter_map(value => value, [failure, reloadError]));
    let _ =
      Browser_dom.copy_text(Utf8.to_js(message))
      |> Js.Promise.then_(() => {
           setCopied(_ => true);
           Js.Promise.resolve();
         })
      |> Js.Promise.catch(cause => {
           setReloadError(_ => Some(Foreign_error.describe(cause)));
           Js.Promise.resolve();
         });
    ();
  };
  React.useEffect0(() => Some(Motion.subscribe_reduced(value => setReducedMotion(_ => value))));
  let present = active => {
    let view = latest.current;
    let facts: Battlefield_view.view = {
      game: view.game,
      catalog: view.catalog,
      artwork: view.artwork,
      selected: view.map_focus.selected,
      target: view.map_focus.target,
      inspected_tile: view.map_focus.inspected,
      recruitment:
        Option.fold(
          ~none=false,
          ~some=((commander, _)) => commander != None,
          Option.bind(
            Option.bind(view.map_focus.inspected, Native.tile(view.interaction_game)),
            Presentation.recruitment_context(
              view.catalog,
              view.interaction_game,
              view.map_focus.selected,
            ),
          ),
        ),
      session_version: view.session_version,
      sound: view.sound,
      action: view.action,
      combat_playback: view.combat_playback,
    };
    Phaser_boot.present(active, facts, reducedLatest.current);
  };
  React.useEffect0(() => {
    let disposed = ref(false);
    let options: Battlefield_view.mount_options = {
      parent: Option.get(Js.Nullable.toOption(host.current)),
      actions: () => {
        Battlefield_view.choose: tile => commands.current.choose(tile),
        clear_selection: reset => commands.current.clear(reset),
      },
      describe: label =>
        if (! disposed^) {
          setDescription(_ => label);
        },
      previews,
      progress: value =>
        if (! disposed^) {
          onProgress(value);
        },
      failed: message =>
        if (! disposed^) {
          fail(message);
        },
    };
    let _ =
      Phaser_boot.mount_battlefield(renderer, options)
      |> Js.Promise.then_(outcome => {
           switch (outcome) {
           | Ok(active) =>
             if (disposed^ || failedOnce.current) {
               Phaser_boot.dispose_battlefield(active);
             } else {
               runtime.current = Some(active);
               present(active);
             }
           | Error(Phaser_boot.Cancelled) => ()
           | Error(Phaser_boot.Failed(cause)) =>
             if (! disposed^) {
               fail(Foreign_error.cause_detail(cause));
             }
           };
           Js.Promise.resolve();
         });
    let motion = event => {
      let detail = Motion.detail(event);
      Option.iter(
        active => Phaser_boot.pulse(active, detail.tile, detail.kind, detail.sound),
        runtime.current,
      );
    };
    let map = Option.get(Js.Nullable.toOption(byId("map")));
    Motion.listen(map, motion);
    let node = Option.get(Js.Nullable.toOption(host.current));
    let commander = event =>
      setHint(_ => Js.Nullable.toOption(Battlefield_view.Commander.detail(event)));
    Battlefield_view.Commander.on(node, "battlefieldcommander", commander);
    Some(
      () => {
        disposed := true;
        Motion.unlisten(map, motion);
        Battlefield_view.Commander.off(node, "battlefieldcommander", commander);
        Option.iter(Phaser_boot.dispose_battlefield, runtime.current);
        runtime.current = None;
      },
    );
  });
  React.useLayoutEffect2(
    () => {
      try(Option.iter(present, runtime.current)) {
      | error => fail(Foreign_error.cause_detail(Foreign_error.Exception(error)))
      };
      None;
    },
    (view, reducedMotion),
  );
  let ready =
    failure != None
    || Option.fold(~none=false, ~some=value => value.Battlefield_view.phase == "ready", progress);
  let loaded = Option.fold(~none=0, ~some=value => value.Battlefield_view.loaded, progress);
  let total = Option.fold(~none=0, ~some=value => value.Battlefield_view.total, progress);
  let phase = Option.fold(~none="loading", ~some=value => value.Battlefield_view.phase, progress);
  <>
    <Battlefield_window
      host
      phase={
        ready
          ? Battlefield_window.Ready
          : phase == "preparing" ? Battlefield_window.Preparing : Battlefield_window.Loading
      }
      loaded
      total
      navigation={
        showCommanderArrow && ready && failure == None
          ? option(hint, angle =>
              <button
                className="commander-return"
                ariaLabel="Return to commander"
                title="Return to commander"
                onPointerDown={Button_press.pointer_down(() =>
                  Option.iter(Phaser_boot.commander, runtime.current)
                )}
                onClick={Button_press.click(() =>
                  Option.iter(Phaser_boot.commander, runtime.current)
                )}>
                <svg
                  viewBox="0 0 24 24"
                  ariaHidden=true
                  style={ReactDOM.Style.make(
                    ~transform="rotate(" ++ string_of_int(angle) ++ "deg)",
                    (),
                  )}>
                  <path d="M12 20V4M5 11l7-7 7 7" />
                </svg>
              </button>
            )
          : React.null
      }>
      <>
        {option(Option.bind(progress, value => value.Battlefield_view.error), message =>
           <div className="prefetch-error" role="alert">
             <p>
               {text(
                  "Some artwork could not be prepared. The current battlefield remains available.",
                )}
             </p>
             <details>
               <summary> {text("Error details")} </summary>
               <pre> {text(message)} </pre>
             </details>
             <button
               onClick={_ =>
                 Option.iter(active => Phaser_boot.retry_resources(active), runtime.current)
               }>
               {text("Retry artwork")}
             </button>
           </div>
         )}
        {option(failure, message =>
           <div className="map-render-error" role="alert">
             <p> {text("The battlefield stopped. Reload to restore saved progress.")} </p>
             <p>
               {text(
                  "Latest checkpoint keeps saved actions. Restarting the turn discards actions since its start. An interrupted AI turn replays from its saved boundary.",
                )}
             </p>
             <button disabled=busy onClick={_ => reloadFrom("checkpoint")}>
               {text("Reload latest checkpoint")}
             </button>
             <button disabled=busy onClick={_ => reloadFrom("turn-start")}>
               {text("Restart current turn")}
             </button>
             <details>
               <summary> {text("Error details")} </summary>
               <pre> {text(message)} </pre>
             </details>
             <button onClick={_ => copyDiagnostics()}>
               {text(copied ? "Diagnostics copied" : "Copy diagnostics")}
             </button>
             {option(reloadError, message => <pre role="alert"> {text(message)} </pre>)}
           </div>
         )}
      </>
    </Battlefield_window>
    <p id="battlefield-controls" className="status">
      {text(
         "Drag to pan. Pinch or Control or Command plus wheel to zoom. Keyboard: plus and minus to zoom, zero to reset zoom. Escape clears selection.",
       )}
    </p>
    <p id="battlefield-description" className="status" ariaLive="polite"> {text(description)} </p>
  </>;
};
