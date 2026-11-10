type phase = Loading | Failed of string | Ready

type t = {
  show : phase -> unit;
  mutable closed : bool;
  mutable cancel : (unit -> unit) option;
  mutable release : (unit -> unit) option;
}

external console_error : exn -> unit = "error" [@@mel.scope "console"]

let reason = Foreign_error.describe

let exception_reason error =
  match Js.Exn.asJsExn error with
  | Some error ->
      Utf8.of_js (Option.value ~default:"The game could not start." (Js.Exn.message error))
  | None -> (
      match error with
      | Failure message | Invalid_argument message -> message
      | _ -> "The game could not start.")

let cleanup action = try action () with error -> console_error error

let close t =
  if not t.closed then (
    t.closed <- true;
    let cancel = t.cancel and release = t.release in
    t.cancel <- None;
    t.release <- None;
    Option.iter cleanup cancel;
    Option.iter cleanup release)

let fail t message =
  if not t.closed then (
    Boot_process.fail message;
    close t;
    t.show (Failed message))

let create ~show ~schedule ~load =
  let t = { show; closed = false; cancel = None; release = None } in
  let current () = not t.closed in
  let failed message = fail t message in
  show Loading;
  let start () =
    if current () then (
      t.cancel <- None;
      try
        let pending = load current in
        let _ =
          Js.Promise.catch
            (fun error ->
              failed (reason error);
              Js.Promise.resolve ())
            (Js.Promise.then_
               (function
                 | Error error ->
                     failed error;
                     Js.Promise.resolve ()
                 | Ok release ->
                     if current () then (
                       t.release <- Some release;
                       show Ready)
                     else cleanup release;
                     Js.Promise.resolve ())
               pending)
        in
        ()
      with error -> failed (exception_reason error))
  in
  let cancel = schedule start in
  if t.closed then cleanup cancel else t.cancel <- Some cancel;
  t
