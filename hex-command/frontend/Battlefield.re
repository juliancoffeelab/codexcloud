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
      ~voicePreparation=None,
      ~onRetryVoice=() => (),
      ~onSkipVoice=() => (),
    ) => {
  let host = React.useRef(Js.Nullable.null);
  let runtime = React.useRef(None);
  let latest = React.useRef(view);
  latest.current = view;
  let commands = React.useRef(actions);
  commands.current = actions;
  let (description, setDescription) = React.useState(() => {js|Battlefield loading|js});
  let (failure, setFailure) = React.useState(() => None);
  let (reducedMotion, setReducedMotion) = React.useState(Motion.reduced_motion);
  let reducedLatest = React.useRef(reducedMotion);
  reducedLatest.current = reducedMotion;
  let (hint, setHint) = React.useState(() => None);
  let (busy, setBusy) = React.useState(() => false);
  let (reloadError, setReloadError) = React.useState(() => None);
  let (copied, setCopied) = React.useState(() => false);
  let failedOnce = React.useRef(false);
  let loadingIdentity = React.useRef(None);
  let fail = message =>
    if (!failedOnce.current) {
      failedOnce.current = true;
      Boot_process.fail(message);
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
      Js.Array.join(
        ~sep={js|\n\n|js},
        Array.of_list(List.filter_map(value => value, [failure, reloadError])),
      );
    let _ =
      Browser_dom.copy_text(message)
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
      follow_ai: view.follow_ai,
      ai_focus: view.ai_focus,
      action: view.action,
      combat_playback: view.combat_playback,
    };
    Phaser_boot.present(active, facts, reducedLatest.current);
  };
  React.useEffect0(() => {
    Loading_progress.begin_battlefield();
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
          let expected = {
            let js_text_part_0 = string_of_int(latest.current.session_version);
            let js_text_part_2 = Native.state(latest.current.game).map_id;
            {j|$(js_text_part_0):$(js_text_part_2)|j};
          };
          if (value.Battlefield_view.identity == expected) {
            if (loadingIdentity.current != Some(expected)) {
              loadingIdentity.current = Some(expected);
              Loading_progress.begin_battlefield();
            };
            switch (value.phase) {
            | matched_text_0 when matched_text_0 == {js|ready|js} => Loading_progress.finish()
            | matched_text_1 when matched_text_1 == {js|preparing|js} =>
              Loading_progress.advance(5)
            | _ when value.total > 0 =>
              let detail = {
                let js_text_part_0 = string_of_int(value.loaded);
                let js_text_part_2 = string_of_int(value.total);
                {j|$(js_text_part_0) / $(js_text_part_2) resources ready|j};
              };
              Loading_progress.advance(
                ~fraction=float_of_int(value.loaded) /. float_of_int(value.total),
                ~detail,
                4,
              );
            | _ => ()
            };
          };
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
    let map = Option.get(Js.Nullable.toOption(byId({js|map|js})));
    Motion.listen(map, motion);
    let node = Option.get(Js.Nullable.toOption(host.current));
    let commander = event =>
      setHint(_ => Js.Nullable.toOption(Battlefield_view.Commander.detail(event)));
    Battlefield_view.Commander.on(node, {js|battlefieldcommander|js}, commander);
    Some(
      () => {
        disposed := true;
        Motion.unlisten(map, motion);
        Battlefield_view.Commander.off(node, {js|battlefieldcommander|js}, commander);
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
    || Option.fold(
         ~none=false,
         ~some=value => value.Battlefield_view.phase == {js|ready|js},
         progress,
       );
  let phase =
    Option.fold(~none={js|loading|js}, ~some=value => value.Battlefield_view.phase, progress);
  <>
    <Battlefield_window
      host
      preparation={ready && failure == None ? voicePreparation : None}
      onRetryPreparation=onRetryVoice
      onSkipPreparation=onSkipVoice
      phase={
        ready
          ? Battlefield_window.Ready
          : phase == {js|preparing|js} ? Battlefield_window.Preparing : Battlefield_window.Loading
      }
      navigation={
        showCommanderArrow && ready && failure == None
          ? option(hint, angle =>
              <button
                className={js|commander-return|js}
                ariaLabel={js|Return to commander|js}
                title={js|Return to commander|js}
                onClick={_ => Option.iter(Phaser_boot.commander, runtime.current)}>
                <svg
                  viewBox={js|0 0 24 24|js}
                  ariaHidden=true
                  style={ReactDOM.Style.make(
                    ~transform=
                      {let js_text_part_1 = string_of_int(angle);
                       {j|rotate($(js_text_part_1)deg)|j}},
                    (),
                  )}>
                  <path d={js|M12 20V4M5 11l7-7 7 7|js} />
                </svg>
              </button>
            )
          : React.null
      }>
      <>
        {option(Option.bind(progress, value => value.Battlefield_view.error), message =>
           <div className={js|prefetch-error|js} role={js|alert|js}>
             <p>
               {text(
                  {js|Some artwork could not be prepared. The current battlefield remains available.|js},
                )}
             </p>
             <details>
               <summary> {text({js|Error details|js})} </summary>
               <pre> {text(message)} </pre>
             </details>
             <button
               onClick={_ =>
                 Option.iter(active => Phaser_boot.retry_resources(active), runtime.current)
               }>
               {text({js|Retry artwork|js})}
             </button>
           </div>
         )}
        {option(failure, message =>
           <div className={js|map-render-error|js} role={js|alert|js}>
             <p> {text({js|The battlefield stopped. Reload to restore saved progress.|js})} </p>
             <p>
               {text(
                  {js|Latest checkpoint keeps saved actions. Restarting the turn discards actions since its start. An interrupted AI turn replays from its saved boundary.|js},
                )}
             </p>
             <button disabled=busy onClick={_ => reloadFrom({js|checkpoint|js})}>
               {text({js|Reload latest checkpoint|js})}
             </button>
             <button disabled=busy onClick={_ => reloadFrom({js|turn-start|js})}>
               {text({js|Restart current turn|js})}
             </button>
             <details>
               <summary> {text({js|Error details|js})} </summary>
               <pre> {text(message)} </pre>
             </details>
             <button onClick={_ => copyDiagnostics()}>
               {text(copied ? {js|Diagnostics copied|js} : {js|Copy diagnostics|js})}
             </button>
             {option(reloadError, message => <pre role={js|alert|js}> {text(message)} </pre>)}
           </div>
         )}
      </>
    </Battlefield_window>
    <p id={js|battlefield-controls|js} className={js|status|js}>
      {text(
         {js|Drag to pan. Pinch or Control or Command plus wheel to zoom. Keyboard: plus and minus to zoom, zero to reset zoom. Escape clears selection.|js},
       )}
    </p>
    <p id={js|battlefield-description|js} className={js|status|js} ariaLive={js|polite|js}>
      {text(description)}
    </p>
  </>;
};
