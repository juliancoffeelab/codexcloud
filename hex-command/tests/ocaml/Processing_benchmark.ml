let require = function Ok value -> value | Error error -> failwith (Native.message error)

(* Optional benchmark adapter. Keep native control flow and representation handling
   in compiled OCaml; the runner supplies ordinary JSON and records host strings. *)
let options policy : Domain.ai_options =
  let policy =
    match Utf8.of_js policy with
    | "classic" -> Domain.Policy.Classic
    | "rush" -> Domain.Policy.Rush
    | "balanced" -> Domain.Policy.Balanced
    | "rotated" -> Domain.Policy.Rotated
    | "situational" -> Domain.Policy.Situational
    | "predictor" -> Domain.Policy.Predictor
    | _ -> invalid_arg "Unknown benchmark policy"
  in
  { blocks = None; policy; diplomacy = Domain.Diplomacy.Diplomatic; recruitment_limit = None }

let restore factory checkpoint =
  match Saved_game_json.decode checkpoint with
  | Ok saved -> require (Native.restore factory saved)
  | Error reason -> invalid_arg reason

let phase battle options =
  let steps = ref 0 in
  let enemy () =
    let state = Native.state (Native.view battle) in
    state.turn = Domain.Side.Enemy && state.winner = None
  in
  while enemy () do
    if !steps >= 1000 then failwith "Benchmark phase exceeded 1000 commands";
    incr steps;
    let _ = require (Native.enemy_step battle options) in
    ()
  done;
  require (Native.save_json battle)
