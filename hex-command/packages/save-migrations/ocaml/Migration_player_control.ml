(* Frozen v41 -> v42. Player automation is a saved battle setup choice.
   Native checkpoints and undo histories are unchanged. *)
open Historical_types

[@@@warning "+8+9"]
[@@@warnerror "+8+9"]

let setup ({ wire; first; enemy } : V41.current_setup) : V42.current_setup =
  { wire; first; enemy; player = `Null }

let v41 (value : V41.t) : V42.t =
  match value with
  | `Current { wire; pending_enemy; undo; progress; version = `V41; checkpoint; setup = s } ->
      `Current { wire; pending_enemy; undo; progress; version = `V42; checkpoint; setup = setup s }
  | `Turn { wire; pending_enemy; undo; progress; version = `V41; checkpoint; setup = s } ->
      `Turn { wire; pending_enemy; undo; progress; version = `V42; checkpoint; setup = setup s }
