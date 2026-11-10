open Ui;
let audioCreditsUrl: string = [%mel.raw
  "(import.meta.env?.BASE_URL ?? '/') + 'assets/audio/palette/CREDITS.txt'"
];

let buildCommit: string = [%mel.raw "import.meta.env?.VITE_BUILD_COMMIT ?? 'development'"];

module SoundControl = {
  [@react.component]
  let make =
    React.memo((~enabled, ~setSound) =>
      <label className="setting-row sound-row" htmlFor="sound-effects">
        <span className="label"> {text("Sound effects")} </span>
        <input
          id="sound-effects"
          type_="checkbox"
          checked=enabled
          onChange={event => ignorePromise(setSound(bindChecked(event)))}
        />
      </label>
    );
};
module Settings = {
  type screen =
    | Hub
    | Saves;
  type testMap =
    | Developer
    | BellNight
    | MirrorNight
    | UndergroundLair;
  [@mel.set] external dataWindow: (ReactDOM.domProps, string) => unit = "data-window";
  [@react.component]
  let make =
      (
        ~session,
        ~view: Session.model,
        ~actions: Session.actions,
        ~dialog: React.ref(Js.Nullable.t(Dom.element)),
        ~onNewBattle,
        ~onRepair,
        ~openKey=0,
        ~initialScreen=Hub,
        ~focusId=None,
        ~showCommanderArrow,
        ~commanderArrowError,
        ~onCommanderArrowChange,
        ~enemyDetailsError,
        ~onEnemyDetailsChange,
      ) => {
    let (screen, setScreen) = React.useState(() => Hub);
    let (saving, setSaving) = React.useState(() => false);
    let (saved, setSaved) = React.useState(() => false);
    let (refreshKey, setRefreshKey) = React.useState(() => 0);
    let body = React.useRef(Js.Nullable.null);
    let positions = React.useRef([]);
    let pendingFocus = React.useRef(None);
    let remember = () =>
      Js.Nullable.toOption(body.current)
      |> Option.iter(node =>
           positions.current = [
             (screen, scrollTop(node)),
             ...List.remove_assoc(screen, positions.current),
           ]
         );
    let dismiss = () => {
      remember();
      close(dialog);
    };
    let navigate = next =>
      if (next == Saves
          && Option.fold(
               ~none=false,
               ~some=p => Save_json.text(p, "kind") != "ready",
               view.load_problem,
             )) {
        onRepair();
      } else {
        remember();
        pendingFocus.current = Some(screen);
        setScreen(_ => next);
        if (next == Saves) {
          setRefreshKey(value => value + 1);
        };
      };
    React.useEffect1(
      () => {
        setScreen(_ => initialScreen);
        setSaved(_ => false);
        if (initialScreen == Saves) {
          setRefreshKey(value => value + 1);
        };
        None;
      },
      [|openKey|],
    );
    React.useLayoutEffect2(
      () => {
        if (Option.fold(~none=false, ~some=isOpen, Js.Nullable.toOption(dialog.current))) {
          Js.Nullable.toOption(body.current)
          |> Option.iter(node => {
               setScrollTop(
                 node,
                 Option.value(~default=0., List.assoc_opt(screen, positions.current)),
               );
               if (screen == Hub) {
                 let selector =
                   switch (pendingFocus.current) {
                   | Some(Saves) => Some("[data-window='saves']")
                   | _ => Option.map(id => "#" ++ id, focusId)
                   };
                 Option.iter(
                   selector =>
                     Js.Nullable.toOption(queryWithin(node, selector))
                     |> Option.iter(target => focusWith(target, { "preventScroll": true })),
                   selector,
                 );
               };
             });
          if (screen != Hub) {
            Js.Nullable.toOption(dialog.current)
            |> Option.iter(node =>
                 Js.Nullable.toOption(queryWithin(node, "h2"))
                 |> Option.iter(target => focusWith(target, { "preventScroll": true }))
               );
          };
        };
        pendingFocus.current = None;
        None;
      },
      (screen, openKey),
    );
    let (starting, setStarting) = React.useState(() => false);
    let startGuard = React.useRef(false);
    let testBattle = React.useRef(None);
    let confirmation = React.useRef(Js.Nullable.null);
    let requestTest = id => {
      testBattle.current = Some(id);
      show(confirmation);
    };
    let finish = () => {
      startGuard.current = false;
      setStarting(_ => false);
    };
    let startTest = () =>
      if (!startGuard.current) {
        Option.iter(
          id => {
            startGuard.current = true;
            setStarting(_ => true);
            try(
              {
                switch (id) {
                | Developer => actions.reset("developer")
                | UndergroundLair => actions.reset("underground-lair")
                | BellNight => actions.start_campaign("bell-night")
                | MirrorNight => actions.start_campaign("mirror-night")
                };
                let _ =
                  Js.Promise.resolve()
                  |> Js.Promise.then_(() => {
                       try(
                         {
                           close(confirmation);
                           dismiss();
                         }
                       ) {
                       | error => Session.error(session, error)
                       };
                       finish();
                       Js.Promise.resolve();
                     });
                ();
              }
            ) {
            | error =>
              Session.error(session, error);
              finish();
            };
          },
          testBattle.current,
        );
      };
    let (difficulty, _, _) = Setup.opponent(view.setup);
    let state = Native.state(view.game);
    let saveGame = () => {
      setSaving(_ => true);
      let _ =
        Session.save_game(session)
        |> Js.Promise.then_(() => {
             setSaved(_ => true);
             setSaving(_ => false);
             Js.Promise.resolve();
           })
        |> Js.Promise.catch(_ => {
             setSaving(_ => false);
             Js.Promise.resolve();
           });
      ();
    };
    let savesProps = ReactDOM.domProps(~className="new-game", ~onClick=_ => navigate(Saves), ());
    dataWindow(savesProps, "saves");
    <div className="settings-style">
      <Dialog_window
        title={screen == Hub ? "Menu" : "Saved games"}
        dialog
        id="settings-dialog"
        onBeforeClose=remember
        tall={screen != Hub}
        width={screen == Hub ? "420px" : "500px"}
        onBack=?{screen == Hub ? None : Some(() => navigate(Hub))}
        body>
        <div className="settings-hub" hidden={screen != Hub}>
          <button
            className="new-game"
            disabled={
              saving
              || saved
              || view.settling
              || view.awaiting_new_game
              || view.load_problem != None
              || state.turn != Domain.Side.Player
              && state.winner == None
            }
            onClick={_ => saveGame()}>
            {text(saving ? "Saving…" : saved ? "Saved" : "Save game")}
          </button>
          {ReactDOM.createElement(
             "button",
             ~props=savesProps,
             [|
               text(
                 Option.fold(
                   ~none=false,
                   ~some=p => Save_json.text(p, "kind") != "ready",
                   view.load_problem,
                 )
                   ? "Repair saved games" : "Saved games",
               ),
             |],
           )}
          <button
            id="new-game"
            className="new-game"
            onClick={_ => {
              remember();
              onNewBattle();
            }}>
            {text("New game")}
          </button>
          <SoundControl enabled={view.sound} setSound={actions.set_sound} />
          <details className="advanced">
            <summary className="setting-row">
              <span className="label"> {text("Advanced")} </span>
              <svg className="chevron" viewBox="0 0 16 16" ariaHidden=true>
                <path d="m3 6 5 5 5-5" />
              </svg>
            </summary>
            <label className="setting-row sound-row">
              <span className="label"> {text("Commander return arrow")} </span>
              <input
                type_="checkbox"
                checked=showCommanderArrow
                onChange={event => onCommanderArrowChange(bindChecked(event))}
              />
            </label>
            <p className="help">
              {text("Show a return arrow when your commander is outside the map view.")}
            </p>
            {commanderArrowError != ""
               ? <div role="alert">
                   <p>
                     {text(
                        "The commander arrow preference could not be read or saved. Changes apply to this session.",
                      )}
                   </p>
                   <pre> {text(commanderArrowError)} </pre>
                   <button onClick={_ => onCommanderArrowChange(showCommanderArrow)}>
                     {text("Retry saving preference")}
                   </button>
                 </div>
               : React.null}
            <label className="setting-row sound-row">
              <span className="label"> {text("Automatically show enemy combat details")} </span>
              <input
                type_="checkbox"
                checked={view.enemy_combat_details}
                onChange={event => onEnemyDetailsChange(bindChecked(event))}
              />
            </label>
            <p className="help">
              {text(
                 "Open the combat report when an enemy attacks. Otherwise keep your current inspection.",
               )}
            </p>
            {enemyDetailsError != ""
               ? <div role="alert">
                   <p>
                     {text(
                        "The enemy combat details preference could not be read or saved. Changes apply to this session.",
                      )}
                   </p>
                   <pre> {text(enemyDetailsError)} </pre>
                   <button onClick={_ => onEnemyDetailsChange(view.enemy_combat_details)}>
                     {text("Retry saving preference")}
                   </button>
                 </div>
               : React.null}
            <h3 className="group-label"> {text("Debug tools")} </h3>
            <p className="help">
              {text(
                 "Download the battle, startup timings, last five enemy turns, and saved progress. Includes enemies hidden by fog. Enemy-turn recordings clear on reload or mission change.",
               )}
            </p>
            <button className="debug-action" onClick={_ => actions.download_debug_report()}>
              {text("Download debug report")}
            </button>
            <h3 className="group-label"> {text("Performance trace")} </h3>
            <Trace_controls session onStart=dismiss />
            <h3 className="group-label"> {text("Test maps")} </h3>
            <p className="help"> {text("Starting a test map replaces the current battle.")} </p>
            {eachList(
               [
                 (Developer, "Developer map"),
                 (UndergroundLair, "Underground Lair"),
                 (BellNight, "Bell Night"),
                 (MirrorNight, "Mirror Night"),
               ],
               ((id, label)) =>
               <button
                 className="debug-action test-launch"
                 disabled=starting
                 onClick={_ => requestTest(id)}>
                 <svg className="flask" viewBox="0 0 24 24" ariaHidden=true>
                   <path
                     d="M9 3h6M10 3v6l-6 10a1.3 1.3 0 0 0 1.1 2h13.8a1.3 1.3 0 0 0 1.1-2L14 9V3M7.5 15h9"
                   />
                 </svg>
                 <span> {text("Start " ++ label)} </span>
               </button>
             )}
          </details>
          <a
            className="credits-link"
            href=audioCreditsUrl
            target="_blank"
            rel="noreferrer"
            ariaLabel="Credits (opens in a new tab)">
            {text("Credits ")}
            <svg viewBox="0 0 24 24" ariaHidden=true>
              <path
                d="M14 4h6v6M20 4 10 14M10 5H5a1 1 0 0 0-1 1v13a1 1 0 0 0 1 1h13a1 1 0 0 0 1-1v-5"
              />
            </svg>
          </a>
          <footer>
            <span> {text("Difficulty: " ++ difficulty)} </span>
            <span> {text("Build: " ++ buildCommit)} </span>
          </footer>
        </div>
        <div hidden={screen != Saves}>
          {screen == Saves ? <Saved_games session view refreshKey onLoad=dismiss /> : React.null}
        </div>
      </Dialog_window>
      <Dialog_window
        title="Start test map?" dialog=confirmation canDismiss={!starting} width="380px">
        <p>
          {text(
             "This replaces your current battle. Saved progress remains available in Saved games.",
           )}
        </p>
        <button disabled=starting onClick={_ => close(confirmation)}> {text("Cancel")} </button>
        <button disabled=starting onClick={_ => startTest()}>
          {text(starting ? "Starting…" : "Start test map")}
        </button>
      </Dialog_window>
    </div>;
  };
};

module Dialogue = {
  [@react.component]
  let make = (~view: Session.model, ~actions: Session.actions, ~narrate) => {
    let state = Native.state(view.game);
    let dialog = React.useRef(Js.Nullable.null);
    let nextButton = React.useRef(Js.Nullable.null);
    let queued = Story.dialogue_pending(state.campaign);
    let line =
      Option.bind(
        state.campaign,
        (campaign: Domain.campaign_state) => {
          let index = int_of_float(campaign.dialogue_index);
          index >= 0 && index < Array.length(campaign.dialogue)
            ? Some(campaign.dialogue[index]) : None;
        },
      );
    let pending =
      queued && Option.fold(~none=false, ~some=(line: Domain.dialogue) => line.text != "", line);
    React.useEffect2(
      () => {
        narrate(view.game);
        if (queued && !pending && !view.settling) {
          actions.dialogue(false);
        };
        None;
      },
      (view.game, view.settling),
    );
    React.useLayoutEffect2(
      () => {
        if (pending) {
          switch (Js.Nullable.toOption(dialog.current)) {
          | Some(element) when !isOpen(element) =>
            showModal(element);
            Js.Nullable.toOption(nextButton.current) |> Option.iter(focus);
          | _ => ()
          };
        } else {
          let wasOpen =
            switch (Js.Nullable.toOption(dialog.current)) {
            | None => false
            | Some(element) => isOpen(element)
            };
          close(dialog);
          if (wasOpen && state.winner != None) {
            byId("mission-result")
            |> Js.Nullable.toOption
            |> Option.iter(element => {
                 focus(element);
                 scroll(element, { "block": "start" });
               });
          };
        };
        None;
      },
      (pending, view.game),
    );
    let navigate = event =>
      if (!view.settling
          && !React.Event.Keyboard.altKey(event)
          && !React.Event.Keyboard.ctrlKey(event)
          && !React.Event.Keyboard.metaKey(event)
          && !React.Event.Keyboard.shiftKey(event)) {
        switch (React.Event.Keyboard.key(event)) {
        | "ArrowLeft" =>
          React.Event.Keyboard.preventDefault(event);
          if (!React.Event.Keyboard.repeat(event)) {
            actions.previous_dialogue();
          };
        | "ArrowRight" =>
          React.Event.Keyboard.preventDefault(event);
          if (!React.Event.Keyboard.repeat(event)) {
            actions.dialogue(false);
          };
        | _ => ()
        };
      };
    let props =
      ReactDOM.domProps(
        ~className="story-dialog",
        ~ref=ReactDOM.Ref.domRef(dialog),
        ~onKeyDown=navigate,
        ~ariaLabelledby="dialogue-speaker",
        (),
      );
    onCancel(props, React.Event.Form.preventDefault);
    ReactDOM.createElement(
      "dialog",
      ~props,
      [|
        option(
          Native.campaign(view.game),
          campaign => {
            let index = int_of_float(campaign.Domain.dialogue_index);
            if (!pending || index < 0 || index >= Array.length(campaign.dialogue)) {
              React.null;
            } else {
              let line = campaign.dialogue[index];
              let chapter = Story.mission(view.catalog, view.game, campaign);
              let forward =
                index + 1 == Array.length(campaign.dialogue)
                  ? state.winner != None ? "Review mission result" : "Return to battle"
                  : "Next line";
              <>
                <p className="story-heading">
                  {text(
                     Catalog.mission_map(view.catalog, chapter.source).name
                     ++ (state.winner != None ? " — After the battle" : ""),
                   )}
                </p>
                <div className="story-line" ariaLive="polite" ariaAtomic=true>
                  <div className="story-portrait">
                    <Sprite
                      artwork={view.artwork}
                      kind="portraits"
                      name={line.portrait}
                      width=90.
                      height=120.
                    />
                  </div>
                  <div>
                    <h2 id="dialogue-speaker"> {text(line.speaker)} </h2>
                    <p className="story-text"> {text(line.text)} </p>
                  </div>
                </div>
                <nav className="story-navigation" ariaLabel="Conversation">
                  <button
                    className="story-back"
                    disabled={index == 0 || view.settling}
                    onPointerDown={Button_press.pointer_down(actions.previous_dialogue)}
                    onClick={Button_press.click(actions.previous_dialogue)}
                    ariaLabel="Previous line"
                    title="Previous line (Left arrow)">
                    <Icon name="left" />
                  </button>
                  <span className="story-count">
                    {text(
                       string_of_int(index + 1)
                       ++ " / "
                       ++ string_of_int(Array.length(campaign.dialogue)),
                     )}
                  </span>
                  <button
                    className="story-next"
                    disabled={view.settling}
                    ref={ReactDOM.Ref.domRef(nextButton)}
                    onPointerDown={Button_press.pointer_down(() => actions.dialogue(false))}
                    onClick={Button_press.click(() => actions.dialogue(false))}
                    ariaLabel=forward
                    title={Utf8.to_js(forward ++ " (Right arrow)")}>
                    <Icon name="right" />
                  </button>
                </nav>
              </>;
            };
          },
        ),
      |],
    );
  };
};
