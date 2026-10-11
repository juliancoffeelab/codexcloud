/** Subscribe to a session and render an accessible battle. [onMount] owns the
    embedding resources and returns the cleanup run at unmount. */
[@react.component]
let make: (~session: Session.t, ~onMount: (unit, unit) => unit) => React.element;
