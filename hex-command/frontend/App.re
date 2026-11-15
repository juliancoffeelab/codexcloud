open Ui;
[@mel.send] external controlMatches: (Dom.element, string) => bool = "matches";
[@mel.send]
external controlAttribute: (Dom.element, string) => Js.Nullable.t(string) = "getAttribute";
let controlCue = control =>
  if (controlMatches(control, {js|button[role='radio'][data-faction]|js})) {
    Js.Nullable.toOption(controlAttribute(control, {js|data-faction|js}))
    |> Option.fold(~none=Phaser_sound.Choice, ~some=Phaser_sound.faction_hud_cue);
  } else if (controlMatches(control, {js|summary, button[aria-expanded]|js})) {
    Phaser_sound.Disclosure;
  } else if (controlMatches(
               control,
               {js|select, input[type='checkbox'], input[type='radio'], input[type='range'], button[aria-pressed], button[role='radio'], button[role='checkbox'], button[role='switch']|js},
             )) {
    Phaser_sound.Choice;
  } else {
    Phaser_sound.Action;
  };
module TurnBar = {
  [@react.component]
  let make =
      (
        ~view: Session.model,
        ~actions: Session.actions,
        ~ready,
        ~onSettings,
        ~briefing,
        ~onBriefing,
      ) => {
    let state = Native.state(view.game);
    let treasury = view.treasury;
    let income = Native.income(view.game, treasury);
    let villages =
      Array.fold_left(
        (count, tile: Domain.tile) =>
          count
          + (
            Catalog.terrain_role(view.catalog, tile.Domain.terrain) == Domain.Terrain_role.Village
              ? 1 : 0
          ),
        0,
        state.tiles,
      );
    let winner = ready ? state.winner : None;
    let canAct =
      ready
      && state.turn == Domain.Turn_phase.Player
      && winner == None
      && view.player_ai == None
      && !view.settling
      && !view.awaiting_new_game
      && !Story.dialogue_pending(state.campaign)
      && Option.fold(
           ~none=true,
           ~some=(frame: Combat_playback.frame) => frame.complete,
           view.combat_playback,
         );
    <Header
      stats={
        Some({
          Header.gold: Native.gold(state.gold, treasury),
          income,
          owned_villages:
            float_of_int(
              Array.fold_left(
                (count, tile: Domain.tile) =>
                  count
                  + (
                    Catalog.terrain_role(view.catalog, tile.terrain)
                    == Domain.Terrain_role.Village
                    && tile.owner == Some(treasury)
                      ? 1 : 0
                  ),
                0,
                state.tiles,
              ),
            ),
          total_villages: float_of_int(villages),
          round: state.round,
        })
      }
      turnLabel={
        !ready
          ? {js|Loading…|js}
          : winner == Some(Domain.Battle_result.Player_victory)
              ? {js|Victory|js}
              : winner == Some(Domain.Battle_result.Player_defeat)
                  ? {js|Defeat|js}
                  : state.turn == Domain.Turn_phase.Player && view.player_ai == None
                      ? {js|End turn|js} : {js|AI turn|js}
      }
      finished={winner != None}
      canEndTurn=canAct
      canUndo={view.can_undo}
      campaign={state.campaign != None}
      briefing
      onEndTurn={() => {
        let _ = actions.end_turn();
        ();
      }}
      onUndo={actions.undo}
      onSettings={Some(onSettings)}
      onBriefing={Some(onBriefing)}
    />;
  };
};
[@ocaml.doc
  {js| The root is the sole session subscriber. Descendants receive one detached\n    publication and a stable command capability; effects acknowledge DOM commit. |js}
]

[@react.component]
let make = (~session: Session.t, ~renderer: Phaser_boot.t, ~onMount: (unit, unit) => unit) => {
  let subscribe =
    React.useCallback1(callback => Session.subscribe(session, _ => callback()), [|session|]);
  let getSnapshot = React.useCallback1(() => Session.view(session), [|session|]);
  let view = React.useSyncExternalStore(~subscribe, ~getSnapshot);
  let voice = React.useMemo1(() => Voiceover.create(), [|session|]);
  let (voiceEnabled, setVoiceEnabled) = React.useState(() => Voiceover.enabled(voice));
  let changeVoice = enabled => {
    Voiceover.set_enabled(voice, enabled);
    setVoiceEnabled(_ => enabled);
  };
  React.useEffect1(() => Some(() => Voiceover.close(voice)), [|voice|]);
  let (battlefieldProgress, setBattlefieldProgress) = React.useState(() => None);
  let previews = React.useMemo0(() => Terrain_preview.create());
  React.useEffect1(() => Some(() => Terrain_preview.close(previews)), [|view.artwork|]);
  let actions = Session.commands(session);
  React.useEffect1(
    () => {
      let cue = (selector, event) => {
        let node = target(event);
        if (trusted(event)
            && Js.Nullable.toOption(closest(node, {js|.ng-strip [data-faction]|js})) == None
            && nodeType(node) == 1
            && Js.Nullable.toOption(closest(node, selector)) != None
            && Js.Nullable.toOption(closest(node, {js|:disabled, [aria-disabled='true']|js}))
            == None) {
          let control = Js.Nullable.toOption(closest(node, selector)) |> Option.get;
          let cue = controlCue(control);
          let _ = Session.menu_cue(~cue, session);
          ();
        };
      };
      let click =
        cue(
          {js|button, a[href], summary, input[type='button'], input[type='submit'], input[type='reset']|js},
        );
      let change =
        cue(
          {js|select, input[type='checkbox'], input[type='radio'], input[type='range'], input[type='file']|js},
        );
      let detachPress = Press_button.installGuard(cue({js|button|js}));
      Browser_audio.listen(
        {js|click|js},
        click,
        {
          "capture": true,
          "passive": true,
        },
      );
      Browser_audio.listen(
        {js|change|js},
        change,
        {
          "capture": true,
          "passive": true,
        },
      );
      Some(
        () => {
          Press_button.dispose(detachPress);
          Browser_audio.unlisten({js|click|js}, click, true);
          Browser_audio.unlisten({js|change|js}, change, true);
        },
      );
    },
    [|session|],
  );
  let (allowNonHumanMaps, setAllowNonHumanMaps) = React.useState(() => false);
  let (campaignTags, setCampaignTags) = React.useState(() => []);
  let (scenarioTags, setScenarioTags) = React.useState(() => []);
  let (playtestPreferenceError, setPlaytestPreferenceError) = React.useState(() => {js||js});
  let savePreference = (key, value) => {
    setPlaytestPreferenceError(_ => {js||js});
    try(Browser_storage.storage_set(key, Js.Json.stringify(value))) {
    | error => setPlaytestPreferenceError(_ => Foreign_error.exception_(error))
    };
  };
  let changeNonHumanMaps = enabled => {
    setAllowNonHumanMaps(_ => enabled);
    savePreference({js|hex-command-show-hidden-maps|js}, Js.Json.boolean(enabled));
  };
  let changeFollowAi = enabled => {
    Session.set_follow_ai(session, enabled);
    savePreference({js|hex-command-follow-ai-camera|js}, Js.Json.boolean(enabled));
  };
  let filterKey = mode =>
    switch (mode) {
    | Setup.Campaign => {js|hex-command-campaign-map-tags|js}
    | Setup.Skirmish => {js|hex-command-scenario-map-tags|js}
    };
  let changeMapTags = (mode, tags) => {
    let setTags = mode == Setup.Campaign ? setCampaignTags : setScenarioTags;
    setTags(_ => tags);
    savePreference(filterKey(mode), Js.Json.stringArray(Array.of_list(tags)));
  };
  let readMapTags = mode =>
    try(
      switch (Js.Nullable.toOption(Browser_storage.storage_get(filterKey(mode)))) {
      | None => []
      | Some(value) =>
        switch (Js.Json.decodeArray(Js.Json.parseExn(value))) {
        | None => invalid_arg({js|Invalid saved map filters|js})
        | Some(tags) =>
          Array.to_list(tags)
          |> List.map(tag =>
               switch (Js.Json.decodeString(tag)) {
               | Some(tag) when tag != {js||js} => tag
               | _ => invalid_arg({js|Invalid saved map filters|js})
               }
             )
        }
      }
    ) {
    | error =>
      setPlaytestPreferenceError(_ => Foreign_error.exception_(error));
      [];
    };
  React.useEffect0(() => {
    try(
      {
        setAllowNonHumanMaps(_ =>
          Js.Nullable.toOption(Browser_storage.storage_get({js|hex-command-show-hidden-maps|js}))
          == Some({js|true|js})
        );
        Session.set_follow_ai(
          session,
          Js.Nullable.toOption(Browser_storage.storage_get({js|hex-command-follow-ai-camera|js}))
          != Some({js|false|js}),
        );
        let campaignTags = readMapTags(Setup.Campaign);
        let scenarioTags = readMapTags(Setup.Skirmish);
        setCampaignTags(_ => campaignTags);
        setScenarioTags(_ => scenarioTags);
      }
    ) {
    | error => setPlaytestPreferenceError(_ => Foreign_error.exception_(error))
    };
    None;
  });
  React.useEffect1(
    () => {
      Session.continue_ai(session);
      None;
    },
    [|view|],
  );
  let (showCommanderArrow, setShowCommanderArrow) = React.useState(() => false);
  let (commanderArrowError, setCommanderArrowError) = React.useState(() => {js||js});
  let changeCommanderArrow = enabled => {
    setShowCommanderArrow(_ => enabled);
    setCommanderArrowError(_ => {js||js});
    try(
      Browser_storage.storage_set({js|hex-command-commander-arrow|js}, string_of_bool(enabled))
    ) {
    | error => setCommanderArrowError(_ => Foreign_error.exception_(error))
    };
  };
  let (enemyDetailsError, setEnemyDetailsError) = React.useState(() => {js||js});
  let changeEnemyDetails = enabled => {
    Session.set_enemy_combat_details(session, enabled);
    setEnemyDetailsError(_ => {js||js});
    try(
      Browser_storage.storage_set(
        {js|hex-command-enemy-combat-details|js},
        string_of_bool(enabled),
      )
    ) {
    | error => setEnemyDetailsError(_ => Foreign_error.exception_(error))
    };
  };
  React.useEffect0(() => {
    let _ = Session.prepare_audio(session);
    try(
      setShowCommanderArrow(_ =>
        Js.Nullable.toOption(Browser_storage.storage_get({js|hex-command-commander-arrow|js}))
        == Some({js|true|js})
      )
    ) {
    | error => setCommanderArrowError(_ => Foreign_error.exception_(error))
    };
    try(
      Session.set_enemy_combat_details(
        session,
        Js.Nullable.toOption(
          Browser_storage.storage_get({js|hex-command-enemy-combat-details|js}),
        )
        == Some({js|true|js}),
      )
    ) {
    | error => setEnemyDetailsError(_ => Foreign_error.exception_(error))
    };
    None;
  });
  let state = Native.state(view.game);
  let identity = {
    let js_text_part_0 = string_of_int(view.session_version);
    let js_text_part_2 = state.map_id;
    {j|$(js_text_part_0):$(js_text_part_2)|j};
  };
  let (voiceOpening, setVoiceOpening) = React.useState(() => None);
  let conversation = Native.campaign(view.game);
  let spokenLines =
    Option.fold(
      ~none=0,
      ~some=
        (campaign: Domain.campaign_state) =>
          Array.fold_left(
            (count, line: Domain.dialogue) => count + (line.text == {js||js} ? 0 : 1),
            0,
            Array.sub(
              campaign.dialogue,
              min(Array.length(campaign.dialogue), int_of_float(campaign.dialogue_index)),
              max(0, Array.length(campaign.dialogue) - int_of_float(campaign.dialogue_index)),
            ),
          ),
      conversation,
    );
  let currentOpening =
    Option.bind(voiceOpening, (value: Voiceover.readiness) =>
      value.scope == identity && value.total > 0 ? Some(value) : None
    );
  let voiceBlocked =
    voiceEnabled
    && !view.awaiting_new_game
    && spokenLines > 0
    && !
         Option.fold(
           ~none=false,
           ~some=(value: Voiceover.readiness) => value.ready,
           currentOpening,
         );
  let voicePreparation: option(Battlefield_window.preparation) =
    voiceBlocked
      ? Some({
          completed:
            Option.fold(
              ~none=0,
              ~some=(value: Voiceover.readiness) => value.completed,
              currentOpening,
            ),
          total: min(2, spokenLines),
          error:
            Option.bind(currentOpening, value => Js.Nullable.toOption(value.Voiceover.error)),
        })
      : None;
  React.useLayoutEffect1(
    () =>
      Option.bind(conversation, (campaign: Domain.campaign_state) =>
        !view.awaiting_new_game && Story.dialogue_pending(Some(campaign))
          ? Some(
              Voiceover.watch_opening(
                voice, identity, campaign.dialogue, int_of_float(campaign.dialogue_index), value =>
                setVoiceOpening(_ => Some(value))
              ),
            )
          : None
      ),
    [|view.game|],
  );
  let progress =
    Option.bind(battlefieldProgress, value =>
      value.Battlefield_view.identity == identity ? Some(value) : None
    );
  let ready =
    !voiceBlocked
    && Option.fold(
         ~none=false,
         ~some=value => value.Battlefield_view.phase == {js|ready|js},
         progress,
       );
  React.useEffect1(
    () => {
      if (!view.awaiting_new_game) {
        Session.scene_audio(session);
        Voiceover.prepare(voice, identity, view.catalog, view.game);
      };
      None;
    },
    [|view.game|],
  );
  let inspectionOpen = Hud_details.is_open(view.hud) && (ready || state.winner == None);
  let detailsContext: Hud_details.context = {
    catalog: view.catalog,
    artwork: view.artwork,
    game: view.interaction_game,
  };
  let (briefing, setBriefing) = React.useState(() => false);
  let battleContent = React.useRef(Js.Nullable.null);
  let (briefingScroll, setBriefingScroll) = React.useState(() => 0);
  let toggleBriefing = () => {
    let visible =
      briefing
      && Option.fold(
           ~none=false,
           ~some=
             overview => {
               let top =
                 Option.fold(
                   ~none=0.,
                   ~some=node => bounds(node).top,
                   Js.Nullable.toOption(battleContent.current),
                 );
               bounds(overview).bottom > top;
             },
           Js.Nullable.toOption(byId({js|mission-overview|js})),
         );
    setBriefing(_ => !visible);
    if (!visible) {
      setBriefingScroll(value => value + 1);
    };
  };
  React.useLayoutEffect1(
    () => {
      if (briefingScroll > 0 && briefing) {
        Option.iter(
          node => scrollTo(node, { "top": 0. }),
          Js.Nullable.toOption(battleContent.current),
        );
      };
      None;
    },
    [|briefingScroll|],
  );
  let (request, setRequest) = React.useState(() => None);
  let (setupBack, setSetupBack) = React.useState(() => false);
  let requestId = React.useRef(0);
  let (settingsOpen, setSettingsOpen) = React.useState(() => false);
  let (settingsKey, setSettingsKey) = React.useState(() => 0);
  let (settingsScreen, setSettingsScreen) = React.useState(() => Dialogs.Settings.Hub);
  let (repairKey, setRepairKey) = React.useState(() => 0);
  let settingsFocus = React.useRef(None);
  let openSettings = (~focusId=None, screen) => {
    settingsFocus.current = focusId;
    setSettingsScreen(_ => screen);
    setSettingsKey(value => value + 1);
    setSettingsOpen(_ => true);
    ();
  };
  let openRepair = () => {
    setSettingsOpen(_ => false);
    setRepairKey(value => value + 1);
  };
  React.useEffect1(
    () => {
      setBriefing(_ => false);
      None;
    },
    [|state.map_id|],
  );
  React.useLayoutEffect1(
    () => {
      Session.rendered(session, view.revision);
      None;
    },
    [|view|],
  );
  React.useEffect1(() => Some(onMount()), [|session|]);
  React.useEffect2(
    () =>
      if (view.failure != None) {
        None;
      } else {
        let key = event =>
          if (Session.view(session).failure == None
              && Js.Undefined.toOption(tagName(keyTarget(event))) != Some({js|CANVAS|js})
              && eventKey(event) == {js|Escape|js}
              && Js.Nullable.toOption(query({js|[data-ui-modal]|js})) == None) {
            actions.clear(true);
          };
        let away = event => {
          let node = target(event);
          if (Session.view(session).failure == None
              && nodeType(node) == 1
              && Js.Nullable.toOption(query({js|[data-ui-modal]|js})) == None
              && Js.Nullable.toOption(
                   closest(
                     node,
                     {js|.phaser-battlefield, aside, button, select, input, summary, a, [data-ui-modal]|js},
                   ),
                 )
              == None) {
            actions.clear(true);
          };
        };
        on_keydown(document, key);
        addDocumentListener(document, {js|click|js}, away);
        Some(
          () => {
            off_keydown(document, key);
            removeDocumentListener(document, {js|click|js}, away);
          },
        );
      },
    (actions, view.failure),
  );
  let openSetup = (~fromSettings=false, mode) => {
    setSetupBack(_ => fromSettings);
    setSettingsOpen(_ => false);
    requestId.current = requestId.current + 1;
    setRequest(_ => Some((requestId.current, mode)));
  };
  React.useEffect1(
    () => {
      if (view.awaiting_new_game) {
        openSetup(Setup.Campaign);
      };
      None;
    },
    [|view.awaiting_new_game|],
  );
  let openCampaign = () => openSetup(Setup.Campaign);
  let openMap = () => openSetup(Setup.Skirmish);
  view.failure != None
    ? <Shell failure={view.failure} />
    : <Dialogs.Settings
        voiceEnabled
        onVoiceChange=changeVoice
        allowNonHumanMaps
        onNonHumanMapsChange=changeNonHumanMaps
        onFollowAiChange=changeFollowAi
        playtestPreferenceError
        enemyDetailsError
        onEnemyDetailsChange=changeEnemyDetails
        showCommanderArrow
        commanderArrowError
        onCommanderArrowChange=changeCommanderArrow
        session
        view
        actions
        opened=settingsOpen
        onOpenChange={value => setSettingsOpen(_ => value)}
        onNewBattle={() => openSetup(~fromSettings=true, Setup.Campaign)}
        onRepair=openRepair
        openKey=settingsKey
        initialScreen=settingsScreen
        focusId={settingsFocus.current}>
        {!view.awaiting_new_game
           ? <main
               className={
                           let js_text_part_1 =
                             ready && state.winner == Some(Domain.Battle_result.Player_defeat)
                               ? {js| defeated|js} : {js||js};
                           {j|battle-layout$(js_text_part_1)|j};
                         }>
               <TurnBar
                 view
                 actions
                 ready
                 onSettings={() => openSettings(Dialogs.Settings.Hub)}
                 briefing
                 onBriefing=toggleBriefing
               />
               <p
                 id={js|status|js}
                 className={js|status|js}
                 role={js|status|js}
                 ariaLive={js|polite|js}>
                 {text(
                    !ready
                      ? {js|Loading battlefield…|js}
                      : Option.fold(
                          ~none=false,
                          ~some=(frame: Combat_playback.frame) => !frame.complete,
                          view.combat_playback,
                        )
                          ? {js|Resolving combat…|js} : view.status,
                  )}
               </p>
               <Audio_status retryHud={() => Session.prepare_audio(session)} renderer />
               <div className={js|battle-content|js} ref={ReactDOM.Ref.domRef(battleContent)}>
                 <Panels.Campaign
                   view
                   actions
                   showResult=ready
                   expanded=briefing
                   onCollapse={() => setBriefing(_ => false)}
                   onNewCampaign=openCampaign
                 />
                 <div
                   className={
                               let js_text_part_1 = inspectionOpen ? {js||js} : {js| map-only|js};
                               {j|workspace$(js_text_part_1)|j};
                             }>
                   <Battlefield
                     view
                     actions
                     renderer
                     previews
                     progress
                     onProgress={value => setBattlefieldProgress(_ => Some(value))}
                     onRendererFailed={message => Session.renderer_failed(session, message)}
                     prepareReload={source => Session.prepare_reload(session, source)}
                     showCommanderArrow
                     voicePreparation
                     onRetryVoice={() => Voiceover.retry(voice)}
                     onSkipVoice={() => changeVoice(false)}
                   />
                   {inspectionOpen
                      ? <aside ariaLabel={js|Unit and terrain details|js}>
                          <Panels.Unit
                            view=detailsContext
                            hud={view.hud}
                            actions
                            onNewBattle=openMap
                          />
                          <Panels.Terrain
                            view=detailsContext
                            panel={Hud_details.terrain(view.hud)}
                            actions
                            previews
                          />
                          <Panels.GroundItems
                            artwork={view.artwork}
                            items={Hud_details.ground_items(view.hud)}
                          />
                        </aside>
                      : React.null}
                 </div>
                 {ready
                    ? <Panels.Defeat
                        view
                        actions
                        onNewGame={state.campaign == None ? openMap : openCampaign}
                      />
                    : React.null}
               </div>
             </main>
           : React.null}
        <Save_failure view actions onManage={() => openSettings(Dialogs.Settings.Saves)} />
        <Import_repair
          session
          view
          onNewGame=openMap
          onManage={() => openSettings(Dialogs.Settings.Saves)}
          openKey=repairKey
        />
        {!view.awaiting_new_game && !voiceBlocked && (ready || state.winner == None)
           ? <Dialogs.Dialogue
               view
               actions
               voice
               scope=identity
               narrate={game => Session.narrate(session, game)}
             />
           : React.null}
        <New_game
          allowNonHumanMaps
          campaignTags
          scenarioTags
          onMapTagsChange=changeMapTags
          session
          view
          actions
          request
          onBack=?{
            setupBack
              ? Some(() => openSettings(~focusId=Some({js|new-game|js}), Dialogs.Settings.Hub))
              : None
          }
        />
      </Dialogs.Settings>;
};
