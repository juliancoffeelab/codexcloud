let require = function Ok value -> value | Error error -> failwith (Native.message error)

(* Optional benchmark adapter. Keep native control flow and representation handling
   in compiled OCaml; the runner supplies ordinary JSON and records host strings. *)
let options policy : Domain.ai_options =
  let policy =
    match policy with
    | matched_text_0 when matched_text_0 = {js|classic|js} -> Domain.Policy.Classic
    | matched_text_1 when matched_text_1 = {js|rush|js} -> Domain.Policy.Rush
    | matched_text_2 when matched_text_2 = {js|balanced|js} -> Domain.Policy.Balanced
    | matched_text_3 when matched_text_3 = {js|rotated|js} -> Domain.Policy.Rotated
    | matched_text_4 when matched_text_4 = {js|situational|js} -> Domain.Policy.Situational
    | matched_text_5 when matched_text_5 = {js|predictor|js} -> Domain.Policy.Predictor
    | _ -> invalid_arg {js|Unknown benchmark policy|js}
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
    state.turn = Domain.Turn_phase.Ai && state.winner = None
  in
  while enemy () do
    if !steps >= 1000 then failwith {js|Benchmark phase exceeded 1000 commands|js};
    incr steps;
    let _ = require (Native.enemy_step battle options) in
    ()
  done;
  require (Native.save_json battle)
