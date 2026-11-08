@send
external then_callbacks: (promise<'a>, 'a => unit, Js.Promise.error => unit) => promise<unit> =
  "then"

let observe_error = (promise, success, failure) =>
  ignore(then_callbacks(promise, success, error => failure(Browser.error_text(error))))
