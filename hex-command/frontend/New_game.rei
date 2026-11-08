/** Persistent setup choices with nested native modal lifetimes. Requests open
    the selected mode; commands own replacement, persistence and enemy pacing. */
[@react.component]
let make:
  (
    ~session: Session.t,
    ~view: Session.model,
    ~actions: Session.actions,
    ~request: option((int, Setup.mode)),
    ~onBack: unit => unit=?
  ) =>
  React.element;
