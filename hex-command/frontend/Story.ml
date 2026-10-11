open Domain

type chapter = { source : mission; briefing : string; debrief : string option;
                 hints : string array; intro : dialogue array;
                 victory : dialogue array; recruits : string array }
type progress = { label : string; current : float; total : float;
                  done_ : bool; tiles : string array }
let filter f values = Array.of_list (List.filter f (Array.to_list values))
let includes values value = Array.exists ((=) value) values
let or_empty = Option.value ~default:[||]
let marked (state : campaign_state) mark = includes state.marks mark

let mission catalog (state : campaign_state) =
  let source = (Catalog.campaign catalog state.id).missions.(int_of_float state.mission) in
  let original = { source; briefing = source.briefing; debrief = source.debrief;
                   hints = source.hints; intro = source.intro; victory = source.victory; recruits = source.recruits } in
  let patch chapter (change : mission_patch) =
    { chapter with briefing = Option.value ~default:chapter.briefing change.briefing;
                   debrief = (match change.debrief with None -> chapter.debrief | value -> value);
                   hints = Option.value ~default:chapter.hints change.hints;
                   intro = Option.value ~default:chapter.intro change.intro;
                   victory = Option.value ~default:chapter.victory change.victory;
                   recruits = Option.value ~default:chapter.recruits change.recruits } in
  let without_dain = includes (or_empty state.choices) "refuge-war" ||
                     includes (or_empty state.choices) "green-dain-dead" || marked state "dain-fell" in
  let chapter = if without_dain then Option.fold ~none:original ~some:(patch original) source.withoutDain
    else original in
  (* Preserve authored marker order while applying each fallen companion once. *)
  let fallen = Array.append (or_empty state.fallenCompanions)
      (Array.map (fun mark -> String.sub mark 7 (String.length mark - 7))
         (filter (String.starts_with ~prefix:"fallen:") state.marks)) in
  let seen = Js.Dict.empty () in
  let chapter = Array.fold_left (fun chapter name ->
      if Js.Dict.get seen name <> None then chapter else begin
        Js.Dict.set seen name true;
        match Option.bind source.companionLosses (fun losses -> Js.Dict.get losses name) with
        | None -> chapter | Some change -> patch chapter change
      end) chapter fallen in
  let chapter = { chapter with intro = filter (fun (line : dialogue) -> not (includes fallen line.speaker)) chapter.intro;
                               victory = filter (fun (line : dialogue) -> not (includes fallen line.speaker)) chapter.victory } in
  match state.refuge, source.refuge with
  | Some counter, Some refuge when counter.escalated ->
    { chapter with briefing = refuge.warBriefing; hints = refuge.warHints }
  | _ -> chapter

let goal_tiles = function
  | Goal_defeat _ -> [||]
  | Goal_control goal -> goal.tiles | Goal_seal goal -> goal.tiles
  | Goal_event goal -> Option.fold ~none:[||] ~some:(fun tile -> [|tile|]) (Js.Nullable.toOption goal.tile)
  | Goal_escort goal -> [|goal.tile|] | Goal_escape goal -> [|goal.tile|]
  | Goal_hold goal -> [|goal.tile|] | Goal_vigil goal -> [|goal.tile|]
  | Goal_refuge goal -> [|goal.tile|]

let progress catalog view goal =
  let state = Native.state view in
  let campaign = state.campaign in
  let marked mark = Option.fold ~none:false ~some:(fun state -> marked state mark) campaign in
  let owned id = Option.fold ~none:false ~some:(fun (tile : tile) ->
      Js.Nullable.toOption tile.owner = Some side_player) (Native.tile view id) in
  let exists role tile = Array.exists (fun (unit : public_unit) ->
      unit.side = side_player && unit.role = Some role && unit.tile = tile) state.units in
  let count f values = float_of_int (Array.length (filter f values)) in
  let yes value = if value then 1. else 0. in
  let held get = Option.fold ~none:0. ~some:get campaign in
  let label, current, total, tiles, hold_done = match goal with
    | Goal_defeat goal -> goal.label, yes (not (Array.exists (fun (unit : public_unit) ->
        unit.side = side_enemy && unit.role = Some role_leader) state.units)), 1., [||], true
    | Goal_escort goal -> goal.label, yes (exists role_courier goal.tile), 1., [|goal.tile|], true
    | Goal_escape goal -> goal.label, yes (exists role_leader goal.tile), 1., [|goal.tile|], true
    | Goal_control goal -> goal.label, count owned goal.tiles, float_of_int (Array.length goal.tiles), goal.tiles, true
    | Goal_seal goal -> goal.label, count (fun tile -> marked ("sealed:" ^ tile)) goal.tiles,
                        float_of_int (Array.length goal.tiles), goal.tiles, true
    | Goal_event goal -> goal.label, yes (marked ("event:" ^ goal.event)), 1., Option.fold ~none:[||] ~some:(fun tile -> [|tile|]) (Js.Nullable.toOption goal.tile), true
    | Goal_hold goal ->
      let started = Option.fold ~none:true ~some:(fun event -> marked ("event:" ^ event)) (Js.Nullable.toOption goal.startEvent) in
      let label = if started then goal.label else if not (owned goal.tile) then "Reach the wreck to start the rescue"
        else "Wait for high tide on turn " ^ Js.Float.toString goal.earliestRound in
      let current = if not started then 0. else held (fun (state : campaign_state) ->
          Option.fold ~none:0. ~some:(fun holds -> Option.fold ~none:0. ~some:(fun (counter : counter) -> counter.heldTurns)
                                         (Js.Dict.get holds goal.tile)) state.holds) in
      label, min current goal.turns, goal.turns, [|goal.tile|], state.turn = side_player && state.round >= goal.earliestRound && owned goal.tile
    | Goal_vigil goal -> goal.label, min goal.turns (held (fun (state : campaign_state) ->
        Option.fold ~none:0. ~some:(fun (counter : counter) -> counter.heldTurns) state.vigil)), goal.turns, [|goal.tile|], true
    | Goal_refuge goal ->
      (match campaign with
       | Some state when Option.fold ~none:false ~some:(fun (counter : refuge_counter) -> counter.escalated) state.refuge ->
         "Defeat the remaining patrol", yes (not (Array.exists (fun (unit : public_unit) -> unit.side = side_enemy) (Native.state view).units)), 1., [||], true
       | _ ->
         let total = Option.fold ~none:goal.turns ~some:(fun (state : campaign_state) ->
             match (mission catalog state).source.refuge with
             | Some refuge when marked ("event:" ^ refuge.medicineEvent) -> refuge.medicineTurns
             | _ -> goal.turns) campaign in
         goal.label, min total (held (fun (state : campaign_state) -> Option.fold ~none:0.
                                         ~some:(fun (counter : refuge_counter) -> counter.heldTurns) state.refuge)), total, [|goal.tile|], true) in
  { label; current; total; tiles; done_ = current = total && hold_done }

let active_tiles catalog view goal =
  let state = Native.state view in
  if Js.Nullable.toOption state.winner <> None then [||] else match goal with
    | Goal_seal goal -> filter (fun tile -> not (Option.fold ~none:false
                                                   ~some:(fun state -> marked state ("sealed:" ^ tile)) state.campaign)) goal.tiles
    | Goal_event _ when (progress catalog view goal).done_ -> [||]
    | _ -> goal_tiles goal
let objective_labels catalog view =
  let labels = Js.Dict.empty () in
  Option.iter (fun state -> Array.iter (fun goal ->
      let label = (progress catalog view goal).label in
      Array.iter (fun tile -> Js.Dict.set labels tile (Array.append
                                                         (Option.value ~default:[||] (Js.Dict.get labels tile)) [|label|])) (active_tiles catalog view goal))
      (mission catalog state).source.goals) (Native.state view).campaign;
  labels
let dialogue_pending = Option.fold ~none:false ~some:(fun (state : campaign_state) ->
    state.dialogueIndex < float_of_int (Array.length state.dialogue))
let returns_with_story catalog (state : campaign_state) (unit : public_unit) =
  let chapters = (Catalog.campaign catalog state.id).missions in
  let next = int_of_float state.mission + 1 in
  next < Array.length chapters && Option.fold ~none:false ~some:(fun name ->
      Array.exists (fun (incoming : starting_unit) -> incoming.side = side_player && incoming.name = Some name)
        chapters.(next).map.units) unit.name
let ends_refuge_mediation catalog (state : state) (target : public_unit) =
  Option.fold ~none:false ~some:(fun campaign ->
      (mission catalog campaign).source.refuge <> None &&
      not (Option.fold ~none:false ~some:(fun (counter : refuge_counter) -> counter.escalated) campaign.refuge) &&
      target.side = side_enemy && (target.retreating = Some true || target.role = Some role_leader)) state.campaign
