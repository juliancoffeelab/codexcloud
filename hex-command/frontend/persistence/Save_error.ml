type kind = Full | Blocked | Internal
type t = { kind : kind; detail : string; cause : Foreign_error.cause option }

exception Failed of t

let internal detail = { kind = Internal; detail; cause = None }
let blocked detail = { kind = Blocked; detail; cause = None }

let kind_name = function
  | Full -> {js|full|js}
  | Blocked -> {js|blocked|js}
  | Internal -> {js|internal|js}

let of_error value =
  match Js.Exn.anyToExnInternal value with
  | Failed failure -> failure
  | exception_ ->
      let browser = Js.Exn.asJsExn exception_ in
      let cause = Foreign_error.Exception exception_ in
      let kind =
        try
          match Option.bind browser Js.Exn.name with
          | Some matched_text_0
            when matched_text_0 = {js|QuotaExceededError|js}
                 || matched_text_0 = {js|NS_ERROR_DOM_QUOTA_REACHED|js} ->
              Full
          | Some matched_text_1
            when matched_text_1 = {js|SecurityError|js} || matched_text_1 = {js|NotAllowedError|js}
            ->
              Blocked
          | _ -> Internal
        with _ -> Internal
      in
      let detail =
        match exception_ with
        | Failure message | Invalid_argument message | Sys_error message -> message
        | _ -> Foreign_error.exception_ exception_
      in
      { kind; detail; cause = Some cause }

let detail value = (of_error value).detail
let exception_ failure = Failed failure
let reject failure = Js.Promise.reject (exception_ failure)
let reject_internal detail = reject (internal detail)
let reject_error error = reject (of_error error)
let attempt work = try work () with error -> reject_error error
let catch promise = Js.Promise.catch reject_error promise
let raise_internal detail = raise (Failed (internal detail))
let promise = function Ok value -> Js.Promise.resolve value | Error failure -> reject failure
let await promise_ = Js.Promise.then_ promise promise_
