/** Install the application UI. The supplied loader invokes its success or failure callback when loading finishes. */
let start: ((result<Bindings.Paint.t, Bindings.Image_error.t> => unit, unit => unit) => unit) => unit
