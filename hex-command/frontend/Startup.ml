type phase = Loading | Failed of string | Ready
type t = {
  mutable closed : bool;
  mutable cancel : (unit -> unit) option;
  mutable release : (unit -> unit) option;
}
external console_error : exn -> unit = "error" [@@mel.scope "console"]
external rejection_json : Js.Promise.error -> Js.Json.t = "%identity"
let reason error =
  let value = rejection_json error in
  let property = Option.bind (Js.Json.decodeObject value) (fun object_ ->
      Option.bind (Js.Dict.get object_ "message") Js.Json.decodeString) in
  Option.value ~default:(Option.value ~default:"The game could not start." (Js.Json.stringifyAny value)) property
let exception_reason error = match Js.Exn.asJsExn error with
  | Some error -> Option.value ~default:"The game could not start." (Js.Exn.message error)
  | None -> match error with Failure message | Invalid_argument message -> message | _ -> "The game could not start."
let cleanup action = try action () with error -> console_error error
let close t =
  if not t.closed then (
    t.closed <- true;
    let cancel = t.cancel and release = t.release in
    t.cancel <- None; t.release <- None;
    Option.iter cleanup cancel; Option.iter cleanup release)
let create ~show ~schedule ~load =
  let t = {closed=false; cancel=None; release=None} in
  let current () = not t.closed in
  let failed message =
    if current () then (
      let release = t.release in t.release <- None;
      Option.iter cleanup release;
      if current () then show (Failed message)) in
  show Loading;
  let start () =
    if current () then (
      t.cancel <- None;
      try
        let pending = load current in
        ignore (Js.Promise.catch (fun error ->
            failed (reason error); Js.Promise.resolve ())
            (Js.Promise.then_ (fun release ->
                 if current () then (t.release <- Some release; show Ready)
                 else cleanup release;
                 Js.Promise.resolve ()) pending))
      with error -> failed (exception_reason error)) in
  let cancel = schedule start in
  if t.closed then cleanup cancel else t.cancel <- Some cancel;
  t
