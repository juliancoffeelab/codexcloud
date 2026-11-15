/** Controlled browse/setup flow. Map filters are browser preferences; other choices
    persist while mounted. Game commands own replacement, saving and enemy pacing. */
[@react.component]
let make:
  (
    ~session: Session.t,
    ~allowNonHumanMaps: bool,
    ~campaignTags: list(string),
    ~scenarioTags: list(string),
    ~onMapTagsChange: (Setup.mode, list(string)) => unit,
    ~view: Session.model,
    ~actions: Session.actions,
    ~request: option((int, Setup.mode)),
    ~onBack: unit => unit=?
  ) =>
  React.element;
