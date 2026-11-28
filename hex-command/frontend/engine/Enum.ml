module Category = Domain.Category
module Damage_type = Domain.Damage_type
module Hex_direction = Domain.Hex_direction
module Planner_kind = Domain.Planner_kind
module Race = Domain.Race
module Recruiter_kind = Domain.Recruiter_kind
module Phase = Domain.Turn_phase
module Terrain = Domain.Terrain
open Domain

let string_of_category (value : Category.t) =
  match value with Category.Melee -> {js|melee|js} | Category.Ranged -> {js|ranged|js}

let string_of_damage_type (value : Damage_type.t) =
  match value with
  | Damage_type.Blade -> {js|blade|js}
  | Damage_type.Pierce -> {js|pierce|js}
  | Damage_type.Impact -> {js|impact|js}

let string_of_hex_direction (value : Hex_direction.t) =
  match value with
  | Hex_direction.E -> {js|e|js}
  | Hex_direction.Se -> {js|se|js}
  | Hex_direction.Sw -> {js|sw|js}
  | Hex_direction.W -> {js|w|js}
  | Hex_direction.Nw -> {js|nw|js}
  | Hex_direction.Ne -> {js|ne|js}

let string_of_planner_kind (value : Planner_kind.t) =
  match value with
  | Planner_kind.Learned -> {js|learned|js}
  | Planner_kind.Classic -> {js|classic|js}
  | Planner_kind.Full -> {js|full|js}
  | Planner_kind.Queued -> {js|queued|js}

let planner_kind_of_string = function
  | "learned" -> Some (Planner_kind.Learned : Planner_kind.t)
  | matched_text_0 when matched_text_0 = {js|classic|js} ->
      Some (Planner_kind.Classic : Planner_kind.t)
  | matched_text_1 when matched_text_1 = {js|full|js} -> Some (Planner_kind.Full : Planner_kind.t)
  | matched_text_2 when matched_text_2 = {js|queued|js} ->
      Some (Planner_kind.Queued : Planner_kind.t)
  | _ -> None

let string_of_race (value : Race.t) =
  match value with
  | Race.Human -> {js|human|js}
  | Race.Merfolk -> {js|merfolk|js}
  | Race.Skeletons -> {js|skeletons|js}
  | Race.Spirit -> {js|spirit|js}
  | Race.Beast -> {js|beast|js}
  | Race.Zombies -> {js|zombies|js}

let string_of_recruiter_kind (value : Recruiter_kind.t) =
  match value with
  | Recruiter_kind.Classic -> {js|classic|js}
  | Recruiter_kind.Rotation -> {js|rotation|js}
  | Recruiter_kind.Frontline -> {js|frontline|js}
  | Recruiter_kind.Balanced -> {js|balanced|js}
  | Recruiter_kind.Situational -> {js|situational|js}
  | Recruiter_kind.Predictor -> {js|predictor|js}
  | Recruiter_kind.Aggressive_predictor -> {js|aggressive-predictor|js}

let recruiter_kind_of_string = function
  | matched_text_3 when matched_text_3 = {js|classic|js} ->
      Some (Recruiter_kind.Classic : Recruiter_kind.t)
  | matched_text_4 when matched_text_4 = {js|rotation|js} ->
      Some (Recruiter_kind.Rotation : Recruiter_kind.t)
  | matched_text_5 when matched_text_5 = {js|frontline|js} ->
      Some (Recruiter_kind.Frontline : Recruiter_kind.t)
  | matched_text_6 when matched_text_6 = {js|balanced|js} ->
      Some (Recruiter_kind.Balanced : Recruiter_kind.t)
  | matched_text_7 when matched_text_7 = {js|situational|js} ->
      Some (Recruiter_kind.Situational : Recruiter_kind.t)
  | matched_text_8 when matched_text_8 = {js|predictor|js} ->
      Some (Recruiter_kind.Predictor : Recruiter_kind.t)
  | matched_text_9 when matched_text_9 = {js|aggressive-predictor|js} ->
      Some (Recruiter_kind.Aggressive_predictor : Recruiter_kind.t)
  | _ -> None

let string_of_phase (value : Phase.t) =
  match value with Phase.Player -> {js|player|js} | Phase.Ai -> {js|ai|js}

let phase_of_string = function
  | matched_text_10 when matched_text_10 = {js|player|js} -> Some (Phase.Player : Phase.t)
  | matched_text_11 when matched_text_11 = {js|ai|js} -> Some (Phase.Ai : Phase.t)
  | _ -> None

let string_of_terrain (value : Terrain.t) =
  match value with
  | Terrain.Cave_passage -> {js|cave-passage|js}
  | Terrain.Cave_wall -> {js|cave-wall|js}
  | Terrain.Monastery -> {js|monastery|js}
  | Terrain.Church -> {js|church|js}
  | Terrain.Graveyard -> {js|graveyard|js}
  | Terrain.Ritual_star -> {js|ritual-star|js}
  | Terrain.Ritual_spent -> {js|ritual-spent|js}
  | Terrain.Castle_bell -> {js|castle-bell|js}
  | Terrain.Castle_wall -> {js|castle-wall|js}
  | Terrain.Plains -> {js|plains|js}
  | Terrain.Road -> {js|road|js}
  | Terrain.Forest -> {js|forest|js}
  | Terrain.Hill -> {js|hill|js}
  | Terrain.Water -> {js|water|js}
  | Terrain.Shallow_water -> {js|shallow-water|js}
  | Terrain.Village -> {js|village|js}
  | Terrain.Keep -> {js|keep|js}
  | Terrain.Grove -> {js|grove|js}
  | Terrain.Fort -> {js|fort|js}
  | Terrain.Ruins -> {js|ruins|js}
  | Terrain.Marsh -> {js|marsh|js}
  | Terrain.Bridge -> {js|bridge|js}
  | Terrain.Ford -> {js|ford|js}
  | Terrain.Landing -> {js|landing|js}
  | Terrain.Ferry -> {js|ferry|js}
  | Terrain.House -> {js|house|js}
  | Terrain.Mountain -> {js|mountain|js}
  | Terrain.Mountain_mine -> {js|mountain-mine|js}
  | Terrain.Burned_village -> {js|burned-village|js}
  | Terrain.Signpost -> {js|signpost|js}

let unit_phase factions (unit : Domain.public_unit) =
  match Array.find_opt (fun (faction : Domain.faction) -> faction.id = unit.faction) factions with
  | Some faction -> faction.phase
  | None -> invalid_arg {js|Unit requires a declared faction|js}
