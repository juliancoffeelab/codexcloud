type cancellation = Session_replaced | Disposed
type outcome = Completed | Cancelled of cancellation | Failed of Foreign_error.cause
type token = { id : int; combat : Domain.combat_animation option }
type task = { mutable settled : bool; resolve : outcome -> unit }
type t = { token : token; finished : outcome Js.Promise.t; task : task }
type lease = task

let tasks : (token, task) Host_collections.Weak_map.t = Host_collections.Weak_map.create ()
let serial = ref 0

let create combat =
  incr serial;
  let token = { id = !serial; combat } in
  let resolve_ref = ref (fun _ -> ()) in
  let finished =
    Js.Promise.make (fun ~resolve ~reject:_ -> resolve_ref := fun value -> (resolve value [@u]))
  in
  let task = { settled = false; resolve = !resolve_ref } in
  let _ = Host_collections.Weak_map.set tasks token task in
  { token; finished; task }

let token t = t.token
let finished t = t.finished

let settle task outcome =
  if not task.settled then (
    task.settled <- true;
    task.resolve outcome)

let fail t error = settle t.task (Failed error)
let cancel t reason = settle t.task (Cancelled reason)

let claim token =
  match Host_collections.Weak_map.get tasks token with
  | Some task -> task
  | None -> failwith "Unknown action presentation"

let complete lease = settle lease Completed

external owner : token -> Phaser_sound.owner = "%identity"
