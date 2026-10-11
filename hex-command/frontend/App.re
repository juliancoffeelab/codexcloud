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
    <header
      className={"battle-bar " ++ (state.campaign != None ? "campaign-controls" : "")}
      ariaLabel="Battle controls">
      <div className="header-economy">
        <span className="treasury" ariaLabel="Gold">
          <i className="coin" ariaHidden=true />
          <b id="gold"> {num(state.gold.player)} </b>
        </span>
        <span className="income" title="Coins earned each turn">
          <b id="income"> {text("+" ++ number(income))} </b>
          <span> {text("/turn")} </span>
        </span>
        <span className="villages" title="Owned villages">
          <Icon name="village" />
          <span>
            <b id="villages"> {text(number(income /. 2.))} </b>
            {text("/")}
            <span id="village-total"> {text(string_of_int(villages))} </span>
          </span>
        </span>
      </div>
      <button
        id="end-turn"
        className="primary"
        disabled={!canAct}
        onClick={_ => ignorePromise(actions.end_turn())}>
        <span id="turn-label">
          {text(
             winner == Some(Domain.side_player)
               ? "Victory"
               : winner == Some(Domain.side_enemy)
                   ? "Defeat" : state.turn == Domain.side_player ? "End turn" : "Enemy turn",
           )}
        </span>
        <span id="round" className="round" title={"Round " ++ number(state.round)}>
          {text("Turn " ++ number(state.round))}
        </span>
      </button>
      <button
        id="undo-move"
        className="icon-button"
        ariaLabel="Undo move"
        title="Undo move"
        disabled={!view.can_undo}
        onClick={_ => actions.undo()}>
        <Icon name="undo" />
      </button>
      {state.campaign != None
         ? <button
             id="mission-briefing-button"
             className="icon-button"
             ariaLabel="Mission briefing"
             title="Mission briefing"
             ariaExpanded=briefing
             ariaControls="mission-overview"
             onClick={_ => onBriefing()}>
             <Icon name="search" />
           </button>
         : React.null}
      <button
        id="settings-button"
        className="icon-button"
        ariaLabel="Settings"
        title="Settings"
        onClick={_ => onSettings()}>
        <Icon name="settings" />
      </button>
    </header>;
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
  let (briefing, setBriefing) = React.useState(() => false);
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
      setAttribute(
        documentElement(document),
        "data-art-style",
        Art.style_name(Art.style(view.artwork)),
      );
      Session.rendered(session, view.revision);
      None;
    },
    [|view|],
  );
  React.useEffect1(() => Some(onMount()), [|session|]);
  React.useEffect1(
    () => {
      let key = event =>
        if (eventKey(event) == "Escape" && Js.Nullable.toOption(query("dialog[open]")) == None) {
          actions.clear(true);
        };
      addListener(document, "keydown", key);
      Some(() => removeListener(document, "keydown", key));
    },
    [|actions|],
  );
  let openCampaign = () => {
    close(settings);
    show(campaign);
  };
  let openMap = () => {
    close(settings);
    setMap(_ =>
      Array.exists(
        (layout: Domain.map_definition) => layout.id == state.mapId,
        Catalog.scenario_maps(view.catalog),
      )
        ? state.mapId : "mirror"
    );
    show(reset);
  };
  <>
    <main>
      <TurnBar
        view
        actions
        onSettings={() => show(settings)}
        briefing
        onBriefing={() => setBriefing(value => !value)}
      />
      <p id="status" className="status" role="status" ariaLive="polite"> {text(view.status)} </p>
      <Panels.Campaign
        view
        actions
        expanded=briefing
        onCollapse={() => setBriefing(_ => false)}
        onNewCampaign=openCampaign
      />
      <div className="workspace">
        <Battlefield view actions />
        <aside>
          <Panels.Unit view actions onNewBattle=openMap />
          <Panels.Terrain view actions />
        </aside>
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
      <Dialogs.CampaignPicker view selected=campaignChoice onSelect={id => setCampaign(_ => id)} />
      <div className="dialog-actions">
        <button className="quiet" onClick={_ => close(campaign)}>
          {text("Keep playing")}
        </button>
        <button
          className="primary"
          onClick={_ => {
            setBriefing(_ => false);
            actions.start_campaign(campaignChoice);
            close(campaign);
          }}>
          {text("Start campaign")}
        </button>
      </div>
    </dialog>
    <dialog id="reset-dialog" ref={ReactDOM.Ref.domRef(reset)}>
      <h2> {text("Start a new battle?")} </h2>
      <p> {text("Choose your battlefield. Your current battle will be replaced.")} </p>
      <Dialogs.MapPicker
        catalog={view.catalog}
        selected=mapChoice
        onSelect={id => setMap(_ => id)}
      />
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
