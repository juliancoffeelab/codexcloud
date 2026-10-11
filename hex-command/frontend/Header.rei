/** Catalog-free shared controls. Missing statistics render loading placeholders;
    settings and briefing are disabled when their capabilities are absent. */
type stats = {
  gold: float,
  income: float,
  owned_villages: float,
  total_villages: float,
  round: float,
};
[@react.component]
let make:
  (
    ~stats: option(stats)=?,
    ~turnLabel: string=?,
    ~canEndTurn: bool=?,
    ~canUndo: bool=?,
    ~campaign: bool=?,
    ~briefing: bool=?,
    ~onEndTurn: unit => unit=?,
    ~onUndo: unit => unit=?,
    ~onSettings: option(unit => unit)=?,
    ~onBriefing: option(unit => unit)=?
  ) =>
  React.element;
