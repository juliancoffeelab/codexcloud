/** Selection panels project only immutable native rules and published artwork. */
module Unit: {
  [@react.component]
  let make:
    (~view: Session.model, ~actions: Session.actions, ~onNewBattle: unit => unit) => React.element;
};
module Terrain: {
  [@react.component]
  let make: (~view: Session.model, ~actions: Session.actions) => React.element;
};
/** Authored briefings, objectives, debriefs and bounded survivor recall. */
module Campaign: {
  [@react.component]
  let make:
    (
      ~view: Session.model,
      ~actions: Session.actions,
      ~expanded: bool,
      ~onCollapse: unit => unit,
      ~onNewCampaign: unit => unit
    ) =>
    React.element;
};
