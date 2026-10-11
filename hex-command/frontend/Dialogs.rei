/** Native modal lifetimes and immutable authored selection menus. */
module Settings: {
  [@react.component]
  let make:
    (
      ~view: Session.model,
      ~actions: Session.actions,
      ~dialog: React.ref(Js.Nullable.t(Dom.element)),
      ~onNewBattle: unit => unit,
      ~onNewCampaign: unit => unit
    ) =>
    React.element;
};
module MapPicker: {
  [@react.component]
  let make: (~catalog: Catalog.t, ~selected: string, ~onSelect: string => unit) => React.element;
};
module CampaignPicker: {
  [@react.component]
  let make: (~view: Session.model, ~selected: string, ~onSelect: string => unit) => React.element;
};
/** Keyboard navigation and post-commit focus belong to the current publication. */
module Dialogue: {
  [@react.component]
  let make: (~view: Session.model, ~actions: Session.actions) => React.element;
};
