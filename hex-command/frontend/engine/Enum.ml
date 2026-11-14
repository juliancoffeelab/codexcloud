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
  match value with Category.Melee -> "melee" | Category.Ranged -> "ranged"

let string_of_damage_type (value : Damage_type.t) =
  match value with
  | Damage_type.Blade -> "blade"
  | Damage_type.Pierce -> "pierce"
  | Damage_type.Impact -> "impact"

let string_of_hex_direction (value : Hex_direction.t) =
  match value with
  | Hex_direction.E -> "e"
  | Hex_direction.Se -> "se"
  | Hex_direction.Sw -> "sw"
  | Hex_direction.W -> "w"
  | Hex_direction.Nw -> "nw"
  | Hex_direction.Ne -> "ne"

let string_of_planner_kind (value : Planner_kind.t) =
  match value with
  | Planner_kind.Classic -> "classic"
  | Planner_kind.Full -> "full"
  | Planner_kind.Queued -> "queued"

let planner_kind_of_string = function
  | "classic" -> Some (Planner_kind.Classic : Planner_kind.t)
  | "full" -> Some (Planner_kind.Full : Planner_kind.t)
  | "queued" -> Some (Planner_kind.Queued : Planner_kind.t)
  | _ -> None

let string_of_race (value : Race.t) =
  match value with
  | Race.Human -> "human"
  | Race.Merfolk -> "merfolk"
  | Race.Skeletons -> "skeletons"
  | Race.Spirit -> "spirit"
  | Race.Beast -> "beast"
  | Race.Zombies -> "zombies"

let string_of_recruiter_kind (value : Recruiter_kind.t) =
  match value with
  | Recruiter_kind.Classic -> "classic"
  | Recruiter_kind.Rotation -> "rotation"
  | Recruiter_kind.Frontline -> "frontline"
  | Recruiter_kind.Balanced -> "balanced"
  | Recruiter_kind.Situational -> "situational"
  | Recruiter_kind.Predictor -> "predictor"
  | Recruiter_kind.Aggressive_predictor -> "aggressive-predictor"

let recruiter_kind_of_string = function
  | "classic" -> Some (Recruiter_kind.Classic : Recruiter_kind.t)
  | "rotation" -> Some (Recruiter_kind.Rotation : Recruiter_kind.t)
  | "frontline" -> Some (Recruiter_kind.Frontline : Recruiter_kind.t)
  | "balanced" -> Some (Recruiter_kind.Balanced : Recruiter_kind.t)
  | "situational" -> Some (Recruiter_kind.Situational : Recruiter_kind.t)
  | "predictor" -> Some (Recruiter_kind.Predictor : Recruiter_kind.t)
  | "aggressive-predictor" -> Some (Recruiter_kind.Aggressive_predictor : Recruiter_kind.t)
  | _ -> None

let string_of_phase (value : Phase.t) =
  match value with Phase.Player -> "player" | Phase.Ai -> "ai"

let phase_of_string = function
  | "player" -> Some (Phase.Player : Phase.t)
  | "ai" -> Some (Phase.Ai : Phase.t)
  | _ -> None

let string_of_terrain (value : Terrain.t) =
  match value with
  | Terrain.Cave_passage -> "cave-passage"
  | Terrain.Cave_wall -> "cave-wall"
  | Terrain.Monastery -> "monastery"
  | Terrain.Church -> "church"
  | Terrain.Graveyard -> "graveyard"
  | Terrain.Ritual_star -> "ritual-star"
  | Terrain.Ritual_spent -> "ritual-spent"
  | Terrain.Castle_bell -> "castle-bell"
  | Terrain.Castle_wall -> "castle-wall"
  | Terrain.Plains -> "plains"
  | Terrain.Road -> "road"
  | Terrain.Forest -> "forest"
  | Terrain.Hill -> "hill"
  | Terrain.Water -> "water"
  | Terrain.Village -> "village"
  | Terrain.Keep -> "keep"
  | Terrain.Grove -> "grove"
  | Terrain.Fort -> "fort"
  | Terrain.Ruins -> "ruins"
  | Terrain.Marsh -> "marsh"
  | Terrain.Bridge -> "bridge"
  | Terrain.Ford -> "ford"
  | Terrain.Landing -> "landing"
  | Terrain.Ferry -> "ferry"
  | Terrain.House -> "house"
  | Terrain.Mountain -> "mountain"
  | Terrain.Mountain_mine -> "mountain-mine"
  | Terrain.Burned_village -> "burned-village"
  | Terrain.Signpost -> "signpost"

let unit_phase factions (unit : Domain.public_unit) =
  match Array.find_opt (fun (faction : Domain.faction) -> faction.id = unit.faction) factions with
  | Some faction -> faction.phase
  | None -> invalid_arg "Unit requires a declared faction"
