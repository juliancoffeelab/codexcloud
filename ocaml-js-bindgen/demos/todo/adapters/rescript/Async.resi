/** Observe fulfillment or rejection. The rejection callback receives the host error converted to text. This does not cancel the operation. */
let observe_error: (promise<'a>, 'a => unit, string => unit) => unit
