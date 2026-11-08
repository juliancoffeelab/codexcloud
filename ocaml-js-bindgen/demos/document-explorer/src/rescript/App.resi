/** Install the application UI. The supplied loader invokes its success or failure callback when loading finishes. */
let start: (
  (
    Bindings.Blob.t,
    result<Bindings.Document.t, Bindings.Document_error.t> => unit,
    unit => unit,
  ) => unit
) => unit
