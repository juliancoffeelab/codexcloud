/** Shared startup/live frame; no renderer or game dependencies. */
type phase =
  | Loading
  | Preparing
  | Ready;
type preparation = {
  completed: int,
  total: int,
  error: option(string),
};
[@react.component]
let make:
  (
    ~host: React.ref(Js.Nullable.t(Dom.element))=?,
    ~phase: phase=?,
    ~preparation: option(preparation)=?,
    ~onRetryPreparation: unit => unit=?,
    ~onSkipPreparation: unit => unit=?,
    ~navigation: React.element=?,
    ~children: React.element=?
  ) =>
  React.element;
