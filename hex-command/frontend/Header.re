/** Shared controls for startup and live battle; this module has no game imports. */
type stats = {
  gold: float,
  income: float,
  owned_villages: float,
  total_villages: float,
  round: float,
};

module Icon = {
  [@react.component]
  let make = (~name) =>
    <svg viewBox="0 0 24 24" ariaHidden=true className={name == "search" ? "ui-icon" : ""}>
      {name == "search" ? <circle cx="10" cy="10" r="6" /> : React.null}
      <path
        d={
          switch (name) {
          | "village" => "m3 11 9-8 9 8M5 10v11h14V10M10 21v-7h4v7"
          | "undo" => "M9 4 4 9l5 5M4 9h9a7 7 0 0 1 7 7v4"
          | "search" => "m15 15 6 6"
          | _ => "m9 3 1-1h4l1 3 3 1 3-1 2 4-2 2v3l2 2-2 4-3-1-3 1-1 3h-4l-1-3-3-1-3 1-2-4 2-2v-3L1 9l2-4 3 1 3-1z"
          }
        }
      />
      {name == "settings" ? <circle cx="12" cy="12" r="3" /> : React.null}
    </svg>;
};

[@react.component]
let make =
    (
      ~stats=None,
      ~turnLabel="End turn",
      ~canEndTurn=false,
      ~finished=false,
      ~canUndo=false,
      ~campaign=false,
      ~briefing=false,
      ~onEndTurn=() => (),
      ~onUndo=() => (),
      ~onSettings=None,
      ~onBriefing=None,
    ) => {
  let text = value => React.string(Utf8.to_js(value));
  let number = Header_number.compact;
  let value = project => Option.fold(~none="—", ~some=stats => number(project(stats)), stats);
  let full = project =>
    Option.map(stats => Utf8.to_js(Header_number.full(project(stats))), stats);
  let invoke = callback => Option.iter(callback => callback(), callback);
  let turnInfo =
    <>
      <span id="turn-label" title={Utf8.to_js(turnLabel)}> {text(turnLabel)} </span>
      <span
        id="round"
        className="round"
        title={Utf8.to_js(
          Option.fold(
            ~none="Round loading",
            ~some=stats => "Round " ++ Header_number.full(stats.round),
            stats,
          ),
        )}>
        <span className="round-caption"> {text("Turn")} </span>
        <b className="round-number"> {text(value(stats => stats.round))} </b>
      </span>
    </>;
  <header
    className={"battle-bar" ++ (campaign ? " campaign-controls" : "")} ariaLabel="Battle controls">
    <div className="header-economy">
      <span className="treasury" ariaLabel="Gold">
        <i className="coin" ariaHidden=true />
        <b id="gold" className="hud-number" title=?{full(stats => stats.gold)}>
          {text(value(stats => stats.gold))}
        </b>
      </span>
      <span className="income" title="Coins earned each turn">
        <b id="income" className="hud-number" title=?{full(stats => stats.income)}>
          {text(
             Option.fold(~none="—", ~some=stats => Header_number.signed(stats.income), stats),
           )}
        </b>
        <span className="income-caption"> {text("/turn")} </span>
      </span>
      <span className="villages" title="Owned villages">
        <Icon name="village" />
        <span className="village-count">
          <b id="villages" className="hud-number" title=?{full(stats => stats.owned_villages)}>
            {text(value(stats => stats.owned_villages))}
          </b>
          <span className="village-divider"> {text("/")} </span>
          <span
            id="village-total" className="hud-number" title=?{full(stats => stats.total_villages)}>
            {text(value(stats => stats.total_villages))}
          </span>
        </span>
      </span>
    </div>
    {finished
       ? <div className="turn-status turn-control" role="status"> turnInfo </div>
       : <button
           id="end-turn"
           className="primary turn-control"
           disabled={!canEndTurn}
           onClick={_ => onEndTurn()}>
           turnInfo
         </button>}
    <button
      id="undo-move"
      className="icon-button"
      ariaLabel="Undo move"
      title="Undo move"
      disabled={!canUndo}
      onPointerDown={Button_press.pointer_down(onUndo)}
      onClick={Button_press.click(onUndo)}>
      <Icon name="undo" />
    </button>
    <button
      id="mission-briefing-button"
      className="icon-button"
      ariaLabel="Mission briefing"
      title="Mission briefing"
      ariaExpanded=briefing
      ariaControls="mission-overview"
      disabled={!campaign || onBriefing == None}
      onPointerDown={Button_press.pointer_down(() => invoke(onBriefing))}
      onClick={Button_press.click(() => invoke(onBriefing))}>
      <Icon name="search" />
    </button>
    <button
      id="settings-button"
      className="icon-button"
      ariaLabel="Menu"
      title="Menu"
      disabled={onSettings == None}
      onPointerDown={Button_press.pointer_down(() => invoke(onSettings))}
      onClick={Button_press.click(() => invoke(onSettings))}>
      <Icon name="settings" />
    </button>
  </header>;
};
