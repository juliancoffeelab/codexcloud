/** Native browse/setup flow. Choices persist while the component is mounted;
    game commands own replacement, save completion and enemy pacing. */
[@react.component]
let make:
  (
    ~session: Session.t,
    ~allowNonHumanMaps: bool,
    ~view: Session.model,
    ~actions: Session.actions,
    ~request: option((int, Setup.mode)),
    ~onBack: unit => unit=?
  ) =>
  React.element;
