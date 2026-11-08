/** Reports descendant render/effect failures before the owner releases them.
    The factory creates a fresh element on each healthy boundary render; props
    retain no previously rendered descendant tree. */
[@react.component]
let make: (~onError: Foreign_error.t => unit, ~render: unit => React.element) => React.element;
