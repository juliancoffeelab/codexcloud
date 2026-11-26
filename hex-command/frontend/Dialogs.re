open Ui;
let audioCreditsUrl: string = [%mel.raw
  {js|(import.meta.env?.BASE_URL ?? '/') + 'assets/audio/palette/CREDITS.txt'|js}
];
[@mel.get] external selectTarget: React.Event.Form.t => {. "value": string } = "currentTarget";
[@mel.get] external selectValue: {. "value": string } => string = "value";
let buildCommit: string = [%mel.raw {js|import.meta.env?.VITE_BUILD_COMMIT ?? 'development'|js}];
module SoundControl = {
  [@react.component]
  let make =
    React.memo((~enabled, ~setSound) =>
      <label className={js|setting-row sound-row|js} htmlFor={js|sound-effects|js}>
        <span className={js|label|js}> {text({js|Sound effects|js})} </span>
        <input
          id={js|sound-effects|js}
          type_={js|checkbox|js}
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
        ~allowNonHumanMaps,
        ~onNonHumanMapsChange,
        ~onFollowAiChange,
        ~playtestPreferenceError,
        ~view: Session.model,
        ~voiceEnabled,
        ~onVoiceChange,
        ~actions: Session.actions,
        ~opened,
        ~onOpenChange,
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
        ~children,
      ) => {
    let (screen, setScreen) = React.useState(() => Hub);
    let (saving, setSaving) = React.useState(() => false);
    let (saved, setSaved) = React.useState(() => false);
    let (refreshKey, setRefreshKey) = React.useState(() => 0);
    let body = React.useRef(Js.Nullable.null);
    let content = React.useRef(Js.Nullable.null);
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
      onOpenChange(false);
    };
    let navigate = next =>
      if (next == Saves
          && Option.fold(
               ~none=false,
               ~some=p => Save_json.text(p, {js|kind|js}) != {js|ready|js},
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
    let restoreScreen = () => {
      Option.iter(
        node => {
          setScrollTop(
            node,
            Option.value(~default=0., List.assoc_opt(screen, positions.current)),
          );
          if (screen == Hub) {
            let selector =
              switch (pendingFocus.current) {
              | Some(Saves) => {js|[data-window='saves']|js}
              | _ =>
                Option.fold(
                  ~none={js|.settings-hub button:not(:disabled)|js},
                  ~some=id => {j|#$(id)|j},
                  focusId,
                )
              };
            Option.iter(
              target => focusWith(target, { "preventScroll": true }),
              Js.Nullable.toOption(queryWithin(node, selector)),
            );
          };
        },
        Js.Nullable.toOption(body.current),
      );
      if (screen != Hub) {
        Option.iter(
          node =>
            Option.iter(
              target => focusWith(target, { "preventScroll": true }),
              Js.Nullable.toOption(queryWithin(node, {js|h2|js})),
            ),
          Js.Nullable.toOption(content.current),
        );
      };
      pendingFocus.current = None;
    };
    React.useLayoutEffect3(
      () => {
        if (opened) {
          restoreScreen();
        };
        None;
      },
      (screen, openKey, opened),
    );
    let (starting, setStarting) = React.useState(() => false);
    let startGuard = React.useRef(false);
    let testBattle = React.useRef(None);
    let (confirmOpen, setConfirmOpen) = React.useState(() => false);
    let requestTest = id => {
      testBattle.current = Some(id);
      setConfirmOpen(_ => true);
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
                | Developer => actions.reset({js|developer|js})
                | UndergroundLair => actions.reset({js|underground-lair|js})
                | BellNight => actions.start_campaign({js|bell-night|js})
                | MirrorNight => actions.start_campaign({js|mirror-night|js})
                };
                let _ =
                  Js.Promise.resolve()
                  |> Js.Promise.then_(() => {
                       try(
                         {
                           setConfirmOpen(_ => false);
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
    let testLaunch = (id, label, trigger) =>
      <Dialog_window
        key=label
        trigger
        title={js|Start test map?|js}
        opened={confirmOpen && testBattle.current == Some(id)}
        onOpenChange={value => setConfirmOpen(_ => value)}
        alert=true
        canDismiss={!starting}
        width={js|380px|js}>
        <p>
          {text(
             {js|This replaces your current battle. Saved progress remains available in Saved games.|js},
           )}
        </p>
        <Radix.AlertCancel asChild=true>
          <button disabled=starting onClick={_ => setConfirmOpen(_ => false)}>
            {text({js|Cancel|js})}
          </button>
        </Radix.AlertCancel>
        <button disabled=starting onClick={_ => startTest()}>
          {text(starting ? {js|Starting…|js} : {js|Start test map|js})}
        </button>
      </Dialog_window>;
    let savesProps =
      ReactDOM.domProps(~className={js|new-game|js}, ~onClick=_ => navigate(Saves), ());
    dataWindow(savesProps, {js|saves|js});
    <Modal.Root
      opened
      onOpenChange={value => {
        if (!value) {
          remember();
        };
        onOpenChange(value);
      }}>
      children
      <Dialog_window.Content
        title={screen == Hub ? {js|Menu|js} : {js|Saved games|js}}
        onOpenChange
        contentRef=content
        className={js|settings-style|js}
        id={js|settings-dialog|js}
        onBeforeClose=remember
        onOpenAutoFocus={event => {
          Modal.preventDefault(event);
          restoreScreen();
        }}
        tall={screen != Hub}
        width={screen == Hub ? {js|420px|js} : {js|500px|js}}
        onBack=?{screen == Hub ? None : Some(() => navigate(Hub))}
        body>
        <div className={js|settings-hub|js} hidden={screen != Hub}>
          <button
            className={js|new-game|js}
            disabled={
              saving
              || saved
              || view.settling
              || view.awaiting_new_game
              || view.load_problem != None
              || state.turn != Domain.Turn_phase.Player
              && state.winner == None
            }
            onClick={_ => saveGame()}>
            {text(saving ? {js|Saving…|js} : saved ? {js|Saved|js} : {js|Save game|js})}
          </button>
          {ReactDOM.createElement(
             {js|button|js},
             ~props=savesProps,
             [|
               text(
                 Option.fold(
                   ~none=false,
                   ~some=p => Save_json.text(p, {js|kind|js}) != {js|ready|js},
                   view.load_problem,
                 )
                   ? {js|Repair saved games|js} : {js|Saved games|js},
               ),
             |],
           )}
          <button
            id={js|new-game|js}
            className={js|new-game|js}
            onClick={_ => {
              remember();
              onNewBattle();
            }}>
            {text({js|New game|js})}
          </button>
          <SoundControl enabled={view.sound} setSound={actions.set_sound} />
          <label className={js|setting-row sound-row|js} htmlFor={js|dialogue-voices|js}>
            <span className={js|label|js}> {text({js|Dialogue voices|js})} </span>
            <input
              id={js|dialogue-voices|js}
              type_={js|checkbox|js}
              checked=voiceEnabled
              onChange={event => onVoiceChange(bindChecked(event))}
            />
          </label>
          <details className={js|advanced|js}>
            <summary className={js|setting-row|js}>
              <span className={js|label|js}> {text({js|Advanced|js})} </span>
              <svg className={js|chevron|js} viewBox={js|0 0 16 16|js} ariaHidden=true>
                <path d={js|m3 6 5 5 5-5|js} />
              </svg>
            </summary>
            <label className={js|setting-row sound-row|js}>
              <span className={js|label|js}> {text({js|Follow AI camera|js})} </span>
              <input
                type_={js|checkbox|js}
                checked={view.follow_ai}
                onChange={e => onFollowAiChange(bindChecked(e))}
              />
            </label>
            <label className={js|setting-row|js}>
              <span className={js|label|js}> {text({js|Player control|js})} </span>
              <select
                ariaLabel={js|Player control|js}
                disabled={state.turn != Domain.Turn_phase.Player}
                value={
                  view.learned_player
                    ? {js|learned|js}
                    : Option.fold(
                        ~none={js|human|js},
                        ~some=p => Setup.preset(p).id,
                        view.player_ai,
                      )
                }
                onChange={event => {
                  let selected = selectValue(selectTarget(event));
                  let policy =
                    List.find_opt(
                      (preset: Setup.difficulty) => preset.id == selected,
                      Setup.difficulties,
                    );
                  if (selected == {js|learned|js}) {
                    Session.set_learned_player(session);
                  } else {
                    Session.set_player_ai(
                      session,
                      Option.map((preset: Setup.difficulty) => preset.policy, policy),
                    );
                  };
                }}>
                <option value={js|human|js}> {text({js|You|js})} </option>
                <option value={js|learned|js}> {text({js|Learned AI · hour 13|js})} </option>
                {eachList(Setup.difficulties, preset =>
                   <option key={preset.Setup.id} value={preset.id}>
                     {text(
                        {
                          let js_text_part_0 = preset.name;
                          {j|$(js_text_part_0) AI|j};
                        },
                      )}
                   </option>
                 )}
              </select>
            </label>
            {view.learned_player
               ? <p className={js|help|js}>
                   {text(
                      {js|Learned planner with Predictor recruitment. Learned control lasts for this session.|js},
                    )}
                 </p>
               : React.null}
            {playtestPreferenceError != {js||js}
               ? <p role={js|alert|js}> {text(playtestPreferenceError)} </p> : React.null}
            <label className={js|setting-row sound-row|js}>
              <span className={js|label|js}> {text({js|Allow non-human maps|js})} </span>
              <input
                type_={js|checkbox|js}
                checked=allowNonHumanMaps
                onChange={e => onNonHumanMapsChange(bindChecked(e))}
              />
            </label>
            <label className={js|setting-row sound-row|js}>
              <span className={js|label|js}> {text({js|Commander return arrow|js})} </span>
              <input
                type_={js|checkbox|js}
                checked=showCommanderArrow
                onChange={event => onCommanderArrowChange(bindChecked(event))}
              />
            </label>
            <p className={js|help|js}>
              {text({js|Show a return arrow when your commander is outside the map view.|js})}
            </p>
            {commanderArrowError != {js||js}
               ? <div role={js|alert|js}>
                   <p>
                     {text(
                        {js|The commander arrow preference could not be read or saved. Changes apply to this session.|js},
                      )}
                   </p>
                   <pre> {text(commanderArrowError)} </pre>
                   <button onClick={_ => onCommanderArrowChange(showCommanderArrow)}>
                     {text({js|Retry saving preference|js})}
                   </button>
                 </div>
               : React.null}
            <label className={js|setting-row sound-row|js}>
              <span className={js|label|js}>
                {text({js|Automatically show enemy combat details|js})}
              </span>
              <input
                type_={js|checkbox|js}
                checked={view.enemy_combat_details}
                onChange={event => onEnemyDetailsChange(bindChecked(event))}
              />
            </label>
            <p className={js|help|js}>
              {text(
                 {js|Open the combat report when an enemy attacks. Otherwise keep your current inspection.|js},
               )}
            </p>
            {enemyDetailsError != {js||js}
               ? <div role={js|alert|js}>
                   <p>
                     {text(
                        {js|The enemy combat details preference could not be read or saved. Changes apply to this session.|js},
                      )}
                   </p>
                   <pre> {text(enemyDetailsError)} </pre>
                   <button onClick={_ => onEnemyDetailsChange(view.enemy_combat_details)}>
                     {text({js|Retry saving preference|js})}
                   </button>
                 </div>
               : React.null}
            <h3 className={js|group-label|js}> {text({js|Debug tools|js})} </h3>
            <p className={js|help|js}>
              {text(
                 {js|Download the battle, startup timings, last five AI turns, and saved progress. AI-turn recordings clear on reload or mission change.|js},
               )}
            </p>
            <button className={js|debug-action|js} onClick={_ => actions.download_debug_report()}>
              {text({js|Download debug report|js})}
            </button>
            <h3 className={js|group-label|js}> {text({js|Performance trace|js})} </h3>
            <Trace_controls session onStart=dismiss />
            <h3 className={js|group-label|js}> {text({js|Test maps|js})} </h3>
            <p className={js|help|js}>
              {text({js|Starting a test map replaces the current battle.|js})}
            </p>
            {eachList(
               [
                 (Developer, {js|Developer map|js}),
                 (UndergroundLair, {js|Underground Lair|js}),
                 (BellNight, {js|Bell Night|js}),
                 (MirrorNight, {js|Mirror Night|js}),
               ],
               ((id, label)) =>
               testLaunch(
                 id,
                 label,
                 <button
                   key=label
                   className={js|debug-action test-launch|js}
                   disabled=starting
                   onClick={_ => requestTest(id)}>
                   <svg className={js|flask|js} viewBox={js|0 0 24 24|js} ariaHidden=true>
                     <path
                       d={js|M9 3h6M10 3v6l-6 10a1.3 1.3 0 0 0 1.1 2h13.8a1.3 1.3 0 0 0 1.1-2L14 9V3M7.5 15h9|js}
                     />
                   </svg>
                   <span> {text({j|Start $(label)|j})} </span>
                 </button>,
               )
             )}
          </details>
          <a
            className={js|credits-link|js}
            href=audioCreditsUrl
            target={js|_blank|js}
            rel={js|noreferrer|js}
            ariaLabel={js|Credits (opens in a new tab)|js}>
            {text({js|Credits |js})}
            <svg viewBox={js|0 0 24 24|js} ariaHidden=true>
              <path
                d={js|M14 4h6v6M20 4 10 14M10 5H5a1 1 0 0 0-1 1v13a1 1 0 0 0 1 1h13a1 1 0 0 0 1-1v-5|js}
              />
            </svg>
          </a>
          <footer>
            <span> {text({j|Difficulty: $(difficulty)|j})} </span>
            <span> {text({j|Build: $(buildCommit)|j})} </span>
          </footer>
        </div>
        <div hidden={screen != Saves}>
          {screen == Saves
             ? <Saved_games session view refreshKey onLoad=dismiss onDeleted=restoreScreen />
             : React.null}
        </div>
      </Dialog_window.Content>
    </Modal.Root>;
  };
};
module VoicedText = {
  [@react.component]
  let make = (~voice, ~scope, ~lines: array(Domain.dialogue), ~index) => {
    let (visible, setVisible) = React.useState(() => {js||js});
    React.useEffect0(() =>
      Some(Voiceover.speak(voice, scope, lines, index, value => setVisible(_ => value)))
    );
    <p className={js|story-text voice-text|js}>
      <span className={js|voice-reserve|js} ariaHidden=true> {text(lines[index].text)} </span>
      <span className={js|voice-visible|js} ariaHidden=true> {React.string(visible)} </span>
      <span className={js|voice-accessible|js}> {text(lines[index].text)} </span>
    </p>;
  };
};
module Dialogue = {
  [@react.component]
  let make = (~view: Session.model, ~actions: Session.actions, ~narrate, ~voice, ~scope) => {
    let state = Native.state(view.game);
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
      queued
      && Option.fold(~none=false, ~some=(line: Domain.dialogue) => line.text != {js||js}, line);
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
    let navigate = event =>
      if (!view.settling
          && !React.Event.Keyboard.altKey(event)
          && !React.Event.Keyboard.ctrlKey(event)
          && !React.Event.Keyboard.metaKey(event)
          && !React.Event.Keyboard.shiftKey(event)) {
        switch (React.Event.Keyboard.key(event)) {
        | matched_text_0 when matched_text_0 == {js|ArrowLeft|js} =>
          React.Event.Keyboard.preventDefault(event);
          if (!React.Event.Keyboard.repeat(event)) {
            actions.previous_dialogue();
          };
        | matched_text_1 when matched_text_1 == {js|ArrowRight|js} =>
          React.Event.Keyboard.preventDefault(event);
          if (!React.Event.Keyboard.repeat(event)) {
            actions.dialogue(false);
          };
        | _ => ()
        };
      };
    <Modal
      opened=pending
      onOpenChange={_ => ()}
      canDismiss=false
      title={Option.fold(
        ~none={js|Conversation|js},
        ~some=(line: Domain.dialogue) => line.speaker,
        line,
      )}
      className={js|story-dialog|js}
      onKeyDown=navigate
      onOpenAutoFocus={event => {
        Modal.preventDefault(event);
        Option.iter(focus, Js.Nullable.toOption(nextButton.current));
      }}
      onCloseAutoFocus={event => {
        Modal.preventDefault(event);
        if (state.winner != None) {
          byId({js|mission-result|js})
          |> Js.Nullable.toOption
          |> Option.iter(element => {
               focus(element);
               scroll(element, { "block": {js|start|js} });
             });
        } else {
          Option.iter(focus, Js.Nullable.toOption(byId({js|settings-button|js})));
        };
      }}>
      {option(
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
                 ? state.winner != None ? {js|Review mission result|js} : {js|Return to battle|js}
                 : {js|Next line|js};
             <>
               <p className={js|story-heading|js}>
                 {text(
                    {
                      let js_text_part_0 = Catalog.mission_map(view.catalog, chapter.source).name;
                      let js_text_part_1 =
                        state.winner != None ? {js| — After the battle|js} : {js||js};
                      {j|$(js_text_part_0)$(js_text_part_1)|j};
                    },
                  )}
               </p>
               <div className={js|story-line|js} ariaLive={js|polite|js} ariaAtomic=true>
                 <div className={js|story-portrait|js}>
                   <Sprite
                     artwork={view.artwork}
                     kind={js|portraits|js}
                     name={line.portrait}
                     width=90.
                     height=120.
                   />
                 </div>
                 <div>
                   <h2 id={js|dialogue-speaker|js}> {text(line.speaker)} </h2>
                   <VoicedText
                     key={
                           let js_text_part_2 = string_of_int(index);
                           let js_text_part_4 = line.text;
                           {j|$(scope):$(js_text_part_2):$(js_text_part_4)|j};
                         }
                     voice
                     scope
                     lines={campaign.dialogue}
                     index
                   />
                 </div>
               </div>
               <nav className={js|story-navigation|js} ariaLabel={js|Conversation|js}>
                 <button
                   className={js|story-back|js}
                   disabled={index == 0 || view.settling}
                   onClick={_ => actions.previous_dialogue()}
                   ariaLabel={js|Previous line|js}
                   title={js|Previous line (Left arrow)|js}>
                   <Icon name={js|left|js} />
                 </button>
                 <span className={js|story-count|js}>
                   {text(
                      {
                        let js_text_part_0 = string_of_int(index + 1);
                        let js_text_part_2 = string_of_int(Array.length(campaign.dialogue));
                        {j|$(js_text_part_0) / $(js_text_part_2)|j};
                      },
                    )}
                 </span>
                 <button
                   className={js|story-next|js}
                   disabled={view.settling}
                   ref={ReactDOM.Ref.domRef(nextButton)}
                   onClick={_ => actions.dialogue(false)}
                   ariaLabel=forward
                   title={j|$(forward) (Right arrow)|j}>
                   <Icon name={js|right|js} />
                 </button>
               </nav>
             </>;
           };
         },
       )}
    </Modal>;
  };
};
