/** Controlled Radix modal lifetimes and immutable authored selection menus. */
module Settings: {
  type screen =
    | Hub
    | Saves;
  [@react.component]
  let make:
    (
      ~session: Session.t,
      ~allowNonHumanMaps: bool,
      ~onNonHumanMapsChange: bool => unit,
      ~onFollowAiChange: bool => unit,
      ~playtestPreferenceError: string,
      ~view: Session.model,
      ~voiceEnabled: bool,
      ~onVoiceChange: bool => unit,
      ~actions: Session.actions,
      ~opened: bool,
      ~onOpenChange: bool => unit,
      ~onNewBattle: unit => unit,
      ~onRepair: unit => unit,
      ~openKey: int=?,
      ~initialScreen: screen=?,
      ~focusId: option(string)=?,
      ~showCommanderArrow: bool,
      ~commanderArrowError: string,
      ~onCommanderArrowChange: bool => unit,
      ~enemyDetailsError: string,
      ~onEnemyDetailsChange: bool => unit,
      ~children: React.element
    ) =>
    React.element;
};
/** Keyboard navigation and post-commit focus belong to the current publication. */
module Dialogue: {
  [@react.component]
  let make:
    (
      ~view: Session.model,
      ~actions: Session.actions,
      ~narrate: Native.view => unit,
      ~voice: Voiceover.t,
      ~scope: string
    ) =>
    React.element;
};
