open Domain

type ability = { name : string; description : string; facts : string array }
type fact = { label : string; value : float }
type terrain_effect = { terrain : string; name : string; label : string;
                        facts : fact array; class_name : string }
type unit_details = { unit : public_unit; definition : unit_rule;
                      terrain_name : string; action_status : string;
                      promotions : unit_rule array; ability : ability;
                      terrain_effects : terrain_effect array }
type panel = Hidden | Outcome of string * string | Details of unit_details
           | Combat of public_unit * public_unit * string
type forecast = { side : side; remaining_hp : float; current_hp : float;
                  maximum_hp : float; health_label : string; current_percent : float;
                  remaining_percent : float; impact_label : string }
type calculation = { description : string; equation : string }
type combat_result = { player : forecast; enemy : forecast; attack : calculation; retaliation : calculation }
type attack = { weapon : weapon; button_label : string;
                result : combat_result option }
type modifier = { name : string; label : string; class_name : string }
type recruit = { definition : unit_rule; enabled : bool }
type recruitment = { options : recruit array; hint : string }
type terrain_panel = { tile : tile; name : string; description : string;
                       ownership : (string * string) option; facts : string array;
                       recruitment : recruitment option }
type tile_display = { tile : tile; occupant : public_unit option;
                      classes : string; label : string; owner_color : string;
                      health_width : float }
type motion = Move | Attack | Recruit
type changes = { tile_effects : (string * motion) array; captured_tiles : string array;
                 turn_changed : bool; new_winner : side option;
                 attacked : bool; recruited : bool; advanced : bool;
                 moved_unit : public_unit option; attacker : public_unit option }
let number = Js.Float.toString
let filter f a = Array.of_list (List.filter f (Array.to_list a))
let join a = String.concat ", " (Array.to_list a)
let signed_value value = if value < 0. then {js|−|js} ^ number (-. value) else "+" ^ number value
let damage_name damage = if damage = damage_type_blade then "Blade" else if damage = damage_type_pierce then "Pierce" else "Impact"
let damage_equation (detail : damage_detail) =
  let term condition sign value label = if condition then sign ^ number value ^ " (" ^ label ^ ")" else "" in
  let equation = number detail.base ^
                 term (detail.modifier < 0.) {js| − |js} (-. detail.modifier) "armor" ^
                 term (detail.modifier > 0.) " + " detail.modifier "weakness" ^
                 term (detail.protection <> 0.) {js| − |js} detail.protection "cover" ^
                 term (detail.exposure <> 0.) " + " detail.exposure
                   (if detail.terrain = "" || detail.terrain = "ford" then "shallow water" else detail.terrain) in
  (if detail.modified -. detail.protection +. detail.exposure < 0. then "max(0, " ^ equation ^ ")" else equation)
  ^ " = " ^ number detail.damage
let terrain_damage_facts rows =
  let values = Array.concat (Array.to_list rows) in
  if Array.length values = 0 then [||] else if Array.for_all ((=) values.(0)) values then [|{label = ""; value = values.(0)}|]
  else Array.concat (Array.to_list (Array.mapi (fun category row ->
      let label = if category = 0 then "Melee" else "Ranged" in
      if Array.for_all ((=) 0.) row then [||]
      else if Array.for_all ((=) row.(0)) row then [|{label; value = row.(0)}|]
      else filter (fun fact -> fact.value <> 0.) (Array.mapi (fun damage value ->
          {label = label ^ " " ^ [|"Blade"; "Pierce"; "Impact"|].(damage); value}) row)) rows))
let ability catalog (unit : public_unit) =
  let rule = Catalog.unit catalog unit.type_ in
  { name = if rule.leader = Some true then "Command" else rule.ability; description = rule.description;
    facts = Array.append
        (if rule.leader = Some true || unit.role = Some role_leader then [|"Losing this unit ends the battle."|] else [||])
        (match rule.healing with Some healing when healing <> 0. ->
           [|"Heals adjacent allies by " ^ number healing ^ {js| HP at the start of its side’s turn. Only the strongest healer applies.|js}|]
                               | _ -> [||]) }
let armor catalog (unit : public_unit) =
  let modifiers = (Catalog.unit catalog unit.type_).damageModifiers in
  Array.map (fun (name,value) -> {name; label = if value = 0. then "0" else signed_value value;
                                  class_name = if value < 0. then "resistant" else if value > 0. then "vulnerable" else ""})
    [|"Blade", modifiers.blade; "Pierce", modifiers.pierce; "Impact", modifiers.impact|]
let weapon_range (weapon : weapon) = if weapon.range = 1. then "1 hex range" else {js|1–|js} ^ number weapon.range ^ " hex range"

let unit_panel catalog view ~selected ~target ~revision =
  let state = Native.state view in
  match state.campaign, Js.Nullable.toOption state.winner with
  | Some _, Some _ -> Hidden
  | _, Some winner when winner = side_player -> Outcome ("The field is yours.", "Your army defeated the enemy commander.")
  | _, Some _ -> Outcome ("An army needs a leader.", "Keep your commander protected and use villages to grow your army.")
  | _ -> match Option.bind selected (Native.unit view) with
    | None -> Hidden
    | Some unit ->
      let can_act = state.turn = side_player && unit.side = side_player in
      match Option.bind target (Native.unit view) with
      | Some defender when can_act && Array.exists (fun (other : public_unit) -> other.id = defender.id) (Native.targets view unit.id) ->
        Combat (unit, defender, unit.id ^ "-" ^ defender.id ^ "-" ^ number revision)
      | _ ->
        let definition = Catalog.unit catalog unit.type_ in
        let occupied = match Native.tile view unit.tile with Some tile -> tile
                                                           | None -> invalid_arg "The selected unit has no battlefield tile." in
        Details {unit; definition; terrain_name = (Catalog.terrain_type catalog occupied.terrain).name;
                 action_status = if unit.attacked then "Turn finished" else if unit.moved then "Can still attack" else "Ready";
                 promotions = if can_act then Array.map (Catalog.unit catalog) (Native.promotion_options view unit.id) else [||];
                 ability = ability catalog unit;
                 terrain_effects = Array.mapi (fun index (terrain : terrain_definition) ->
                     let cost = Native.movement_cost view unit.type_ (terrain.id :> string) in
                     let (melee,ranged) = definition.environment.(index) in
                     let row (blade,pierce,impact) = [|blade; pierce; impact|] in
                     {terrain = (terrain.id :> string); name = terrain.name;
                      label = if Js.Float.isFinite cost then number cost ^ (if cost = 1. then " Step" else " Steps") else "-";
                      facts = terrain_damage_facts [|row melee; row ranged|];
                      class_name = if not (Js.Float.isFinite cost) then "blocked" else if cost = 1. then "easy" else if cost = 2. then "slow" else "difficult"})
                     (Catalog.terrain catalog) }
let forecast catalog (unit : public_unit) damage label impact =
  let maximum_hp = (Catalog.unit catalog unit.type_).hp in
  let remaining_hp = max 0. (unit.hp -. damage) in
  { side = unit.side; remaining_hp; current_hp = unit.hp; maximum_hp;
    health_label = label ^ ": " ^ number unit.hp ^ " to " ^ number remaining_hp ^ " HP after this attack";
    current_percent = unit.hp /. maximum_hp *. 100.; remaining_percent = remaining_hp /. maximum_hp *. 100.;
    impact_label = impact ^ " " ^ (if damage = 0. then "0" else {js|−|js} ^ number damage) ^ " HP" ^
                   (if remaining_hp = 0. then " (dies)" else "") }
let describe catalog label (weapon : weapon) (detail : damage_detail) terrain =
  label ^ ": " ^ number weapon.damage ^ " " ^ String.lowercase_ascii (damage_name weapon.damageType) ^ ", " ^
  String.lowercase_ascii weapon.name ^ ", " ^ (weapon.category :> string) ^
  (if detail.protection <> 0. || detail.exposure <> 0. then ", against terrain: " ^
                                                            String.lowercase_ascii (Catalog.terrain_type catalog terrain).name else "")
let attacks catalog view (attacker : public_unit) (defender : public_unit) =
  Array.map (fun (weapon : weapon) -> match Native.attack_details view attacker.id defender.id weapon.id with
      | None -> {weapon; result = None; button_label = "Attack with " ^ weapon.name ^ {js| — out of reach or blocked|js}}
      | Some preview ->
        let terrain id = match Native.tile view id with Some tile -> tile.terrain | None -> invalid_arg "Combatant has no battlefield tile." in
        let warning = if preview.retaliation >= attacker.hp then ", you fall" else if preview.damage >= defender.hp then ", enemy falls" else "" in
        let response = match Js.Nullable.toOption preview.response, Js.Nullable.toOption preview.retaliationDetail with
          | Some weapon, Some detail -> { description = describe catalog "Their attack" weapon detail (terrain attacker.tile); equation = damage_equation detail }
          | _ -> {description = "Their attack: none"; equation = if preview.damage >= defender.hp then "0 (enemy defeated)" else "0 (out of range)"} in
        {weapon; button_label = "Attack with " ^ weapon.name ^ ": deal " ^ number preview.damage ^ " damage, take " ^ number preview.retaliation ^ " damage" ^ warning;
         result = Some {player = forecast catalog attacker preview.retaliation "You" "Take"; enemy = forecast catalog defender preview.damage "Enemy" "Deal";
                        attack = {description = describe catalog "Your attack" weapon preview.damageDetail (terrain defender.tile); equation = damage_equation preview.damageDetail}; retaliation = response}})
    (Catalog.unit catalog attacker.type_).attacks

let terrain_description = function
  | "marsh" -> "Mud and reeds expose land troops; merfolk find protection here."
  | "ruins" -> "Broken walls shelter against ranged attacks."
  | "fort" -> "Earthworks protect against every damage kind. Capture them to hold a crossing."
  | "grove" -> {js|A sacred woodland refuge. Its owner’s occupants heal 4 HP each turn.|js}
  | "plains" | "road" -> "Open grassland with easy footing."
  | "forest" -> "Trees shelter land troops and obstruct ranged shots. Merfolk are exposed here."
  | "hill" -> "High ground shelters land troops and obstructs ranged shots. Merfolk are exposed here."
  | "water" -> "Deep water is impassable to land units. Merfolk can swim through it."
  | "bridge" -> "A sturdy crossing over water with easy footing, but no protection."
  | "ford" -> "Shallow water can be crossed on foot. The open water leaves units exposed."
  | "village" -> "Capture this settlement by moving a unit onto it."
  | "keep" -> {js|A fortified home for its army’s commander.|js}
  | terrain -> invalid_arg ("Unknown terrain: " ^ terrain)
let terrain_facts terrain =
  let protection = match terrain with
    | "ford" -> "Land troops: +1 damage received. Merfolk: no exposure."
    | "marsh" -> {js|Land troops: +1 damage received. Merfolk: −1 damage received.|js}
    | "hill" -> {js|Land troops: −1 damage received. Merfolk: +1 damage received.|js}
    | "fort" -> {js|Protection: −1 damage from all attacks|js}
    | "forest" | "grove" -> {js|Land troops: −1 ranged damage. Merfolk: +1 damage from all attacks.|js}
    | "ruins" -> {js|Protection: −1 damage from ranged attacks|js}
    | _ -> "Protection: none" in
  Array.concat [if terrain = "water" then [||] else [|protection|];
                (match terrain with "forest" | "hill" | "grove" -> [|"Blocks ranged shots through this hex"|] | _ -> [||]);
                (match terrain with
                 | "village" -> [|{js|+2 gold at the start of its owner’s turn|js}; "Heals a friendly occupant by 4 HP at the start of its turn"|]
                 | "grove" -> [|{js|Heals its owner’s occupant by 4 HP at the start of its turn|js}|]
                 | "keep" -> [|{js|Its owner’s leader can recruit into adjacent empty terrain passable to the recruit|js};
                               "Select an empty adjacent hex to recruit"; "Keeps cannot be captured"|]
                 | _ -> [||])]
let recruitment_context catalog view selected (tile : tile) =
  let state = Native.state view in
  if Native.at view tile.id <> None || Js.Nullable.toOption state.winner <> None ||
     Option.bind selected (Native.unit view) <> None then None else
    match Array.find_opt (fun (neighbor : tile) -> neighbor.terrain = "keep" &&
                                                   Js.Nullable.toOption neighbor.owner = Some side_player) (Native.neighbors view tile.id) with
    | None -> None
    | Some keep ->
      let commander = Array.find_opt (fun (unit : public_unit) -> unit.side = side_player &&
                                                                  (Catalog.unit catalog unit.type_).leader = Some true) state.units in
      Some (commander, Option.fold ~none:false ~some:(fun (unit : public_unit) -> unit.tile = keep.id) commander)
let terrain_panel catalog view ~selected ~inspected =
  Option.map (fun (tile : tile) ->
      let state = Native.state view in
      let facts = ref (terrain_facts tile.terrain) in
      Option.iter (fun campaign -> Option.iter (function
          | Goal_seal goal ->
            facts := Array.append !facts [|if Array.exists ((=) ("sealed:" ^ tile.id)) campaign.marks then
                                             "Binding stone sealed; it will not raise the dead" else "Enter with " ^
                                                                                                     (match Js.Nullable.toOption goal.unitType with None -> "a friendly unit"
                                                                                                                                                  | Some kind -> "a friendly " ^ String.lowercase_ascii (Catalog.unit catalog kind).name) ^ " to seal this binding stone"|]
          | _ -> ()) (Array.find_opt (function Goal_seal goal -> Array.exists ((=) tile.id) goal.tiles | _ -> false)
          (Story.mission catalog campaign).source.goals)) state.campaign;
      let recruitment = Option.map (fun (commander,stationed) ->
          let player_turn = state.turn = side_player && Js.Nullable.toOption state.winner = None in
          let options = Array.map (fun kind -> {definition = Catalog.unit catalog kind;
                                                enabled = player_turn && stationed && Option.fold ~none:false
                                                            ~some:(fun (unit : public_unit) -> Array.exists (fun (destination : tile) -> destination.id = tile.id)
                                                                      (Native.recruit_tiles view unit.id kind)) commander}) (Native.recruitment_options view side_player) in
          let hint = if not stationed then "Return your leader to this keep to recruit."
            else if not player_turn then "Recruit on your turn." else "New recruits act next turn." in
          let cheapest = Array.fold_left (fun cost recruit -> min cost recruit.definition.cost) infinity options in
          {options; hint = hint ^ (if player_turn && stationed && state.gold.player < cheapest then " Not enough gold." else "")})
          (recruitment_context catalog view selected tile) in
      let owner = Js.Nullable.toOption tile.owner in
      let ownership = match tile.terrain with
        | "village" | "keep" | "grove" | "ruins" | "fort" -> Some (
            (match owner with None -> "Unclaimed" | Some side -> if side = side_player then "Your" else "Enemy") ^ " " ^ tile.terrain,
            Option.fold ~none:"neutral" ~some:(fun (side : Domain.side) -> (side :> string)) owner)
        | _ -> None in
      {tile; name = (Catalog.terrain_type catalog tile.terrain).name; description = terrain_description tile.terrain;
       ownership; facts = !facts; recruitment}) (Option.bind inspected (Native.tile view))

let battlefield catalog view ~selected ~inspected =
  let state = Native.state view in
  let selected_unit = Option.bind selected (Native.unit view) in
  let can_act = state.turn = side_player && Js.Nullable.toOption state.winner = None &&
                Option.fold ~none:false ~some:(fun (unit : public_unit) -> unit.side = side_player) selected_unit in
  let reachable = if can_act then Native.reachable view (Option.get selected) else Js.Dict.empty () in
  let targets = Js.Dict.empty () in
  if can_act then Array.iter (fun (unit : public_unit) -> Js.Dict.set targets unit.tile true) (Native.targets view (Option.get selected));
  let objectives = Story.objective_labels catalog view in
  Array.map (fun (tile : tile) ->
      let occupant = Native.at view tile.id in
      let reachable = Js.Dict.get reachable tile.id <> None and target = Js.Dict.get targets tile.id <> None in
      let labels = Option.value ~default:[||] (Js.Dict.get objectives tile.id) in
      let recruitable = inspected = Some tile.id && recruitment_context catalog view selected tile <> None in
      let classes = String.concat " " (List.filter ((<>) "") ["hex";
                                                              (if Array.length labels <> 0 then "objective" else ""); "terrain-" ^ tile.terrain;
                                                              (if Option.fold ~none:false ~some:(fun (unit : public_unit) -> unit.tile = tile.id) selected_unit then "selected" else "");
                                                              (if reachable then "reachable" else ""); (if target then "target" else ""); (if recruitable then "recruitable" else "")]) in
      let label = Array.concat [[|(Catalog.terrain_type catalog tile.terrain).name;
                                  "column " ^ number (tile.col +. 1.); "row " ^ number (tile.row +. 1.)|];
                                (match Js.Nullable.toOption tile.owner with None -> [||] | Some side -> [|if side = side_player then "your territory" else "enemy territory"|]);
                                (match occupant with None -> [||] | Some unit ->
                                    let name = (Catalog.unit catalog unit.type_).name in
                                    Array.concat [[|(if unit.side = side_player then "your " else "enemy ") ^
                                                    Option.fold ~none:name ~some:(fun personal -> personal ^ " (" ^ name ^ ")") unit.name; number unit.hp ^ " health"|];
                                                  (if unit.attacked then [|"turn finished"|] else [||]); (if unit.retreating = Some true then [|"retreating"|] else [||])]);
                                Array.map (fun label -> "Objective: " ^ label) labels;
                                (if reachable then [|"move here"|] else [||]); (if target then [|"attack available"|] else [||])] |> join in
      {tile; occupant; classes; label; owner_color = if Js.Nullable.toOption tile.owner = Some side_player then "#6fdec0" else "#ff9590";
       health_width = Option.fold ~none:0. ~some:(fun (unit : public_unit) -> 38. *. unit.hp /. (Catalog.unit catalog unit.type_).hp) occupant}) state.tiles

let changes (before : state) (after : state) =
  let previous = Js.Dict.empty () and alive = Js.Dict.empty () and tile_effects = Js.Dict.empty () in
  Array.iter (fun (unit : public_unit) -> Js.Dict.set previous unit.id unit) before.units;
  let recruited = ref false and advanced = ref false and attacked = ref false and moved_unit = ref None in
  Array.iter (fun (unit : public_unit) ->
      Js.Dict.set alive unit.id true;
      match Js.Dict.get previous unit.id with
      | None -> Js.Dict.set tile_effects unit.tile Recruit; recruited := !recruited || unit.moved && unit.attacked
      | Some old ->
        if old.type_ <> unit.type_ then (advanced := true; Js.Dict.set tile_effects unit.tile Recruit);
        if old.tile <> unit.tile then (moved_unit := Some unit; Js.Dict.set tile_effects unit.tile Move);
        if old.hp > unit.hp then (attacked := true; Js.Dict.set tile_effects unit.tile Attack)) after.units;
  Array.iter (fun (unit : public_unit) -> if Js.Dict.get alive unit.id = None then
                 (attacked := true; Js.Dict.set tile_effects unit.tile Attack)) before.units;
  let old_tiles = Js.Dict.empty () in
  Array.iter (fun (tile : tile) -> Js.Dict.set old_tiles tile.id tile.owner) before.tiles;
  let territory = Array.append (filter (fun (tile : tile) -> tile.terrain = "village") after.tiles)
      (filter (fun (tile : tile) -> List.mem tile.terrain ["grove";"fort";"ruins"]) after.tiles) in
  let captured_tiles = filter (fun (tile : tile) -> Js.Nullable.toOption tile.owner <> None &&
                                                    Js.Dict.get old_tiles tile.id <> Some tile.owner) territory |> Array.map (fun (tile : tile) -> tile.id) in
  let attacker = match Array.find_opt (fun (unit : public_unit) -> unit.side = before.turn && unit.attacked &&
                                                                   Option.fold ~none:false ~some:(fun (old : public_unit) -> old.type_ = unit.type_ && not old.attacked) (Js.Dict.get previous unit.id)) after.units with
  | Some unit -> Some unit
  | None -> Array.find_opt (fun (unit : public_unit) -> unit.side = before.turn && Js.Dict.get alive unit.id = None) before.units in
  { tile_effects = Js.Dict.entries tile_effects; captured_tiles; turn_changed = before.turn <> after.turn || before.round <> after.round;
    new_winner = if before.winner <> after.winner then Js.Nullable.toOption after.winner else None;
    attacked = !attacked || attacker <> None; recruited = !recruited; advanced = !advanced; moved_unit = !moved_unit; attacker }
