open Ui;

module TurnBar = {
  [@react.component]
  let make =
      (~view: Session.model, ~actions: Session.actions, ~onSettings, ~briefing, ~onBriefing) => {
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
    let winner = state.winner;
    let canAct =
      state.turn == Domain.Side.Player
      && winner == None
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
        winner == Some(Domain.Side.Player)
          ? "Victory"
          : winner == Some(Domain.Side.Enemy)
              ? "Defeat" : state.turn == Domain.Side.Player ? "End turn" : "Enemy turn"
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

/** The root is the sole session subscriber. Descendants receive one detached
    publication and a stable command capability; effects acknowledge DOM commit. */
[@react.component]
let make = (~session: Session.t, ~renderer: Phaser_boot.t, ~onMount: (unit, unit) => unit) => {
  let subscribe =
    React.useCallback1(callback => Session.subscribe(session, _ => callback()), [|session|]);
  let getSnapshot = React.useCallback1(() => Session.view(session), [|session|]);
  let view = React.useSyncExternalStore(~subscribe, ~getSnapshot);
  let previews = React.useMemo0(() => Terrain_preview.create());
  React.useEffect1(() => {Some(() => Terrain_preview.close(previews))}, [|view.artwork|]);
  let actions = Session.commands(session);
  let (showCommanderArrow, setShowCommanderArrow) = React.useState(() => false);
  let (commanderArrowError, setCommanderArrowError) = React.useState(() => "");
  let changeCommanderArrow = enabled => {
    setShowCommanderArrow(_ => enabled);
    setCommanderArrowError(_ => "");
    try(Browser_storage.storage_set("hex-command-commander-arrow", string_of_bool(enabled))) {
    | error => setCommanderArrowError(_ => Foreign_error.exception_(error))
    };
  };
  let (enemyDetailsError, setEnemyDetailsError) = React.useState(() => "");
  let changeEnemyDetails = enabled => {
    Session.set_enemy_combat_details(session, enabled);
    setEnemyDetailsError(_ => "");
    try(Browser_storage.storage_set("hex-command-enemy-combat-details", string_of_bool(enabled))) {
    | error => setEnemyDetailsError(_ => Foreign_error.exception_(error))
    };
  };
  React.useEffect0(() => {
    let _ = Session.prepare_audio(session);
    try(
      setShowCommanderArrow(_ =>
        Js.Nullable.toOption(Browser_storage.storage_get("hex-command-commander-arrow"))
        == Some("true")
      )
    ) {
    | error => setCommanderArrowError(_ => Foreign_error.exception_(error))
    };
    try(
      Session.set_enemy_combat_details(
        session,
        Js.Nullable.toOption(Browser_storage.storage_get("hex-command-enemy-combat-details"))
        == Some("true"),
      )
    ) {
    | error => setEnemyDetailsError(_ => Foreign_error.exception_(error))
    };
    None;
  });
  let state = Native.state(view.game);
  React.useEffect1(
    () => {
      if (!view.awaiting_new_game) {
        Session.scene_audio(session);
      };
      None;
    },
    [|view.game|],
  );
  let inspectionOpen = Hud_details.is_open(view.hud);
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
           Js.Nullable.toOption(byId("mission-overview")),
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
  let settings = React.useRef(Js.Nullable.null);
  let (settingsKey, setSettingsKey) = React.useState(() => 0);
  let (settingsScreen, setSettingsScreen) = React.useState(() => Dialogs.Settings.Hub);
  let (repairKey, setRepairKey) = React.useState(() => 0);
  let settingsFocus = React.useRef(None);
  let openSettings = (~focusId=None, screen) => {
    settingsFocus.current = focusId;
    setSettingsScreen(_ => screen);
    setSettingsKey(value => value + 1);
    show(settings);
    let _ = Session.menu_cue(session);
    ();
  };
  let openRepair = () => {
    close(settings);
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
              && Js.Undefined.toOption(tagName(keyTarget(event))) != Some("CANVAS")
              && eventKey(event) == "Escape"
              && Js.Nullable.toOption(query("dialog[open]")) == None) {
            actions.clear(true);
          };
        let away = event => {
          let node = target(event);
          if (Session.view(session).failure == None
              && nodeType(node) == 1
              && Js.Nullable.toOption(query("dialog[open]")) == None
              && Js.Nullable.toOption(
                   closest(
                     node,
                     ".phaser-battlefield, aside, button, select, input, summary, a, dialog",
                   ),
                 )
              == None) {
            actions.clear(true);
          };
        };
        on_keydown(document, key);
        addDocumentListener(document, "click", away);
        Some(
          () => {
            off_keydown(document, key);
            removeDocumentListener(document, "click", away);
          },
        );
      },
    (actions, view.failure),
  );
  let openSetup = (~fromSettings=false, mode) => {
    setSetupBack(_ => fromSettings);
    close(settings);
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
    : <>
        {!view.awaiting_new_game
           ? <main
               className={
                 "battle-layout" ++ (state.winner == Some(Domain.Side.Enemy) ? " defeated" : "")
               }>
               <TurnBar
                 view
                 actions
                 onSettings={() => openSettings(Dialogs.Settings.Hub)}
                 briefing
                 onBriefing=toggleBriefing
               />
               <p id="status" className="status" role="status" ariaLive="polite">
                 {text(
                    Option.fold(
                      ~none=false,
                      ~some=(frame: Combat_playback.frame) => !frame.complete,
                      view.combat_playback,
                    )
                      ? "Resolving combat…" : view.status,
                  )}
               </p>
               <Audio_status retryHud={() => Session.prepare_audio(session)} renderer />
               <div className="battle-content" ref={ReactDOM.Ref.domRef(battleContent)}>
                 <Panels.Campaign
                   view
                   actions
                   expanded=briefing
                   onCollapse={() => setBriefing(_ => false)}
                   onNewCampaign=openCampaign
                 />
                 <div className={"workspace" ++ (inspectionOpen ? "" : " map-only")}>
                   <Battlefield
                     view
                     actions
                     renderer
                     previews
                     onRendererFailed={message => Session.renderer_failed(session, message)}
                     prepareReload={source => Session.prepare_reload(session, source)}
                     showCommanderArrow
                   />
                   {inspectionOpen
                      ? {
                        <aside ariaLabel="Unit and terrain details">
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
                        </aside>;
                      }
                      : React.null}
                 </div>
                 <Panels.Defeat
                   view
                   actions
                   onNewGame={state.campaign == None ? openMap : openCampaign}
                 />
               </div>
             </main>
           : React.null}
        <Dialogs.Settings
          enemyDetailsError
          onEnemyDetailsChange=changeEnemyDetails
          showCommanderArrow
          commanderArrowError
          onCommanderArrowChange=changeCommanderArrow
          session
          view
          actions
          dialog=settings
          onNewBattle={() => openSetup(~fromSettings=true, Setup.Campaign)}
          onRepair=openRepair
          openKey=settingsKey
          initialScreen=settingsScreen
          focusId={settingsFocus.current}
        />
        <Save_failure view actions onManage={() => openSettings(Dialogs.Settings.Saves)} />
        <Import_repair
          session
          view
          onNewGame=openMap
          onManage={() => openSettings(Dialogs.Settings.Saves)}
          openKey=repairKey
        />
        {!view.awaiting_new_game
           ? <Dialogs.Dialogue view actions narrate={game => Session.narrate(session, game)} />
           : React.null}
        <New_game
          session
          view
          actions
          request
          onBack=?{
            setupBack
              ? Some(() => {openSettings(~focusId=Some("new-game"), Dialogs.Settings.Hub)})
              : None
          }
        />
      </>;
};
