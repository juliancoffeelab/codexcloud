external then_callbacks :
  'a Js.Promise.t ->
  ('a -> unit) ->
  (Js.Promise.error -> unit) ->
  unit Js.Promise.t = "then"
[@@mel.send]

let observe_error promise success failure =
  let _ =
    then_callbacks promise success (fun error ->
        failure (Browser.error_text error))
  in
  ()
