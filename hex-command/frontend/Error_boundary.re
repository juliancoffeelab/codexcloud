[@mel.module "preact/hooks"]
external useErrorBoundary:
  (Foreign_error.t => unit) => (Js.Undefined.t(Foreign_error.t), unit => unit) =
  "useErrorBoundary";

[@react.component]
let make = (~onError, ~render) => {
  let (error, _) = useErrorBoundary(onError);
  switch (Js.Undefined.toOption(error)) {
  | None => render()
  | Some(_) => React.null
  };
};
