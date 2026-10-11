open Ui;

module TurnBar = {
  [@react.component]
  let make =
      (~view: Session.model, ~actions: Session.actions, ~onSettings, ~briefing, ~onBriefing) => {
    let state = Native.state(view.game);
    let income = Native.income(view.game, Domain.side_player);
    let villages =
      Array.fold_left(
        (count, tile: Domain.tile) => count + (tile.Domain.terrain == "village" ? 1 : 0),
        0,
        state.tiles,
      );
    let winner = Js.Nullable.toOption(state.winner);
    let canAct =
      state.turn == Domain.side_player
      && winner == None
      && !Story.dialogue_pending(state.campaign);
    <Header
      stats={
        Some({
          Header.gold: state.gold.player,
          income,
          owned_villages: income /. 2.,
          total_villages: float_of_int(villages),
          round: state.round,
        })
      }
      turnLabel={
        winner == Some(Domain.side_player)
          ? "Victory"
          : winner == Some(Domain.side_enemy)
              ? "Defeat" : state.turn == Domain.side_player ? "End turn" : "Enemy turn"
      }
      canEndTurn=canAct
      canUndo={view.can_undo}
      campaign={state.campaign != None}
      briefing
      onEndTurn={() => ignorePromise(actions.end_turn())}
      onUndo={actions.undo}
      onSettings={Some(onSettings)}
      onBriefing={Some(onBriefing)}
    />;
  };
};

/** The root is the sole session subscriber. Descendants receive one immutable
    publication and a stable command capability; effects acknowledge DOM commit. */
[@react.component]
let make = (~session: Session.t, ~onMount: (unit, unit) => unit) => {
  let subscribe =
    React.useCallback1(callback => Session.subscribe(session, _ => callback()), [|session|]);
  let getSnapshot = React.useCallback1(() => Session.view(session), [|session|]);
  let view = React.useSyncExternalStore(~subscribe, ~getSnapshot);
  let actions = Session.commands(session);
  let state = Native.state(view.game);
  let inspectionOpen =
    view.selected != None
    || view.inspected != None
    || Js.Nullable.toOption(state.winner) != None
    && state.campaign == None;
  let (briefing, setBriefing) = React.useState(() => false);
  let (campaignOpen, setCampaignOpen) = React.useState(() => false);
  let (mapOpen, setMapOpen) = React.useState(() => false);
  let (campaignChoice, setCampaign) = React.useState(() => "lantern-road");
  let (mapChoice, setMap) = React.useState(() => "mirror");
  let settings = React.useRef(Js.Nullable.null);
  let campaign = React.useRef(Js.Nullable.null);
  let reset = React.useRef(Js.Nullable.null);
  React.useEffect1(
    () => {
      setBriefing(_ => false);
      None;
    },
    [|state.mapId|],
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
                   closest(node, ".hex, aside, button, select, input, summary, a, dialog"),
                 )
              == None) {
            actions.clear(true);
          };
        };
        addListener(document, "keydown", key);
        addListener(document, "click", away);
        Some(
          () => {
            removeListener(document, "keydown", key);
            removeListener(document, "click", away);
          },
        );
      },
    (actions, view.failure),
  );
  React.useEffect2(
    () => {
      switch (Js.Nullable.toOption(campaign.current), view.failure) {
      | (Some(element), None) =>
        let closed = _ =>
          if (!isOpen(element)) {
            setCampaignOpen(_ => false);
          };
        addListener(element, "close", closed);
        Some(() => removeListener(element, "close", closed));
      | _ => None
      }
    },
    (campaign, view.failure),
  );
  React.useEffect2(
    () => {
      switch (Js.Nullable.toOption(reset.current), view.failure) {
      | (Some(element), None) =>
        let closed = _ =>
          if (!isOpen(element)) {
            setMapOpen(_ => false);
          };
        addListener(element, "close", closed);
        Some(() => removeListener(element, "close", closed));
      | _ => None
      }
    },
    (reset, view.failure),
  );
  React.useLayoutEffect1(
    () => {
      if (campaignOpen) {
        show(campaign);
      };
      None;
    },
    [|campaignOpen|],
  );
  let openCampaign = () => {
    close(settings);
    setCampaignOpen(_ => true);
  };
  React.useLayoutEffect1(
    () => {
      if (mapOpen) {
        show(reset);
      };
      None;
    },
    [|mapOpen|],
  );
  let openMap = () => {
    close(settings);
    setMap(_ =>
      Array.exists(
        (layout: Domain.map_definition) => layout.id == state.mapId,
        Catalog.battle_maps(view.catalog),
      )
        ? state.mapId : "mirror"
    );
    setMapOpen(_ => true);
  };
  view.failure != None
    ? <Shell failure={view.failure} />
    : <>
        <main className="battle-layout">
          <TurnBar
            view
            actions
            onSettings={() => show(settings)}
            briefing
            onBriefing={() => setBriefing(value => !value)}
          />
          <p id="status" className="status" role="status" ariaLive="polite">
            {text(view.status)}
          </p>
          <div className="battle-content">
            <Panels.Campaign
              view
              actions
              expanded=briefing
              onCollapse={() => setBriefing(_ => false)}
              onNewCampaign=openCampaign
            />
            <div className={"workspace" ++ (inspectionOpen ? "" : " map-only")}>
              <Battlefield view actions />
              {inspectionOpen
                 ? <aside ariaLabel="Unit and terrain details">
                     <Panels.Unit view actions onNewBattle=openMap />
                     <Panels.Terrain view actions />
                   </aside>
                 : React.null}
            </div>
          </div>
        </main>
        <Dialogs.Settings
          view
          actions
          dialog=settings
          onNewBattle=openMap
          onNewCampaign=openCampaign
        />
        <Dialogs.Dialogue view actions />
        <dialog className="campaign-dialog" ref={ReactDOM.Ref.domRef(campaign)}>
          <h2> {text("New campaign")} </h2>
          <p> {text("Choose a company and a story. Starting replaces your current battle.")} </p>
          {campaignOpen
             ? <Dialogs.CampaignPicker
                 view
                 selected=campaignChoice
                 onSelect={id => setCampaign(_ => id)}
               />
             : React.null}
          <div className="dialog-actions">
            <button
              className="quiet"
              onClick={_ => {
                setCampaignOpen(_ => false);
                close(campaign);
              }}>
              {text("Keep playing")}
            </button>
            <button
              className="primary"
              onClick={_ => {
                setBriefing(_ => false);
                actions.start_campaign(campaignChoice);
                setCampaignOpen(_ => false);
                close(campaign);
              }}>
              {text("Start campaign")}
            </button>
          </div>
        </dialog>
        <dialog id="reset-dialog" ref={ReactDOM.Ref.domRef(reset)}>
          <h2> {text("Start a new battle?")} </h2>
          <p> {text("Choose your battlefield. Your current battle will be replaced.")} </p>
          {mapOpen
             ? <Dialogs.MapPicker
                 catalog={view.catalog}
                 selected=mapChoice
                 onSelect={id => setMap(_ => id)}
               />
             : React.null}
          <div className="dialog-actions">
            <button id="keep-playing" className="quiet" onClick={_ => close(reset)}>
              {text("Keep playing")}
            </button>
            <button
              id="confirm-reset"
              className="primary"
              onClick={_ => {
                setBriefing(_ => false);
                close(reset);
                actions.reset(mapChoice);
              }}>
              {text("New battle")}
            </button>
          </div>
        </dialog>
      </>;
};
