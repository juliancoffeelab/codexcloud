module Controller = Domain.Controller
module Damage_type = Domain.Damage_type
module Relation = Domain.Relation
module Role = Domain.Role
module Phase = Domain.Turn_phase
module Terrain_role = Domain.Terrain_role
open Domain

let dictionary : unit -> 'a Js.Dict.t = [%mel.raw {js|() => Object.create(null)|js}]

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
  definition : Domain.unit_details;
  terrain_name : string;
  action_status : string;
  promotions : Domain.unit_details list;
  ability : ability;
  terrain_effects : terrain_effect list;
}

type panel =
  | Hidden
  | Outcome of string * string
  | Details of unit_details
  | Combat of public_unit * public_unit * string

type forecast = {
  palette : string;
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
  weapon : attack_card;
  name : string;
  response_name : string;
  button_label : string;
  result : combat_result option;
}

type modifier = { name : string; label : string; class_name : string }
type recruit = { definition : Domain.unit_details; cost : float; enabled : bool }
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
  new_winner : Domain.Battle_result.t option;
  attacked : bool;
  recruited : bool;
  advanced : bool;
  moved_unit : public_unit option;
  attacker : public_unit option;
}

let number = Js.Float.toString
let filter = List.filter
let join a = Js.Array.join ~sep:{js|, |js} (Array.of_list a)

let signed_value value =
  if value < 0. then
    let js_text_part_1 = number (-.value) in
    {j|−$(js_text_part_1)|j}
  else
    let js_text_part_1 = number value in
    {j|+$(js_text_part_1)|j}

let damage_name damage =
  if damage = Damage_type.Blade then {js|Blade|js}
  else if damage = Damage_type.Pierce then {js|Pierce|js}
  else {js|Impact|js}

let range minimum maximum =
  if minimum = maximum then number minimum
  else
    let js_text_part_0 = number minimum in
    let js_text_part_2 = number maximum in
    {j|$(js_text_part_0)–$(js_text_part_2)|j}

let weapon_damage (weapon : attack_card) =
  range
    (Int32.to_float weapon.damage -. Int32.to_float weapon.spread)
    (Int32.to_float weapon.damage +. Int32.to_float weapon.spread)

let damage_equation (detail : damage_detail) =
  let variable = Int32.to_float detail.roll_min <> Int32.to_float detail.roll_max in
  let term condition sign value label =
    if condition then
      let js_text_part_1 = number value in
      {j|$(sign)$(js_text_part_1) ($(label))|j}
    else {js||js}
  in
  let equation =
    let js_text_part_0 =
      if variable then range (Int32.to_float detail.roll_min) (Int32.to_float detail.roll_max)
      else number (Int32.to_float detail.base)
    in
    let js_text_part_1 =
      term
        (Int32.to_float detail.modifier < 0.)
        {js| − |js}
        (-.Int32.to_float detail.modifier)
        {js|armor|js}
    in
    let js_text_part_2 =
      term
        (Int32.to_float detail.modifier > 0.)
        {js| + |js} (Int32.to_float detail.modifier) {js|weakness|js}
    in
    let js_text_part_3 =
      term
        (Int32.to_float detail.protection <> 0.)
        {js| − |js}
        (Int32.to_float detail.protection)
        {js|cover|js}
    in
    let js_text_part_4 =
      term
        (Int32.to_float detail.exposure <> 0.)
        {js| + |js} (Int32.to_float detail.exposure)
        (if detail.terrain = {js||js} || detail.terrain = {js|ford|js} then {js|shallow water|js}
         else detail.terrain)
    in
    {j|$(js_text_part_0)$(js_text_part_1)$(js_text_part_2)$(js_text_part_3)$(js_text_part_4)|j}
  in
  (if
     Int32.to_float detail.roll_min +. Int32.to_float detail.modifier
     -. Int32.to_float detail.protection +. Int32.to_float detail.exposure
     < 0.
   then {j|max(0, $(equation))|j}
   else equation)
  |> fun per_hit ->
  let counted =
    if float_of_int detail.count = 1. then per_hit
    else
      let js_text_part_3 = number (float_of_int detail.count) in
      {j|($(per_hit)) × $(js_text_part_3)|j}
  in
  let js_text_part_2 =
    if variable then
      let js_text_part_0 = number (Int32.to_float detail.minimum) in
      let js_text_part_2 = number (Int32.to_float detail.maximum) in
      let js_text_part_4 = Js.Float.toFixed detail.expected ~digits:1 in
      {j|$(js_text_part_0)–$(js_text_part_2) (average $(js_text_part_4))|j}
    else
      number
        (if float_of_int detail.count = 1. then Int32.to_float detail.damage
         else Int32.to_float detail.total)
  in
  {j|$(counted) = $(js_text_part_2)|j}

let terrain_damage_facts rows =
  let rows =
    Array.to_list (Array.map (fun row -> Array.to_list (Array.map Int32.to_float row)) rows)
  in
  match List.concat rows with
  | [] -> []
  | first :: rest when List.for_all (( = ) first) rest -> [ { label = {js||js}; value = first } ]
  | _ ->
      List.concat
        (List.mapi
           (fun category row ->
             let label = if category = 0 then {js|Melee|js} else {js|Ranged|js} in
             match row with
             | [] -> []
             | _ when List.for_all (( = ) 0.) row -> []
             | first :: rest when List.for_all (( = ) first) rest -> [ { label; value = first } ]
             | _ ->
                 List.mapi
                   (fun damage value ->
                     {
                       label =
                         (let js_text_part_2 =
                            List.nth [ {js|Blade|js}; {js|Pierce|js}; {js|Impact|js} ] damage
                          in
                          {j|$(label) $(js_text_part_2)|j});
                       value;
                     })
                   row
                 |> filter (fun fact -> fact.value <> 0.))
           rows)

let weapons view (unit : public_unit) = Array.to_list (Native.unit_details view unit.id).attacks

let ability view (unit : public_unit) =
  let rule = Native.unit_details view unit.id in
  let effects = Array.to_list rule.effects in
  let facts =
    List.filter_map
      (fun (entry : scoped_effect) ->
        let radius = Js.Int.toString entry.scope.radius in
        let self =
          if entry.scope.include_self then {js|Includes itself.|js} else {js|Excludes itself.|js}
        in
        match entry.effect_ with
        | Effect_value.Healing { power } ->
            let power = number (Int32.to_float power) in
            Some
              {j|Heals by $(power) HP within $(radius) tiles at the start of its side’s turn. $(self)|j}
        | Effect_value.Thorns { damage_per_strike; incoming_category } ->
            let damage = number (Int32.to_float damage_per_strike) in
            let category =
              match incoming_category with
              | Category.Melee -> {js|melee|js}
              | Ranged -> {js|ranged|js}
            in
            Some
              {j|Reflects $(damage) damage per incoming $(category) strike within $(radius) tiles. $(self)|j}
        | _ -> None)
      effects
  in
  let drain =
    List.fold_left (fun best (weapon : attack_card) -> max best weapon.drain) 0 (weapons view unit)
  in
  let facts =
    if drain = 0 then facts
    else
      facts
      @ [
          (let drain = Js.Int.toString drain in
           {j|Draining attacks restore $(drain)% of HP removed, up to maximum HP.|j});
        ]
  in
  {
    name = (if unit.role = Some Role.Leader then {js|Command|js} else rule.ability);
    description = rule.description;
    facts =
      (facts
      @ if unit.role = Some Role.Leader then [ {js|Losing this unit ends the battle.|js} ] else []);
  }

let armor view (unit : public_unit) =
  let modifiers = (Native.unit_details view unit.id).damage_modifiers in
  List.map
    (fun (name, value) ->
      {
        name;
        label = (if value = 0. then {js|0|js} else signed_value value);
        class_name =
          (if value < 0. then {js|resistant|js}
           else if value > 0. then {js|vulnerable|js}
           else {js||js});
      })
    [
      ({js|Blade|js}, Int32.to_float modifiers.blade);
      ({js|Pierce|js}, Int32.to_float modifiers.pierce);
      ({js|Impact|js}, Int32.to_float modifiers.impact);
    ]

let weapon_range (weapon : attack_card) =
  if float_of_int weapon.range = 1. then {js|1 hex range|js}
  else
    let js_text_part_1 = number (float_of_int weapon.range) in
    {j|1–$(js_text_part_1) hex range|j}

let unit_panel catalog view ~selected ~target ~revision =
  let state = Native.state view in
  match (state.campaign, state.winner) with
  | Some _, Some _ -> Hidden
  | _, Some winner when winner = Domain.Battle_result.Player_victory ->
      Outcome ({js|The field is yours.|js}, {js|Your army defeated the enemy commander.|js})
  | _, Some _ -> Hidden
  | _ -> (
      match Option.bind selected (Native.unit view) with
      | None -> Hidden
      | Some unit -> (
          let can_act = state.turn = Phase.Player && Native.controllable view unit.id in
          match Option.bind target (Native.unit view) with
          | Some defender
            when can_act
                 && Array.exists
                      (fun (other : public_unit) -> other.id = defender.id)
                      (Native.targets view unit.id) ->
              Combat
                ( unit,
                  defender,
                  let js_text_part_0 = unit.id in
                  let js_text_part_2 = defender.id in
                  let js_text_part_4 = number revision in
                  {j|$(js_text_part_0)-$(js_text_part_2)-$(js_text_part_4)|j} )
          | _ ->
              let definition = Native.unit_details view unit.id in
              let occupied =
                match Native.tile view (Hex.key unit.tile) with
                | Some tile -> tile
                | None -> invalid_arg {js|The selected unit has no battlefield tile.|js}
              in
              Details
                {
                  unit;
                  definition;
                  terrain_name = (Catalog.terrain_type catalog occupied.terrain).name;
                  action_status =
                    (if not (Native.controllable view unit.id) then {js|Not under your control|js}
                     else if unit.attack_spent then {js|Turn finished|js}
                     else if unit.moved then {js|Can still attack|js}
                     else {js|Ready|js});
                  promotions =
                    (if can_act then Array.to_list (Native.promotion_details view unit.id) else []);
                  ability = ability view unit;
                  terrain_effects =
                    Array.to_list definition.terrain
                    |> List.sort_uniq (fun (a : unit_tile_details) (b : unit_tile_details) ->
                        compare
                          (a.terrain, a.cost, a.defense, a.healing)
                          (b.terrain, b.cost, b.defense, b.healing))
                    |> List.map (fun (tile : unit_tile_details) ->
                        let cost = Option.fold ~none:infinity ~some:float_of_int tile.cost in
                        {
                          terrain = Enum.string_of_terrain tile.terrain;
                          name = tile.name;
                          label =
                            (if Js.Float.isFinite cost then
                               let n = number cost in
                               let suffix = if cost = 1. then {js|Step|js} else {js|Steps|js} in
                               {j|$(n) $(suffix)|j}
                             else {js|-|js});
                          facts = terrain_damage_facts tile.defense;
                          class_name =
                            (if not (Js.Float.isFinite cost) then {js|blocked|js}
                             else if cost = 1. then {js|easy|js}
                             else if cost = 2. then {js|slow|js}
                             else {js|difficult|js});
                        });
                }))

let death_notice probability =
  if probability = 0. then {js||js}
  else if probability = 1. then {js| (dies)|js}
  else
    let js_text_part_1 =
      if probability < 0.01 then {js|<1|js}
      else if probability > 0.99 then {js|>99|js}
      else number (Float.floor ((probability *. 100.) +. 0.5))
    in
    {j| ($(js_text_part_1)% chance to die)|j}

let forecast _catalog view (unit : public_unit) label impact death (health : hp_forecast) =
  let maximum_hp = Int32.to_float (Native.unit_details view unit.id).hp in
  let remaining_hp = Float.floor ((health.expected *. 10.) +. 0.5) /. 10. in
  let minimum_loss = Int32.to_float unit.hp -. Int32.to_float health.maximum
  and maximum_loss = Int32.to_float unit.hp -. Int32.to_float health.minimum in
  let impact_label =
    if minimum_loss >= 0. then
      let js_text_part_2 = if minimum_loss > 0. then {js|−|js} else {js||js} in
      let js_text_part_3 = range minimum_loss maximum_loss in
      {j|$(impact) $(js_text_part_2)$(js_text_part_3) HP|j}
    else if maximum_loss <= 0. then
      let js_text_part_0 = if label = {js|You|js} then {js|Heal|js} else {js|Ai heals|js} in
      let js_text_part_2 = range (-.maximum_loss) (-.minimum_loss) in
      {j|$(js_text_part_0) +$(js_text_part_2) HP|j}
    else
      let js_text_part_1 = number maximum_loss in
      let js_text_part_3 = number (-.minimum_loss) in
      {j|Change −$(js_text_part_1)–+$(js_text_part_3) HP|j}
  in
  {
    palette =
      Option.fold ~none:{js|passive|js}
        ~some:(fun (f : unit_faction) -> f.palette)
        (Native.faction view unit.id);
    remaining_hp;
    current_hp = Int32.to_float unit.hp;
    maximum_hp;
    remaining_label = range (Int32.to_float health.minimum) (Int32.to_float health.maximum);
    remaining_class = (if death = 1. then {js|fatal|js} else {js||js});
    health_label =
      (let js_text_part_2 = number (Int32.to_float unit.hp) in
       let js_text_part_4 = range (Int32.to_float health.minimum) (Int32.to_float health.maximum) in
       let js_text_part_6 = number remaining_hp in
       let js_text_part_8 = death_notice death in
       {j|$(label): $(js_text_part_2) HP before, $(js_text_part_4) HP after this attack, average $(js_text_part_6) HP$(js_text_part_8)|j});
    current_percent = Int32.to_float unit.hp /. maximum_hp *. 100.;
    minimum_percent = Int32.to_float health.minimum /. maximum_hp *. 100.;
    maximum_percent = Int32.to_float health.maximum /. maximum_hp *. 100.;
    impact_label =
      (let js_text_part_1 = death_notice death in
       {j|$(impact_label)$(js_text_part_1)|j});
  }

let describe catalog label (weapon : attack_card) (detail : damage_detail) terrain =
  let js_text_part_2 = number (float_of_int weapon.count) in
  let js_text_part_4 = weapon_damage weapon in
  let js_text_part_6 = Js.String.toLowerCase (damage_name weapon.damage_type) in
  let js_text_part_8 = Js.String.toLowerCase weapon.name in
  let js_text_part_10 = Enum.string_of_category weapon.category in
  let js_text_part_11 =
    if Int32.to_float detail.protection <> 0. || Int32.to_float detail.exposure <> 0. then
      let js_text_part_1 = Js.String.toLowerCase (Catalog.terrain_type catalog terrain).name in
      {j|, against terrain: $(js_text_part_1)|j}
    else {js||js}
  in
  {j|$(label): $(js_text_part_2) × $(js_text_part_4) $(js_text_part_6), $(js_text_part_8), $(js_text_part_10)$(js_text_part_11)|j}

let weapon_label (weapon : attack_card) (detail : damage_detail) =
  let js_text_part_1 = number (float_of_int weapon.count) in
  let js_text_part_3 =
    range
      (Int32.to_float detail.minimum /. float_of_int weapon.count)
      (Int32.to_float detail.maximum /. float_of_int weapon.count)
  in
  let js_text_part_5 = weapon.name in
  {j|($(js_text_part_1) × $(js_text_part_3)) $(js_text_part_5)|j}

let attacks catalog view (attacker : public_unit) (defender : public_unit) =
  List.map
    (fun (weapon : attack_card) ->
      match Native.attack_details view attacker.id defender.id weapon.id with
      | None ->
          {
            weapon;
            name = weapon.name;
            response_name = {js||js};
            result = None;
            button_label =
              (let js_text_part_1 = weapon.name in
               {j|Attack with $(js_text_part_1) — out of reach or blocked|j});
          }
      | Some preview ->
          let terrain id =
            match Native.tile view id with
            | Some tile -> tile.terrain
            | None -> invalid_arg {js|Combatant has no battlefield tile.|js}
          in
          let response =
            match (preview.response, preview.retaliation_detail) with
            | Some weapon, Some detail ->
                {
                  description =
                    describe catalog {js|Their attack|js} weapon detail
                      (terrain (Hex.key attacker.tile));
                  equation = damage_equation detail;
                }
            | _ ->
                {
                  description = {js|Their attack: none|js};
                  equation =
                    (if preview.kill_chance = 1. then {js|0 (enemy defeated)|js}
                     else {js|0 (out of range)|js});
                }
          in
          let response =
            if Int32.to_float preview.thorn_damage = 0. then response
            else
              {
                description =
                  (let js_text_part_0 = response.description in
                   {j|$(js_text_part_0); thorns reflect damage per incoming melee strike|j});
                equation =
                  (let js_text_part_0 = response.equation in
                   let js_text_part_2 =
                     number
                       (Int32.to_float preview.retaliation -. Int32.to_float preview.thorn_damage)
                   in
                   let js_text_part_4 = number (Int32.to_float preview.thorn_damage) in
                   let js_text_part_6 = number (Int32.to_float preview.retaliation) in
                   {j|$(js_text_part_0); $(js_text_part_2) + $(js_text_part_4) (thorns: average across executed strikes) = $(js_text_part_6)|j});
              }
          in
          let player =
            forecast catalog view attacker {js|You|js} {js|Take|js} preview.death_chance
              preview.attacker_health
          in
          let enemy =
            forecast catalog view defender {js|Ai|js} {js|Deal|js} preview.kill_chance
              preview.defender_health
          in
          let name = weapon_label weapon preview.damage_detail in
          let response_name =
            match (preview.response, preview.retaliation_detail) with
            | Some weapon, Some detail -> weapon_label weapon detail
            | _ -> {js|No retaliation|js}
          in
          {
            weapon;
            name;
            response_name;
            button_label =
              (let js_text_part_5 = player.health_label in
               let js_text_part_7 = enemy.health_label in
               {j|Attack with $(name). Ai: $(response_name). $(js_text_part_5). $(js_text_part_7).|j});
            result =
              Some
                {
                  player;
                  enemy;
                  attack =
                    {
                      description =
                        describe catalog {js|Your attack|js} weapon preview.damage_detail
                          (terrain (Hex.key defender.tile));
                      equation = damage_equation preview.damage_detail;
                    };
                  retaliation = response;
                };
          })
    (weapons view attacker)

let terrain_facts (terrain : wiki_terrain) (physical : tile_details)
    (actor : unit_tile_details option) =
  let protection =
    Option.fold ~none:[]
      ~some:(fun (profile : unit_tile_details) ->
        terrain_damage_facts profile.defense
        |> List.filter_map (fun fact ->
            if fact.value = 0. then None
            else
              let n = signed_value fact.value in
              let label = fact.label in
              Some {j|$(label): $(n) damage received|j}))
      actor
  in
  let healing =
    Option.fold ~none:[]
      ~some:(fun (profile : unit_tile_details) ->
        if Int32.to_float profile.healing = 0. then []
        else
          let power = number (Int32.to_float profile.healing) in
          [ {j|Heals this unit by $(power) HP at the start of its turn|j} ])
      actor
  in
  protection @ healing
  @ (if physical.blocks_shots then [ {js|Blocks ranged shots through this hex|js} ] else [])
  @ (if Int32.to_float physical.income = 0. then []
     else
       let amount = number (Int32.to_float physical.income) in
       [ {j|+$(amount) gold at the start of its owner’s turn|j} ])
  @
  if terrain.role = Terrain_role.Keep then
    [
      {js|Its owner’s leader can recruit into adjacent empty terrain passable to the recruit|js};
      {js|Select an empty adjacent hex to recruit|js};
      {js|Move onto a keep to claim it|js};
    ]
  else []

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
          && (neighbor.owner = Some {js|player|js}
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
  Option.fold ~none:{js|player|js}
    ~some:(fun (unit : public_unit) ->
      match Native.faction view unit.id with
      | Some profile when profile.controllable -> profile.id
      | _ -> {js|player|js})
    actor

let terrain_panel catalog view ~selected ~inspected =
  Option.map
    (fun (tile : tile) ->
      let state = Native.state view in
      let actor =
        match Option.bind selected (Native.unit view) with
        | Some unit -> Some unit
        | None -> Native.at view tile.id
      in
      let actor =
        Option.bind actor (fun (unit : public_unit) ->
            Array.find_opt
              (fun (profile : unit_tile_details) -> Hex.key profile.tile = tile.id)
              (Native.unit_details view unit.id).terrain)
      in
      let physical = Native.tile_details view tile.id in
      let facts = terrain_facts (Catalog.terrain_type catalog tile.terrain) physical actor in
      let recruitment =
        Option.map
          (fun (commander, stationed) ->
            let player_turn = state.turn = Phase.Player && state.winner = None in
            let faction =
              Option.fold
                ~none:(Option.value ~default:{js|player|js} tile.owner)
                ~some:(fun (unit : public_unit) ->
                  Option.fold ~none:{js|player|js}
                    ~some:(fun (f : unit_faction) -> f.id)
                    (Native.faction view unit.id))
                commander
            in
            let options =
              List.map
                (fun kind ->
                  {
                    definition = Native.recruit_details view faction kind;
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
              if options = [] then {js|No units are available for recruitment.|js}
              else if not stationed then {js|Return your leader to this keep to recruit.|js}
              else if not player_turn then {js|Recruit on your turn.|js}
              else {js|New recruits act next turn.|js}
            in
            let cheapest =
              List.fold_left (fun cost recruit -> min cost recruit.cost) infinity options
            in
            {
              options;
              hint =
                (let js_text_part_1 =
                   if player_turn && stationed && Native.gold state.gold faction < cheapest then
                     {js| Not enough gold.|js}
                   else {js||js}
                 in
                 {j|$(hint)$(js_text_part_1)|j});
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
                | None ->
                    let js_text_part_1 = tile.terrain in
                    {j|Unclaimed $(js_text_part_1)|j}
                | Some id ->
                    let js_text_part_0 =
                      if id = {js|player|js} then {js|Your|js}
                      else if id = {js|enemy|js} then {js|Ai|js}
                      else
                        Option.fold
                          ~none:
                            (Js.Array.join ~sep:{js| |js}
                               (Array.of_list (Array.to_list (Js.String.split ~sep:{js|-|js} id))))
                          ~some:(fun (f : faction) ->
                            Option.value
                              ~default:
                                (Js.Array.join ~sep:{js| |js}
                                   (Array.of_list
                                      (Array.to_list (Js.String.split ~sep:{js|-|js} id))))
                              f.name)
                          (Array.find_opt (fun (f : faction) -> f.id = id) state.factions)
                    in
                    let js_text_part_2 = tile.terrain in
                    {j|$(js_text_part_0) $(js_text_part_2)|j}),
                match owner with
                | None -> {js|neutral|js}
                | Some matched_text_0 when matched_text_0 = {js|player|js} -> {js|player|js}
                | Some _ -> {js|enemy|js} )
        | _ -> None
      in
      {
        tile;
        name = (Catalog.terrain_type catalog tile.terrain).name;
        description = (Catalog.terrain_type catalog tile.terrain).description;
        ownership;
        facts;
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

let create_battlefield_cache () = { catalog = None; map = {js||js}; cells = dictionary () }

let battlefield ?cache catalog view ~selected ~inspected =
  let state = Native.state view in
  let selected_unit = Option.bind selected (Native.unit view) in
  let can_act =
    state.turn = Phase.Player && state.winner = None
    && Option.fold ~none:false
         ~some:(fun (unit : public_unit) -> Native.controllable view unit.id)
         selected_unit
  in
  let reachable =
    if can_act then Native.reachable view (Option.get selected) else Domain.Text_map.empty
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
        let reachable = Domain.Text_map.find_opt tile.id reachable <> None
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
              Js.Array.join ~sep:{js| |js}
                (Array.of_list
                   (List.filter (( <> ) {js||js})
                      [
                        {js|hex|js};
                        (if labels <> [] then {js|objective|js} else {js||js});
                        (let js_text_part_1 = tile.terrain in
                         {j|terrain-$(js_text_part_1)|j});
                        (if selected_here then {js|selected|js} else {js||js});
                        (if reachable then {js|reachable|js} else {js||js});
                        (if target then {js|target|js} else {js||js});
                        (if recruitable then {js|recruitable|js} else {js||js});
                      ]))
            in
            let label =
              List.concat
                [
                  [
                    (Catalog.terrain_type catalog tile.terrain).name;
                    (let js_text_part_1 = number (Int32.to_float tile.col +. 1.) in
                     {j|column $(js_text_part_1)|j});
                    (let js_text_part_1 = number (Int32.to_float tile.row +. 1.) in
                     {j|row $(js_text_part_1)|j});
                  ];
                  (match tile.owner with
                  | None -> []
                  | Some side ->
                      [
                        (if side = {js|player|js} then {js|your territory|js}
                         else
                           let js_text_part_0 =
                             Js.Array.join ~sep:{js| |js}
                               (Array.of_list (Array.to_list (Js.String.split ~sep:{js|-|js} side)))
                           in
                           {j|$(js_text_part_0) territory|j});
                      ]);
                  (match occupant with
                  | None -> []
                  | Some unit ->
                      let name = (Catalog.unit catalog unit.type_).name in
                      List.concat
                        [
                          [
                            (let js_text_part_0 =
                               match Native.faction view unit.id with
                               | Some faction when faction.relation = Relation.Neutral ->
                                   {js|neutral |js}
                               | Some faction when faction.controllable -> {js|your |js}
                               | Some faction when faction.relation = Relation.Allied ->
                                   {js|allied |js}
                               | _ -> {js|enemy |js}
                             in
                             let js_text_part_1 =
                               Option.fold ~none:name
                                 ~some:(fun personal -> {j|$(personal) ($(name))|j})
                                 (Native.unit_name view unit.id)
                             in
                             {j|$(js_text_part_0)$(js_text_part_1)|j});
                            (let js_text_part_0 = number (Int32.to_float unit.hp) in
                             {j|$(js_text_part_0) health|j});
                          ];
                          (if unit.attack_spent then [ {js|turn finished|js} ] else []);
                          (if unit.retreating = Some true then [ {js|retreating|js} ] else []);
                        ]);
                  List.map (fun label -> {j|Objective: $(label)|j}) labels;
                  (if reachable then [ {js|move here|js} ] else []);
                  (if target then [ {js|attack available|js} ] else []);
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
                  (if tile.owner = Some {js|player|js} then {js|#6fdec0|js}
                   else
                     match
                       Option.bind tile.owner (fun id ->
                           Array.find_opt (fun (f : faction) -> f.id = id) state.factions)
                     with
                     | Some { palette = matched_text_1; _ } when matched_text_1 = {js|green|js} ->
                         {js|#83b765|js}
                     | Some { palette = matched_text_2; _ } when matched_text_2 = {js|passive|js} ->
                         {js|#efecdc|js}
                     | _ -> {js|#ff9590|js});
                health_width =
                  Option.fold ~none:0.
                    ~some:(fun (unit : public_unit) ->
                      38. *. Int32.to_float unit.hp
                      /. Int32.to_float (Native.unit_details view unit.id).hp)
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
          recruited := !recruited || (unit.moved && unit.attack_spent)
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
      (Array_ext.filter (fun (tile : tile) -> tile.terrain = {js|village|js}) after.tiles)
      (Array_ext.filter
         (fun (tile : tile) -> List.mem tile.terrain [ {js|grove|js}; {js|fort|js}; {js|ruins|js} ])
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
          Enum.unit_phase before.factions unit = before.turn
          && unit.attack_spent
          && Option.fold ~none:false
               ~some:(fun (old : public_unit) -> old.type_ = unit.type_ && not old.attack_spent)
               (Js.Dict.get previous unit.id))
        after.units
    with
    | Some unit -> Some unit
    | None ->
        Array.find_opt
          (fun (unit : public_unit) ->
            Enum.unit_phase before.factions unit = before.turn && Js.Dict.get alive unit.id = None)
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

let loss value =
  if value = 0. then {js|0|js}
  else
    let js_text_part_1 = number value in
    {j|−$(js_text_part_1)|j}

let exchange_entries _catalog (combat : combat_animation) =
  let units = combat.units in
  let previous = ref [| units.(0).hp; units.(1).hp |] in
  Array.mapi
    (fun index (hit : combat_strike) ->
      let actor = if units.(0).id = hit.unit_id then 0 else 1 in
      let before = !previous in
      previous := hit.health;
      let yours = units.(actor).faction = {js|player|js} in
      let equation =
        let js_text_part_0 = number (Int32.to_float hit.raw) in
        let js_text_part_1 =
          if Int32.to_float hit.modifier < 0. then
            let js_text_part_1 = number (-.Int32.to_float hit.modifier) in
            {j| − $(js_text_part_1) armor|j}
          else if Int32.to_float hit.modifier > 0. then
            let js_text_part_1 = number (Int32.to_float hit.modifier) in
            {j| + $(js_text_part_1) weakness|j}
          else {js||js}
        in
        let js_text_part_2 =
          if Int32.to_float hit.terrain < 0. then
            let js_text_part_1 = number (-.Int32.to_float hit.terrain) in
            {j| − $(js_text_part_1) cover|j}
          else if Int32.to_float hit.terrain > 0. then
            let js_text_part_1 = number (Int32.to_float hit.terrain) in
            {j| + $(js_text_part_1) exposure|j}
          else {js||js}
        in
        {j|$(js_text_part_0)$(js_text_part_1)$(js_text_part_2)|j}
      in
      let equation =
        if Int32.to_float hit.raw +. Int32.to_float hit.modifier +. Int32.to_float hit.terrain < 0.
        then {j|max(0, $(equation))|j}
        else equation
      in
      let entry id good text details =
        {
          id =
            (let js_text_part_2 = string_of_int index in
             {j|$(id)-$(js_text_part_2)|j});
          good;
          text;
          details;
          strike = index;
        }
      in
      [
        entry {js|hit|js} yours
          (let js_text_part_1 = if yours then {js|deal|js} else {js|take|js} in
           let js_text_part_3 = loss (Int32.to_float hit.effective) in
           {j|You $(js_text_part_1) $(js_text_part_3) HP|j})
          [
            (let js_text_part_0 = hit.weapon in
             let js_text_part_2 = number (Int32.to_float hit.raw) in
             let js_text_part_4 = number (Int32.to_float hit.roll_min) in
             let js_text_part_6 = number (Int32.to_float hit.roll_max) in
             {j|$(js_text_part_0): rolled $(js_text_part_2) from $(js_text_part_4)–$(js_text_part_6).|j});
            (let js_text_part_2 = number (Int32.to_float hit.damage) in
             {j|$(equation) = $(js_text_part_2) damage|j});
            (let js_text_part_1 = number (Int32.to_float before.(1 - actor)) in
             let js_text_part_3 = number (Int32.to_float hit.damage) in
             let js_text_part_5 = number (Int32.to_float hit.effective) in
             {j|min($(js_text_part_1) HP, $(js_text_part_3) damage) = $(js_text_part_5) HP removed|j});
          ];
      ]
      @ (if Int32.to_float hit.healed = 0. then []
         else
           [
             entry {js|drain|js} yours
               (let js_text_part_0 = if yours then {js|You heal|js} else {js|Opponent heals|js} in
                let js_text_part_2 = number (Int32.to_float hit.healed) in
                {j|$(js_text_part_0) +$(js_text_part_2) HP|j})
               [
                 (let js_text_part_1 = number (Int32.to_float hit.effective) in
                  let js_text_part_3 = number (float_of_int hit.drain) in
                  let js_text_part_5 =
                    number
                      (Int32.to_float combat.maximum_hp.(actor) -. Int32.to_float before.(actor))
                  in
                  let js_text_part_7 = number (Int32.to_float hit.healed) in
                  {j|min(floor($(js_text_part_1) × $(js_text_part_3)%), $(js_text_part_5) missing HP) = $(js_text_part_7) HP restored|j});
               ];
           ])
      @
      if Int32.to_float hit.thorns = 0. then []
      else
        [
          entry {js|thorns|js} (not yours)
            (let js_text_part_1 = if yours then {js|take|js} else {js|deal|js} in
             let js_text_part_3 = loss (Int32.to_float hit.reflected) in
             {j|You $(js_text_part_1) $(js_text_part_3) HP · thorns|j})
            [
              (let js_text_part_1 =
                 number (Int32.to_float before.(actor) +. Int32.to_float hit.healed)
               in
               let js_text_part_3 = number (Int32.to_float hit.thorns) in
               let js_text_part_5 = number (Int32.to_float hit.reflected) in
               {j|min($(js_text_part_1) HP, $(js_text_part_3) thorns) = $(js_text_part_5) HP removed|j});
            ];
        ])
    combat.strikes
  |> Array.to_list |> List.concat
