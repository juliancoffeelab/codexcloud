module Controller = Domain.Controller
module Damage_type = Domain.Damage_type
module Goal = Domain.Goal
module Relation = Domain.Relation
module Role = Domain.Role
module Side = Domain.Side
module Terrain_role = Domain.Terrain_role
open Domain

let dictionary : unit -> 'a Js.Dict.t = [%mel.raw "() => Object.create(null)"]

type ability = { name : string; description : string; facts : string list }
type fact = { label : string; value : float }

type terrain_effect = {
  terrain : string;
  name : string;
  label : string;
  facts : fact list;
  class_name : string;
}

type unit_details = {
  unit : public_unit;
  definition : unit_rule;
  terrain_name : string;
  action_status : string;
  promotions : unit_rule list;
  ability : ability;
  terrain_effects : terrain_effect list;
}

type panel =
  | Hidden
  | Outcome of string * string
  | Details of unit_details
  | Combat of public_unit * public_unit * string

type forecast = {
  side : Side.t;
  remaining_hp : float;
  current_hp : float;
  maximum_hp : float;
  health_label : string;
  current_percent : float;
  minimum_percent : float;
  maximum_percent : float;
  remaining_label : string;
  remaining_class : string;
  impact_label : string;
}

type calculation = { description : string; equation : string }

type combat_result = {
  player : forecast;
  enemy : forecast;
  attack : calculation;
  retaliation : calculation;
}

type attack = {
  weapon : weapon;
  name : string;
  response_name : string;
  button_label : string;
  result : combat_result option;
}

type modifier = { name : string; label : string; class_name : string }
type recruit = { definition : unit_rule; cost : float; enabled : bool }
type recruitment = { options : recruit list; hint : string }

type terrain_panel = {
  tile : tile;
  name : string;
  description : string;
  ownership : (string * string) option;
  facts : string list;
  recruitment : recruitment option;
}

type tile_display = {
  tile : tile;
  occupant : public_unit option;
  classes : string;
  label : string;
  owner_color : string;
  health_width : float;
}

type motion = Move | Attack | Recruit

type changes = {
  tile_effects : (string * motion) list;
  captured_tiles : string list;
  turn_changed : bool;
  new_winner : Side.t option;
  attacked : bool;
  recruited : bool;
  advanced : bool;
  moved_unit : public_unit option;
  attacker : public_unit option;
}

let number = Js.Float.toString
let filter = List.filter
let join a = String.concat ", " a
let signed_value value = if value < 0. then "−" ^ number (-.value) else "+" ^ number value

let damage_name damage =
  if damage = Damage_type.Blade then "Blade"
  else if damage = Damage_type.Pierce then "Pierce"
  else "Impact"

let range minimum maximum =
  if minimum = maximum then number minimum else number minimum ^ "–" ^ number maximum

let weapon_damage (weapon : weapon) =
  range
    (Int32.to_float weapon.damage -. Int32.to_float weapon.spread)
    (Int32.to_float weapon.damage +. Int32.to_float weapon.spread)

let damage_equation (detail : damage_detail) =
  let variable = Int32.to_float detail.roll_min <> Int32.to_float detail.roll_max in
  let term condition sign value label =
    if condition then sign ^ number value ^ " (" ^ label ^ ")" else ""
  in
  let equation =
    (if variable then range (Int32.to_float detail.roll_min) (Int32.to_float detail.roll_max)
     else number (Int32.to_float detail.base))
    ^ term (Int32.to_float detail.modifier < 0.) " − " (-.Int32.to_float detail.modifier) "armor"
    ^ term (Int32.to_float detail.modifier > 0.) " + " (Int32.to_float detail.modifier) "weakness"
    ^ term (Int32.to_float detail.protection <> 0.) " − " (Int32.to_float detail.protection) "cover"
    ^ term
        (Int32.to_float detail.exposure <> 0.)
        " + " (Int32.to_float detail.exposure)
        (if detail.terrain = "" || detail.terrain = "ford" then "shallow water" else detail.terrain)
  in
  (if
     Int32.to_float detail.roll_min +. Int32.to_float detail.modifier
     -. Int32.to_float detail.protection +. Int32.to_float detail.exposure
     < 0.
   then "max(0, " ^ equation ^ ")"
   else equation)
  |> fun per_hit ->
  let counted =
    if float_of_int detail.count = 1. then per_hit
    else "(" ^ per_hit ^ ") × " ^ number (float_of_int detail.count)
  in
  counted ^ " = "
  ^
  if variable then
    number (Int32.to_float detail.minimum)
    ^ "–"
    ^ number (Int32.to_float detail.maximum)
    ^ " (average "
    ^ Js.Float.toFixed detail.expected ~digits:1
    ^ ")"
  else
    number
      (if float_of_int detail.count = 1. then Int32.to_float detail.damage
       else Int32.to_float detail.total)

let terrain_damage_facts rows =
  let rows =
    Array.to_list (Array.map (fun row -> Array.to_list (Array.map Int32.to_float row)) rows)
  in
  match List.concat rows with
  | [] -> []
  | first :: rest when List.for_all (( = ) first) rest -> [ { label = ""; value = first } ]
  | _ ->
      List.concat
        (List.mapi
           (fun category row ->
             let label = if category = 0 then "Melee" else "Ranged" in
             match row with
             | [] -> []
             | _ when List.for_all (( = ) 0.) row -> []
             | first :: rest when List.for_all (( = ) first) rest -> [ { label; value = first } ]
             | _ ->
                 List.mapi
                   (fun damage value ->
                     {
                       label = label ^ " " ^ List.nth [ "Blade"; "Pierce"; "Impact" ] damage;
                       value;
                     })
                   row
                 |> filter (fun fact -> fact.value <> 0.))
           rows)

let weapons catalog (unit : public_unit) =
  Array.to_list (Catalog.unit catalog unit.type_).attacks
  @ List.concat_map (fun (item : item) -> Array.to_list item.attacks) (Array.to_list unit.items)

let ability catalog (unit : public_unit) =
  let rule = Catalog.unit catalog unit.type_ in
  let extras =
    List.concat
      [
        (if Int32.to_float rule.thorns = 0. then []
         else
           [
             "When defending, reflects "
             ^ number (Int32.to_float rule.thorns)
             ^ " damage per incoming melee strike, including killing hits.";
           ]);
        (if Int32.to_float rule.thorn_aura = 0. then []
         else
           [
             "When defending, adjacent allies reflect "
             ^ number (Int32.to_float rule.thorn_aura)
             ^ " damage per incoming melee strike. The strongest aura applies and stacks with \
                Thornshield.";
           ]);
        (let drain =
           List.fold_left
             (fun best (weapon : weapon) -> max best (float_of_int weapon.drain))
             0. (weapons catalog unit)
         in
         if drain = 0. then []
         else
           [
             "Draining attacks restore " ^ number drain
             ^ "% of HP removed, including retaliation, up to maximum HP.";
           ]);
      ]
  in
  {
    name = (if unit.role = Some Role.Leader then "Command" else rule.ability);
    description = rule.description;
    facts =
      List.append extras
        (List.append
           (if unit.role = Some Role.Leader then [ "Losing this unit ends the battle." ] else [])
           (match Int32.to_float rule.healing with
           | healing when healing <> 0. ->
               [
                 "Heals adjacent allies by " ^ number healing
                 ^ " HP at the start of its side’s turn. Only the strongest healer applies.";
               ]
           | _ -> []));
  }

let armor catalog (unit : public_unit) =
  let modifiers = (Catalog.unit catalog unit.type_).damage_modifiers in
  List.map
    (fun (name, value) ->
      {
        name;
        label = (if value = 0. then "0" else signed_value value);
        class_name = (if value < 0. then "resistant" else if value > 0. then "vulnerable" else "");
      })
    [
      ("Blade", Int32.to_float modifiers.blade);
      ("Pierce", Int32.to_float modifiers.pierce);
      ("Impact", Int32.to_float modifiers.impact);
    ]

let weapon_range (weapon : weapon) =
  if float_of_int weapon.range = 1. then "1 hex range"
  else "1–" ^ number (float_of_int weapon.range) ^ " hex range"

let unit_panel catalog view ~selected ~target ~revision =
  let state = Native.state view in
  match (state.campaign, state.winner) with
  | Some _, Some _ -> Hidden
  | _, Some winner when winner = Side.Player ->
      Outcome ("The field is yours.", "Your army defeated the enemy commander.")
  | _, Some _ -> Hidden
  | _ -> (
      match Option.bind selected (Native.unit view) with
      | None -> Hidden
      | Some unit -> (
          let can_act = state.turn = Side.Player && Native.controllable view unit.id in
          match Option.bind target (Native.unit view) with
          | Some defender
            when can_act
                 && Array.exists
                      (fun (other : public_unit) -> other.id = defender.id)
                      (Native.targets view unit.id) ->
              Combat (unit, defender, unit.id ^ "-" ^ defender.id ^ "-" ^ number revision)
          | _ ->
              let definition = Catalog.unit catalog unit.type_ in
              let occupied =
                match Native.tile view (Hex.key unit.tile) with
                | Some tile -> tile
                | None -> invalid_arg "The selected unit has no battlefield tile."
              in
              Details
                {
                  unit;
                  definition;
                  terrain_name = (Catalog.terrain_type catalog occupied.terrain).name;
                  action_status =
                    (if not (Native.controllable view unit.id) then "Not under your control"
                     else if unit.attacked then "Turn finished"
                     else if unit.moved then "Can still attack"
                     else "Ready");
                  promotions =
                    (if can_act then
                       Array.to_list
                         (Array.map (Catalog.unit catalog) (Native.promotion_options view unit.id))
                     else []);
                  ability = ability catalog unit;
                  terrain_effects =
                    List.map2
                      (fun (terrain : terrain_definition) rows ->
                        let cost =
                          Native.movement_cost view unit.type_ (Enum.string_of_terrain terrain.id)
                        in
                        {
                          terrain = Enum.string_of_terrain terrain.id;
                          name = terrain.name;
                          label =
                            (if Js.Float.isFinite cost then
                               number cost ^ if cost = 1. then " Step" else " Steps"
                             else "-");
                          facts = terrain_damage_facts rows;
                          class_name =
                            (if not (Js.Float.isFinite cost) then "blocked"
                             else if cost = 1. then "easy"
                             else if cost = 2. then "slow"
                             else "difficult");
                        })
                      (Catalog.terrain catalog)
                      (Array.to_list definition.environment);
                }))

let death_notice probability =
  if probability = 0. then ""
  else if probability = 1. then " (dies)"
  else
    " ("
    ^ (if probability < 0.01 then "<1"
       else if probability > 0.99 then ">99"
       else number (Float.floor ((probability *. 100.) +. 0.5)))
    ^ "% chance to die)"

let forecast catalog (unit : public_unit) label impact death (health : hp_forecast) =
  let maximum_hp = Int32.to_float (Catalog.unit catalog unit.type_).hp in
  let remaining_hp = Float.floor ((health.expected *. 10.) +. 0.5) /. 10. in
  let minimum_loss = Int32.to_float unit.hp -. Int32.to_float health.maximum
  and maximum_loss = Int32.to_float unit.hp -. Int32.to_float health.minimum in
  let impact_label =
    if minimum_loss >= 0. then
      impact ^ " "
      ^ (if minimum_loss > 0. then "−" else "")
      ^ range minimum_loss maximum_loss ^ " HP"
    else if maximum_loss <= 0. then
      (if label = "You" then "Heal" else "Enemy heals")
      ^ " +"
      ^ range (-.maximum_loss) (-.minimum_loss)
      ^ " HP"
    else "Change −" ^ number maximum_loss ^ "–+" ^ number (-.minimum_loss) ^ " HP"
  in
  {
    side = unit.side;
    remaining_hp;
    current_hp = Int32.to_float unit.hp;
    maximum_hp;
    remaining_label = range (Int32.to_float health.minimum) (Int32.to_float health.maximum);
    remaining_class = (if death = 1. then "fatal" else "");
    health_label =
      label ^ ": "
      ^ number (Int32.to_float unit.hp)
      ^ " HP before, "
      ^ range (Int32.to_float health.minimum) (Int32.to_float health.maximum)
      ^ " HP after this attack, average " ^ number remaining_hp ^ " HP" ^ death_notice death;
    current_percent = Int32.to_float unit.hp /. maximum_hp *. 100.;
    minimum_percent = Int32.to_float health.minimum /. maximum_hp *. 100.;
    maximum_percent = Int32.to_float health.maximum /. maximum_hp *. 100.;
    impact_label = impact_label ^ death_notice death;
  }

let describe catalog label (weapon : weapon) (detail : damage_detail) terrain =
  label ^ ": "
  ^ number (float_of_int weapon.count)
  ^ " × " ^ weapon_damage weapon ^ " "
  ^ String.lowercase_ascii (damage_name weapon.damage_type)
  ^ ", "
  ^ String.lowercase_ascii weapon.name
  ^ ", "
  ^ Enum.string_of_category weapon.category
  ^
  if Int32.to_float detail.protection <> 0. || Int32.to_float detail.exposure <> 0. then
    ", against terrain: " ^ String.lowercase_ascii (Catalog.terrain_type catalog terrain).name
  else ""

let weapon_label (weapon : weapon) (detail : damage_detail) =
  "("
  ^ number (float_of_int weapon.count)
  ^ " × "
  ^ range
      (Int32.to_float detail.minimum /. float_of_int weapon.count)
      (Int32.to_float detail.maximum /. float_of_int weapon.count)
  ^ ") " ^ weapon.name

let attacks catalog view (attacker : public_unit) (defender : public_unit) =
  List.map
    (fun (weapon : weapon) ->
      match Native.attack_details view attacker.id defender.id weapon.id with
      | None ->
          {
            weapon;
            name = weapon.name;
            response_name = "";
            result = None;
            button_label = "Attack with " ^ weapon.name ^ " — out of reach or blocked";
          }
      | Some preview ->
          let terrain id =
            match Native.tile view id with
            | Some tile -> tile.terrain
            | None -> invalid_arg "Combatant has no battlefield tile."
          in
          let response =
            match (preview.response, preview.retaliation_detail) with
            | Some weapon, Some detail ->
                {
                  description =
                    describe catalog "Their attack" weapon detail (terrain (Hex.key attacker.tile));
                  equation = damage_equation detail;
                }
            | _ ->
                {
                  description = "Their attack: none";
                  equation =
                    (if preview.kill_chance = 1. then "0 (enemy defeated)" else "0 (out of range)");
                }
          in
          let response =
            if Int32.to_float preview.thorn_damage = 0. then response
            else
              {
                description =
                  response.description ^ "; thorns reflect damage per incoming melee strike";
                equation =
                  response.equation ^ "; "
                  ^ number
                      (Int32.to_float preview.retaliation -. Int32.to_float preview.thorn_damage)
                  ^ " + "
                  ^ number (Int32.to_float preview.thorn_damage)
                  ^ " (thorns: average across executed strikes) = "
                  ^ number (Int32.to_float preview.retaliation);
              }
          in
          let player =
            forecast catalog attacker "You" "Take" preview.death_chance preview.attacker_health
          in
          let enemy =
            forecast catalog defender "Enemy" "Deal" preview.kill_chance preview.defender_health
          in
          let name = weapon_label weapon preview.damage_detail in
          let response_name =
            match (preview.response, preview.retaliation_detail) with
            | Some weapon, Some detail -> weapon_label weapon detail
            | _ -> "No retaliation"
          in
          {
            weapon;
            name;
            response_name;
            button_label =
              "Attack with " ^ name ^ ". Enemy: " ^ response_name ^ ". " ^ player.health_label
              ^ ". " ^ enemy.health_label ^ ".";
            result =
              Some
                {
                  player;
                  enemy;
                  attack =
                    {
                      description =
                        describe catalog "Your attack" weapon preview.damage_detail
                          (terrain (Hex.key defender.tile));
                      equation = damage_equation preview.damage_detail;
                    };
                  retaliation = response;
                };
          })
    (weapons catalog attacker)

let terrain_facts (terrain : terrain_definition) =
  let role = terrain.role and modifiers = terrain.damage_modifiers in
  let protection =
    match role with
    | Terrain_role.Ford -> "Land troops: +1 damage received. Merfolk: no exposure."
    | Terrain_role.Marsh -> "Land troops: +1 damage received. Merfolk: −1 damage received."
    | Terrain_role.Hill -> "Land troops: −1 damage received. Merfolk: +1 damage received."
    | Terrain_role.Forest | Terrain_role.Grove ->
        "Land troops: −1 ranged damage. Merfolk: +1 damage from all attacks."
    | _ when Int32.to_float modifiers.melee = -1. && Int32.to_float modifiers.ranged = -1. ->
        "Protection: −1 damage from all attacks"
    | _ ->
        let effects =
          List.filter_map
            (fun (name, value) ->
              if value = 0. then None
              else Some ((if value > 0. then "+" else "") ^ number value ^ " " ^ name ^ " damage"))
            [
              ("melee", Int32.to_float modifiers.melee);
              ("ranged", Int32.to_float modifiers.ranged);
              ("blade", Int32.to_float modifiers.blade);
              ("pierce", Int32.to_float modifiers.pierce);
              ("impact", Int32.to_float modifiers.impact);
            ]
        in
        if effects = [] then "Protection: none" else "Protection: " ^ String.concat ", " effects
  in
  (if role = Terrain_role.Water then [] else [ protection ])
  @ (if
       List.mem role
         [
           Terrain_role.Forest;
           Terrain_role.Hill;
           Terrain_role.Grove;
           Terrain_role.Mountain;
           Terrain_role.Cave;
           Terrain_role.Keep;
         ]
     then [ "Blocks ranged shots through this hex" ]
     else [])
  @
  match role with
  | Terrain_role.Village ->
      [
        "+2 gold at the start of its owner’s turn";
        "Heals a friendly occupant by 4 HP at the start of its turn";
      ]
  | Terrain_role.Grove -> [ "Heals its owner’s occupant by 4 HP at the start of its turn" ]
  | Terrain_role.Keep ->
      [
        "Its owner’s leader can recruit into adjacent empty terrain passable to the recruit";
        "Select an empty adjacent hex to recruit";
        "Move onto a keep to claim it";
      ]
  | _ -> []

let recruitment_context catalog view selected (tile : tile) =
  let state = Native.state view in
  if
    Native.at view tile.id <> None
    || state.winner <> None
    || Option.bind selected (Native.unit view) <> None
  then None
  else
    let keeps =
      Array_ext.filter
        (fun (neighbor : tile) ->
          Catalog.terrain_role catalog neighbor.terrain = Terrain_role.Keep
          && (neighbor.owner = Some "player"
             || Option.fold ~none:false
                  ~some:(fun id ->
                    Array.exists
                      (fun (f : faction) -> f.id = id && f.controller = Controller.Human)
                      state.factions)
                  neighbor.owner))
        (Native.neighbors view tile.id)
    in
    if Array.length keeps = 0 then None
    else
      let commanders =
        Array_ext.filter
          (fun (unit : public_unit) ->
            Native.controllable view unit.id && unit.role = Some Role.Leader)
          state.units
      in
      let owns (unit : public_unit) (keep : tile) =
        keep.owner = Option.map (fun (f : unit_faction) -> f.id) (Native.faction view unit.id)
      in
      let stationed (unit : public_unit) =
        Array.exists (fun (keep : tile) -> keep.id = Hex.key unit.tile && owns unit keep) keeps
      in
      let commander =
        match Array.find_opt stationed commanders with
        | Some _ as commander -> commander
        | None -> Array.find_opt (fun unit -> Array.exists (owns unit) keeps) commanders
      in
      Some (commander, Option.fold ~none:false ~some:stationed commander)

let treasury catalog view ~selected ~inspected =
  let actor =
    match selected with
    | Some id -> Native.unit view id
    | None ->
        Option.bind
          (Option.bind inspected (Native.tile view))
          (fun tile ->
            Option.bind (recruitment_context catalog view selected tile) (fun (commander, _) ->
                commander))
  in
  Option.fold ~none:"player"
    ~some:(fun (unit : public_unit) ->
      match Native.faction view unit.id with
      | Some profile when profile.controllable -> profile.id
      | _ -> "player")
    actor

let terrain_panel catalog view ~selected ~inspected =
  Option.map
    (fun (tile : tile) ->
      let state = Native.state view in
      let facts = ref (terrain_facts (Catalog.terrain_type catalog tile.terrain)) in
      Option.iter
        (fun (campaign : campaign_state) ->
          Option.iter
            (function
              | Goal.Seal goal ->
                  facts :=
                    List.append !facts
                      [
                        (if Array.exists (( = ) ("sealed:" ^ tile.id)) campaign.marks then
                           "Binding stone sealed; it will not raise the dead"
                         else
                           "Enter with "
                           ^ (match goal.unit_type with
                             | None -> "a friendly unit"
                             | Some kind ->
                                 "a friendly "
                                 ^ String.lowercase_ascii (Catalog.unit catalog kind).name)
                           ^ " to seal this binding stone");
                      ]
              | _ -> ())
            (Array.find_opt
               (function
                 | Goal.Seal goal -> Array.exists (Hex.matches tile) goal.tiles | _ -> false)
               (Array.append (Story.mission catalog view campaign).source.goals
                  (Story.mission catalog view campaign).source.optional_goals)))
        state.campaign;
      let recruitment =
        Option.map
          (fun (commander, stationed) ->
            let player_turn = state.turn = Side.Player && state.winner = None in
            let faction =
              Option.fold
                ~none:(Option.value ~default:"player" tile.owner)
                ~some:(fun (unit : public_unit) ->
                  Option.fold ~none:"player"
                    ~some:(fun (f : unit_faction) -> f.id)
                    (Native.faction view unit.id))
                commander
            in
            let options =
              List.map
                (fun kind ->
                  {
                    definition = Catalog.unit catalog kind;
                    cost = Native.recruit_price view faction kind;
                    enabled =
                      player_turn && stationed
                      && Option.fold ~none:false
                           ~some:(fun (unit : public_unit) ->
                             Array.exists
                               (fun (destination : tile) -> destination.id = tile.id)
                               (Native.recruit_tiles view unit.id kind))
                           commander;
                  })
                (Array.to_list (Native.recruitment_options view faction))
            in
            let hint =
              if options = [] then "No units are available for recruitment."
              else if not stationed then "Return your leader to this keep to recruit."
              else if not player_turn then "Recruit on your turn."
              else "New recruits act next turn."
            in
            let cheapest =
              List.fold_left (fun cost recruit -> min cost recruit.cost) infinity options
            in
            {
              options;
              hint =
                (hint
                ^
                if player_turn && stationed && Native.gold state.gold faction < cheapest then
                  " Not enough gold."
                else "");
            })
          (recruitment_context catalog view selected tile)
      in
      let owner = tile.owner in
      let ownership =
        match Catalog.terrain_role catalog tile.terrain with
        | Terrain_role.Village | Terrain_role.Keep | Terrain_role.Grove | Terrain_role.Ruins
        | Terrain_role.Fort | Terrain_role.Monastery ->
            Some
              ( (match owner with
                | None -> "Unclaimed " ^ tile.terrain
                | Some id ->
                    (if id = "player" then "Your"
                     else if id = "enemy" then "Enemy"
                     else
                       Option.fold
                         ~none:(String.concat " " (String.split_on_char '-' id))
                         ~some:(fun (f : faction) ->
                           Option.value
                             ~default:(String.concat " " (String.split_on_char '-' id))
                             f.name)
                         (Array.find_opt (fun (f : faction) -> f.id = id) state.factions))
                    ^ " " ^ tile.terrain),
                match owner with None -> "neutral" | Some "player" -> "player" | Some _ -> "enemy"
              )
        | _ -> None
      in
      {
        tile;
        name = (Catalog.terrain_type catalog tile.terrain).name;
        description = (Catalog.terrain_type catalog tile.terrain).description;
        ownership;
        facts = !facts;
        recruitment;
      })
    (Option.bind inspected (fun id -> if Native.fogged view id then None else Native.tile view id))

type battlefield_cell = {
  display : tile_display;
  faction : unit_faction option;
  objectives : string list;
  selected_here : bool;
  reachable : bool;
  target : bool;
  recruitable : bool;
}

type battlefield_cache = {
  mutable catalog : Catalog.t option;
  mutable map : string;
  mutable cells : battlefield_cell Js.Dict.t;
}

let create_battlefield_cache () = { catalog = None; map = ""; cells = dictionary () }

let battlefield ?cache catalog view ~selected ~inspected =
  let state = Native.state view in
  let selected_unit = Option.bind selected (Native.unit view) in
  let can_act =
    state.turn = Side.Player && state.winner = None
    && Option.fold ~none:false
         ~some:(fun (unit : public_unit) -> Native.controllable view unit.id)
         selected_unit
  in
  let reachable =
    if can_act then Native.reachable view (Option.get selected) else Domain.String_map.empty
  in
  let targets = dictionary () in
  if can_act then
    Array.iter
      (fun (unit : public_unit) -> Js.Dict.set targets (Hex.key unit.tile) true)
      (Native.targets view (Option.get selected));
  let objectives = Story.objective_labels catalog view in
  let previous =
    match cache with
    | Some cache when Option.equal ( == ) cache.catalog (Some catalog) && cache.map = state.map_id
      ->
        cache.cells
    | _ -> dictionary ()
  in
  let next = dictionary () in
  let displays =
    Array.map
      (fun (tile : tile) ->
        let occupant =
          Option.bind (Native.at view tile.id) (fun unit ->
              if Native.visible view unit then Some unit else None)
        in
        let reachable = Domain.String_map.find_opt tile.id reachable <> None
        and target = Js.Dict.get targets tile.id <> None in
        let labels = Option.value ~default:[] (Js.Dict.get objectives tile.id) in
        let recruitable =
          inspected = Some tile.id && recruitment_context catalog view selected tile <> None
        in
        let selected_here =
          Option.fold ~none:false
            ~some:(fun (unit : public_unit) -> Hex.key unit.tile = tile.id)
            selected_unit
        in
        let faction =
          Option.bind occupant (fun (unit : public_unit) -> Native.faction view unit.id)
        in
        match Js.Dict.get previous tile.id with
        | Some old
          when old.display.tile == tile
               && Option.equal ( == ) old.display.occupant occupant
               && Option.equal ( == ) old.faction faction
               && old.selected_here = selected_here && old.reachable = reachable
               && old.target = target && old.recruitable = recruitable && old.objectives = labels ->
            Js.Dict.set next tile.id old;
            old.display
        | _ ->
            let classes =
              String.concat " "
                (List.filter (( <> ) "")
                   [
                     "hex";
                     (if labels <> [] then "objective" else "");
                     "terrain-" ^ tile.terrain;
                     (if selected_here then "selected" else "");
                     (if reachable then "reachable" else "");
                     (if target then "target" else "");
                     (if recruitable then "recruitable" else "");
                   ])
            in
            let label =
              List.concat
                [
                  [
                    (Catalog.terrain_type catalog tile.terrain).name;
                    "column " ^ number (Int32.to_float tile.col +. 1.);
                    "row " ^ number (Int32.to_float tile.row +. 1.);
                  ];
                  (match tile.owner with
                  | None -> []
                  | Some side ->
                      [
                        (if side = "player" then "your territory"
                         else String.concat " " (String.split_on_char '-' side) ^ " territory");
                      ]);
                  (match occupant with
                  | None -> []
                  | Some unit ->
                      let name = (Catalog.unit catalog unit.type_).name in
                      List.concat
                        [
                          [
                            (match Native.faction view unit.id with
                              | Some faction when faction.relation = Relation.Neutral -> "neutral "
                              | Some faction when faction.controllable -> "your "
                              | Some faction when faction.relation = Relation.Allied -> "allied "
                              | _ -> "enemy ")
                            ^ Option.fold ~none:name
                                ~some:(fun personal -> personal ^ " (" ^ name ^ ")")
                                (Native.unit_name view unit.id);
                            number (Int32.to_float unit.hp) ^ " health";
                          ];
                          (if unit.attacked then [ "turn finished" ] else []);
                          (if unit.retreating = Some true then [ "retreating" ] else []);
                        ]);
                  List.map (fun label -> "Objective: " ^ label) labels;
                  (if reachable then [ "move here" ] else []);
                  (if target then [ "attack available" ] else []);
                ]
              |> join
            in
            let display =
              {
                tile;
                occupant;
                classes;
                label;
                owner_color =
                  (if tile.owner = Some "player" then "#6fdec0"
                   else
                     match
                       Option.bind tile.owner (fun id ->
                           Array.find_opt (fun (f : faction) -> f.id = id) state.factions)
                     with
                     | Some { palette = "green"; _ } -> "#83b765"
                     | Some { palette = "passive"; _ } -> "#efecdc"
                     | _ -> "#ff9590");
                health_width =
                  Option.fold ~none:0.
                    ~some:(fun (unit : public_unit) ->
                      38. *. Int32.to_float unit.hp
                      /. Int32.to_float (Catalog.unit catalog unit.type_).hp)
                    occupant;
              }
            in
            Js.Dict.set next tile.id
              {
                display;
                faction;
                objectives = labels;
                selected_here;
                reachable;
                target;
                recruitable;
              };
            display)
      state.tiles
  in
  Option.iter
    (fun cache ->
      cache.catalog <- Some catalog;
      cache.map <- state.map_id;
      cache.cells <- next)
    cache;
  Array.to_list displays

let changes (before : state) (after : state) =
  let previous = dictionary () and alive = dictionary () and tile_effects = dictionary () in
  Array.iter (fun (unit : public_unit) -> Js.Dict.set previous unit.id unit) before.units;
  let recruited = ref false
  and advanced = ref false
  and attacked = ref false
  and moved_unit = ref None in
  Array.iter
    (fun (unit : public_unit) ->
      Js.Dict.set alive unit.id true;
      match Js.Dict.get previous unit.id with
      | None ->
          Js.Dict.set tile_effects (Hex.key unit.tile) Recruit;
          recruited := !recruited || (unit.moved && unit.attacked)
      | Some old ->
          if old.type_ <> unit.type_ then (
            advanced := true;
            Js.Dict.set tile_effects (Hex.key unit.tile) Recruit);
          if old.tile <> unit.tile then (
            moved_unit := Some unit;
            Js.Dict.set tile_effects (Hex.key unit.tile) Move);
          if Int32.to_float old.hp > Int32.to_float unit.hp then (
            attacked := true;
            Js.Dict.set tile_effects (Hex.key unit.tile) Attack))
    after.units;
  Array.iter
    (fun (unit : public_unit) ->
      if Js.Dict.get alive unit.id = None then (
        attacked := true;
        Js.Dict.set tile_effects (Hex.key unit.tile) Attack))
    before.units;
  let old_tiles = dictionary () in
  Array.iter (fun (tile : tile) -> Js.Dict.set old_tiles tile.id tile.owner) before.tiles;
  let territory =
    Array.append
      (Array_ext.filter (fun (tile : tile) -> tile.terrain = "village") after.tiles)
      (Array_ext.filter
         (fun (tile : tile) -> List.mem tile.terrain [ "grove"; "fort"; "ruins" ])
         after.tiles)
  in
  let captured_tiles =
    Array_ext.filter
      (fun (tile : tile) -> tile.owner <> None && Js.Dict.get old_tiles tile.id <> Some tile.owner)
      territory
    |> Array.map (fun (tile : tile) -> tile.id)
    |> Array.to_list
  in
  let attacker =
    match
      Array.find_opt
        (fun (unit : public_unit) ->
          unit.side = before.turn && unit.attacked
          && Option.fold ~none:false
               ~some:(fun (old : public_unit) -> old.type_ = unit.type_ && not old.attacked)
               (Js.Dict.get previous unit.id))
        after.units
    with
    | Some unit -> Some unit
    | None ->
        Array.find_opt
          (fun (unit : public_unit) -> unit.side = before.turn && Js.Dict.get alive unit.id = None)
          before.units
  in
  {
    tile_effects = Array.to_list (Js.Dict.entries tile_effects);
    captured_tiles;
    turn_changed = before.turn <> after.turn || before.round <> after.round;
    new_winner = (if before.winner <> after.winner then after.winner else None);
    attacked = !attacked || attacker <> None;
    recruited = !recruited;
    advanced = !advanced;
    moved_unit = !moved_unit;
    attacker;
  }

type exchange_entry = {
  id : string;
  good : bool;
  text : string;
  details : string list;
  strike : int;
}

let loss value = if value = 0. then "0" else "−" ^ number value

let exchange_entries catalog (combat : combat_animation) =
  let units = combat.units in
  let previous = ref [| units.(0).hp; units.(1).hp |] in
  Array.mapi
    (fun index (hit : combat_strike) ->
      let actor = if units.(0).side = hit.side then 0 else 1 in
      let before = !previous in
      previous := hit.health;
      let yours = hit.side = Side.Player in
      let equation =
        number (Int32.to_float hit.raw)
        ^ (if Int32.to_float hit.modifier < 0. then
             " − " ^ number (-.Int32.to_float hit.modifier) ^ " armor"
           else if Int32.to_float hit.modifier > 0. then
             " + " ^ number (Int32.to_float hit.modifier) ^ " weakness"
           else "")
        ^
        if Int32.to_float hit.terrain < 0. then
          " − " ^ number (-.Int32.to_float hit.terrain) ^ " cover"
        else if Int32.to_float hit.terrain > 0. then
          " + " ^ number (Int32.to_float hit.terrain) ^ " exposure"
        else ""
      in
      let equation =
        if Int32.to_float hit.raw +. Int32.to_float hit.modifier +. Int32.to_float hit.terrain < 0.
        then "max(0, " ^ equation ^ ")"
        else equation
      in
      let entry id good text details =
        { id = id ^ "-" ^ string_of_int index; good; text; details; strike = index }
      in
      [
        entry "hit" yours
          ("You "
          ^ (if yours then "deal" else "take")
          ^ " "
          ^ loss (Int32.to_float hit.effective)
          ^ " HP")
          [
            hit.weapon ^ ": rolled "
            ^ number (Int32.to_float hit.raw)
            ^ " from "
            ^ number (Int32.to_float hit.roll_min)
            ^ "–"
            ^ number (Int32.to_float hit.roll_max)
            ^ ".";
            equation ^ " = " ^ number (Int32.to_float hit.damage) ^ " damage";
            "min("
            ^ number (Int32.to_float before.(1 - actor))
            ^ " HP, "
            ^ number (Int32.to_float hit.damage)
            ^ " damage) = "
            ^ number (Int32.to_float hit.effective)
            ^ " HP removed";
          ];
      ]
      @ (if Int32.to_float hit.healed = 0. then []
         else
           [
             entry "drain" yours
               ((if yours then "You heal" else "Opponent heals")
               ^ " +"
               ^ number (Int32.to_float hit.healed)
               ^ " HP")
               [
                 "min(floor("
                 ^ number (Int32.to_float hit.effective)
                 ^ " × "
                 ^ number (float_of_int hit.drain)
                 ^ "%), "
                 ^ number
                     (Int32.to_float (Catalog.unit catalog units.(actor).type_).hp
                     -. Int32.to_float before.(actor))
                 ^ " missing HP) = "
                 ^ number (Int32.to_float hit.healed)
                 ^ " HP restored";
               ];
           ])
      @
      if Int32.to_float hit.thorns = 0. then []
      else
        [
          entry "thorns" (not yours)
            ("You "
            ^ (if yours then "take" else "deal")
            ^ " "
            ^ loss (Int32.to_float hit.reflected)
            ^ " HP · thorns")
            [
              "min("
              ^ number (Int32.to_float before.(actor) +. Int32.to_float hit.healed)
              ^ " HP, "
              ^ number (Int32.to_float hit.thorns)
              ^ " thorns) = "
              ^ number (Int32.to_float hit.reflected)
              ^ " HP removed";
            ];
        ])
    combat.strikes
  |> Array.to_list |> List.concat
