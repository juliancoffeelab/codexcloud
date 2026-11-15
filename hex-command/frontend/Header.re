[@ocaml.doc
  {js| Shared controls for startup and live battle; this module has no game imports. |js}
]

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
    <svg
      viewBox={js|0 0 24 24|js}
      ariaHidden=true
      className={name == {js|search|js} ? {js|ui-icon|js} : {js||js}}>
      {name == {js|search|js} ? <circle cx={js|10|js} cy={js|10|js} r={js|6|js} /> : React.null}
      <path
        d={
          switch (name) {
          | matched_text_0 when matched_text_0 == {js|village|js} => {js|m3 11 9-8 9 8M5 10v11h14V10M10 21v-7h4v7|js}
          | matched_text_1 when matched_text_1 == {js|undo|js} => {js|M9 4 4 9l5 5M4 9h9a7 7 0 0 1 7 7v4|js}
          | matched_text_2 when matched_text_2 == {js|search|js} => {js|m15 15 6 6|js}
          | _ => {js|m9 3 1-1h4l1 3 3 1 3-1 2 4-2 2v3l2 2-2 4-3-1-3 1-1 3h-4l-1-3-3-1-3 1-2-4 2-2v-3L1 9l2-4 3 1 3-1z|js}
          }
        }
      />
      {name == {js|settings|js} ? <circle cx={js|12|js} cy={js|12|js} r={js|3|js} /> : React.null}
    </svg>;
};
[@react.component]
let make =
    (
      ~stats=None,
      ~turnLabel={js|End turn|js},
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
  let text = value => React.string(value);
  let number = Header_number.compact;
  let value = project =>
    Option.fold(~none={js|—|js}, ~some=stats => number(project(stats)), stats);
  let full = project => Option.map(stats => Header_number.full(project(stats)), stats);
  let invoke = callback => Option.iter(callback => callback(), callback);
  let turnInfo =
    <>
      <span id={js|turn-label|js} title=turnLabel> {text(turnLabel)} </span>
      <span
        id={js|round|js}
        className={js|round|js}
        title={Option.fold(
          ~none={js|Round loading|js},
          ~some=
            stats => {
              let js_text_part_1 = Header_number.full(stats.round);
              {j|Round $(js_text_part_1)|j};
            },
          stats,
        )}>
        <span className={js|round-caption|js}> {text({js|Turn|js})} </span>
        <b className={js|round-number|js}> {text(value(stats => stats.round))} </b>
      </span>
    </>;
  /* Press_button owns immediate/keyboard activation. Prevent Trigger's click
     toggle from undoing the controlled opening; Radix still restores focus. */
  let menuButton =
    <button
      onClick={event => React.Event.Mouse.preventDefault(event)}
      id={js|settings-button|js}
      className={js|icon-button|js}
      ariaLabel={js|Menu|js}
      title={js|Menu|js}
      disabled={onSettings == None}>
      <Icon name={js|settings|js} />
    </button>;
  <header
    className={
                let js_text_part_1 = campaign ? {js| campaign-controls|js} : {js||js};
                {j|battle-bar$(js_text_part_1)|j};
              }
    ariaLabel={js|Battle controls|js}>
    <div className={js|header-economy|js}>
      <span className={js|treasury|js} ariaLabel={js|Gold|js}>
        <i className={js|coin|js} ariaHidden=true />
        <b id={js|gold|js} className={js|hud-number|js} title=?{full(stats => stats.gold)}>
          {text(value(stats => stats.gold))}
        </b>
      </span>
      <span className={js|income|js} title={js|Coins earned each turn|js}>
        <b id={js|income|js} className={js|hud-number|js} title=?{full(stats => stats.income)}>
          {text(
             Option.fold(
               ~none={js|—|js},
               ~some=stats => Header_number.signed(stats.income),
               stats,
             ),
           )}
        </b>
        <span className={js|income-caption|js}> {text({js|/turn|js})} </span>
      </span>
      <span className={js|villages|js} title={js|Owned villages|js}>
        <Icon name={js|village|js} />
        <span className={js|village-count|js}>
          <b
            id={js|villages|js}
            className={js|hud-number|js}
            title=?{full(stats => stats.owned_villages)}>
            {text(value(stats => stats.owned_villages))}
          </b>
          <span className={js|village-divider|js}> {text({js|/|js})} </span>
          <span
            id={js|village-total|js}
            className={js|hud-number|js}
            title=?{full(stats => stats.total_villages)}>
            {text(value(stats => stats.total_villages))}
          </span>
        </span>
      </span>
    </div>
    {finished
       ? <div className={js|turn-status turn-control|js} role={js|status|js}> turnInfo </div>
       : <button
           id={js|end-turn|js}
           className={js|primary turn-control|js}
           disabled={!canEndTurn}
           onClick={_ => onEndTurn()}>
           turnInfo
         </button>}
    <button
      id={js|undo-move|js}
      className={js|icon-button|js}
      ariaLabel={js|Undo move|js}
      title={js|Undo move|js}
      disabled={!canUndo}
      onClick={_ => onUndo()}>
      <Icon name={js|undo|js} />
    </button>
    <button
      id={js|mission-briefing-button|js}
      className={js|icon-button|js}
      ariaLabel={js|Mission briefing|js}
      title={js|Mission briefing|js}
      ariaExpanded=briefing
      ariaControls={js|mission-overview|js}
      disabled={!campaign || onBriefing == None}
      onClick={_ => invoke(onBriefing)}>
      <Icon name={js|search|js} />
    </button>
    {onSettings == None
       ? menuButton
       : <Press_button onActivate={() => invoke(onSettings)}>
           <Modal.Trigger asChild=true> menuButton </Modal.Trigger>
         </Press_button>}
  </header>;
};
