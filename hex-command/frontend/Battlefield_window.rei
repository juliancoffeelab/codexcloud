/** Shared startup/live frame; no renderer or game dependencies. */
type phase =
  | Loading
  | Preparing
  | Ready;
[@react.component]
let make:
  (
    ~host: React.ref(Js.Nullable.t(Dom.element))=?,
    ~phase: phase=?,
    ~loaded: int=?,
    ~total: int=?,
    ~navigation: React.element=?,
    ~children: React.element=?
  ) =>
  React.element;
