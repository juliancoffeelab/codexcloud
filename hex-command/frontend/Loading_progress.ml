type t = { step : int; value : float; detail : string }

let state = ref { step = 1; value = 0.; detail = "" }
let listeners = ref []
let current () = !state

let label step =
  match step with
  | 1 -> "Loading game engine…"
  | 2 -> "Restoring battle…"
  | 3 -> "Starting battlefield renderer…"
  | 4 -> "Loading images and sounds…"
  | _ -> "Drawing battlefield…"

let publish next =
  if next <> !state then (
    state := next;
    Loading_motion.advance ~step:next.step ~value:next.value;
    List.iter (fun (_, notify) -> notify next) !listeners)

let subscribe notify =
  let token = ref () in
  listeners := (token, notify) :: !listeners;
  fun () -> listeners := List.filter (fun (id, _) -> id != token) !listeners

(* Five completed-work segments, not an estimate of elapsed time. Only asset
   preparation has measurable work within its segment. Never rewind a run. *)
let advance ?(fraction = 0.) ?(detail = "") step =
  let value = float_of_int (step - 1) +. max 0. (min 1. fraction) in
  if step >= !state.step then publish { step; value = max !state.value value; detail }

let begin_battlefield () =
  if !state.step > 3 then Loading_motion.restart ~step:3 ~value:2.;
  publish { step = 3; value = 2.; detail = "" }

let finish () = publish { step = 5; value = 5.; detail = "" }
