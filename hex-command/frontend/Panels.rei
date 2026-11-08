/** Selection panels project only detached native rules and published artwork. */
module Unit: {
  [@react.component]
  let make:
    (
      ~view: Hud_details.context,
      ~hud: Hud_details.t,
      ~actions: Session.actions,
      ~onNewBattle: unit => unit
    ) =>
    React.element;
};
module Terrain: {
  [@react.component]
  let make:
    (
      ~view: Hud_details.context,
      ~panel: option(Presentation.terrain_panel),
      ~actions: Session.actions,
      ~previews: Terrain_preview.t
    ) =>
    React.element;
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

module GroundItems: {
  [@react.component]
  let make: (~artwork: Art.t, ~items: array(Domain.item)) => React.element;
};
