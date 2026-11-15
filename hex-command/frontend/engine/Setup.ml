type mode = Skirmish | Campaign
type policy = Relaxed | Standard | Veteran
type difficulty_choice = Preset of policy | Custom
type first_turn = Random | First of Domain.Turn_phase.t

type t = {
  first : Domain.Turn_phase.t;
  policy : policy;
  diplomacy : Domain.Diplomacy.t;
  overrides : (Domain.Planner_kind.t * Domain.Recruiter_kind.t) option;
  player_ai : policy option;
}

type difficulty = {
  policy : policy;
  id : string;
  name : string;
  planner : Domain.Planner_kind.t;
  recruiter : Domain.Recruiter_kind.t;
}

let planners = Domain.[ Planner_kind.Classic; Planner_kind.Queued; Planner_kind.Full ]

let recruiters =
  Domain.
    [
      Recruiter_kind.Classic;
      Recruiter_kind.Balanced;
      Recruiter_kind.Frontline;
      Recruiter_kind.Rotation;
      Recruiter_kind.Situational;
      Recruiter_kind.Predictor;
      Recruiter_kind.Aggressive_predictor;
    ]

let difficulties =
  Domain.
    [
      {
        policy = Relaxed;
        id = {js|classic|js};
        name = {js|Relaxed|js};
        planner = Planner_kind.Classic;
        recruiter = Recruiter_kind.Classic;
      };
      {
        policy = Standard;
        id = {js|situational|js};
        name = {js|Standard|js};
        planner = Planner_kind.Queued;
        recruiter = Recruiter_kind.Situational;
      };
      {
        policy = Veteran;
        id = {js|predictor|js};
        name = {js|Veteran|js};
        planner = Planner_kind.Queued;
        recruiter = Recruiter_kind.Predictor;
      };
    ]

let preset policy =
  match policy with
  | Relaxed -> List.nth difficulties 0
  | Standard -> List.nth difficulties 1
  | Veteran -> List.nth difficulties 2

let difficulty planner recruiter =
  match
    List.find_opt
      (fun preset -> preset.planner = planner && preset.recruiter = recruiter)
      difficulties
  with
  | Some preset -> Preset preset.policy
  | None -> Custom

let first setup = setup.first
let player_ai setup = setup.player_ai
let with_player_ai setup player_ai = { setup with player_ai }

let default =
  {
    first = Domain.Turn_phase.Player;
    policy = Relaxed;
    diplomacy = Domain.Diplomacy.Diplomatic;
    overrides = None;
    player_ai = None;
  }

let campaign ?overrides policy =
  {
    first = Domain.Turn_phase.Player;
    policy;
    diplomacy = Domain.Diplomacy.Diplomatic;
    overrides;
    player_ai = None;
  }

let skirmish ~first ~planner ~recruiter =
  let policy =
    match difficulty planner recruiter with Preset policy -> policy | Custom -> Veteran
  in

  {
    first;
    policy;
    diplomacy = Domain.Diplomacy.Diplomatic;
    overrides = Some (planner, recruiter);
    player_ai = None;
  }

let ai_options setup : Domain.ai_options =
  {
    policy =
      (match setup.policy with
      | Relaxed -> Domain.Policy.Classic
      | Standard -> Domain.Policy.Situational
      | Veteran -> Domain.Policy.Predictor);
    diplomacy = setup.diplomacy;
    recruitment_limit = None;
    blocks =
      Option.map
        (fun (planner, recruiter) ->
          { Domain.planner; recruiter; campaign = Domain.Campaigner_kind.Mission })
        setup.overrides;
  }

let completed_turns setup (state : Domain.state) =
  max 0.
    (((state.round -. 1.) *. 2.)
    +. (if state.turn = Domain.Turn_phase.Ai then 1. else 0.)
    -. if setup.first = Domain.Turn_phase.Ai then 1. else 0.)

let invalid () = invalid_arg {js|Invalid saved setup|js}

(* The saved browser preference type is narrower than Rust's AI configuration.
   This adapter preserves that distinction rather than accepting simulation-only
   policies which the browser cannot offer or restore. *)
let of_wire
    ({ Save_format.first; enemy = { policy; diplomacy; blocks }; player } : Save_format.setup) =
  let policy =
    match policy with `Classic -> Relaxed | `Situational -> Standard | `Predictor -> Veteran
  in
  let overrides =
    Option.map
      (fun ({ Save_format.planner; recruiter; campaign = `Mission } : Save_format.blocks) ->
        (planner, recruiter))
      blocks
  in
  let player_ai =
    Option.map
      (function `Classic -> Relaxed | `Situational -> Standard | `Predictor -> Veteran)
      player
  in
  { first; policy; diplomacy; overrides; player_ai }

let to_wire setup : Save_format.setup =
  {
    first = setup.first;
    player =
      Option.map
        (function Relaxed -> `Classic | Standard -> `Situational | Veteran -> `Predictor)
        setup.player_ai;
    enemy =
      {
        policy =
          (match setup.policy with
          | Relaxed -> `Classic
          | Standard -> `Situational
          | Veteran -> `Predictor);
        diplomacy = setup.diplomacy;
        blocks =
          Option.map
            (fun (planner, recruiter) ->
              ({ Save_format.planner; recruiter; campaign = `Mission } : Save_format.blocks))
            setup.overrides;
      };
  }

let of_json value =
  try of_wire (Save_format.setup_of_json value) with Invalid_argument _ -> invalid ()

let to_json setup = Save_format.setup_to_json (to_wire setup)

let opponent (setup : t) =
  let selected = preset setup.policy in
  let planner, recruiter =
    Option.value ~default:(selected.planner, selected.recruiter) setup.overrides
  in
  ( (if planner = selected.planner && recruiter = selected.recruiter then selected.name
     else {js|Custom|js}),
    Js.Array.join ~sep:{js| |js}
      (Array.of_list
         (List.map
            (fun text ->
              let js_text_part_0 = Js.String.toUpperCase (Js.String.slice ~start:0 ~end_:1 text) in
              let js_text_part_1 = Js.String.slice ~start:1 text in
              {j|$(js_text_part_0)$(js_text_part_1)|j})
            (Array.to_list (Js.String.split ~sep:{js|-|js} (Enum.string_of_planner_kind planner))))),
    Js.Array.join ~sep:{js| |js}
      (Array.of_list
         (List.map
            (fun text ->
              let js_text_part_0 = Js.String.toUpperCase (Js.String.slice ~start:0 ~end_:1 text) in
              let js_text_part_1 = Js.String.slice ~start:1 text in
              {j|$(js_text_part_0)$(js_text_part_1)|j})
            (Array.to_list
               (Js.String.split ~sep:{js|-|js} (Enum.string_of_recruiter_kind recruiter))))) )
