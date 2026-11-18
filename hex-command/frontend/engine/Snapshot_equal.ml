module Category = Domain.Category
module Condition = Domain.Condition
module Condition_side = Domain.Condition_side
module Damage_type = Domain.Damage_type
module Relation = Domain.Relation
module Role = Domain.Role
module Side = Domain.Turn_phase

(* No physical-identity shortcut: a snapshot containing NaN is unequal to itself. *)
open Domain

let option equal left right =
  match (left, right) with
  | None, None -> true
  | Some left, Some right -> equal left right
  | _ -> false

let sequence = Array_ext.equal

let rec battlefield_light (left : battlefield_light) (right : battlefield_light) =
  option
    (fun left right -> sequence (fun left right -> left = right) left right)
    left.color right.color
  && option (fun left right -> left = right) left.fixture right.fixture
  && left.col = right.col && left.row = right.row
  && option (fun left right -> left = right) left.dx right.dx
  && option (fun left right -> left = right) left.dy right.dy
  && left.radius = right.radius && left.power = right.power
  && option (fun left right -> condition_rule left right) left.when_ right.when_

and battlefield_lighting (left : battlefield_lighting) (right : battlefield_lighting) =
  sequence Hex.equal left.unlit_tiles right.unlit_tiles
  && sequence (fun left right -> battlefield_light left right) left.lights right.lights

and condition (left : Condition.t) (right : Condition.t) =
  match left with
  | Condition.Owner { tile = left_tile; side = left_side } -> (
      match right with
      | Condition.Owner { tile = right_tile; side = right_side } ->
          option (fun left right -> left = right) left_tile right_tile && left_side = right_side
      | _ -> false)
  | Condition.Proximity
      {
        unit_types = left_unit_types;
        tile = left_tile;
        radius = left_radius;
        side = left_side;
        unit_type = left_unit_type;
        character = left_character;
      } -> (
      match right with
      | Condition.Proximity
          {
            unit_types = right_unit_types;
            tile = right_tile;
            radius = right_radius;
            side = right_side;
            unit_type = right_unit_type;
            character = right_character;
          } ->
          sequence (fun left right -> left = right) left_unit_types right_unit_types
          && option (fun left right -> left = right) left_tile right_tile
          && left_radius = right_radius
          && condition_side left_side right_side
          && option (fun left right -> left = right) left_unit_type right_unit_type
          && option (fun left right -> left = right) left_character right_character
      | _ -> false)
  | Condition.Visible_units { side = left_side; unit_types = left_unit_types } -> (
      match right with
      | Condition.Visible_units { side = right_side; unit_types = right_unit_types } ->
          condition_side left_side right_side
          && sequence (fun left right -> left = right) left_unit_types right_unit_types
      | _ -> false)
  | Condition.Has_units
      {
        ai_group = left_ai_group;
        side = left_side;
        unit_type = left_unit_type;
        character = left_character;
      } -> (
      match right with
      | Condition.Has_units
          {
            ai_group = right_ai_group;
            side = right_side;
            unit_type = right_unit_type;
            character = right_character;
          } ->
          option (fun left right -> left = right) left_ai_group right_ai_group
          && condition_side left_side right_side
          && option (fun left right -> left = right) left_unit_type right_unit_type
          && option (fun left right -> left = right) left_character right_character
      | _ -> false)
  | Condition.No_units { side = left_side; unit_type = left_unit_type; character = left_character }
    -> (
      match right with
      | Condition.No_units
          { side = right_side; unit_type = right_unit_type; character = right_character } ->
          condition_side left_side right_side
          && option (fun left right -> left = right) left_unit_type right_unit_type
          && option (fun left right -> left = right) left_character right_character
      | _ -> false)
  | Condition.Choice { choice = left_choice } -> (
      match right with
      | Condition.Choice { choice = right_choice } -> left_choice = right_choice
      | _ -> false)
  | Condition.Flag { flag = left_flag } -> (
      match right with Condition.Flag { flag = right_flag } -> left_flag = right_flag | _ -> false)
  | Condition.Event { event = left_event } -> (
      match right with
      | Condition.Event { event = right_event } -> left_event = right_event
      | _ -> false)
  | Condition.Round { round = left_round } -> (
      match right with
      | Condition.Round { round = right_round } -> left_round = right_round
      | _ -> false)
  | Condition.All { conditions = left_conditions } -> (
      match right with
      | Condition.All { conditions = right_conditions } ->
          sequence (fun left right -> condition left right) left_conditions right_conditions
      | _ -> false)
  | Condition.Any { conditions = left_conditions } -> (
      match right with
      | Condition.Any { conditions = right_conditions } ->
          sequence (fun left right -> condition left right) left_conditions right_conditions
      | _ -> false)

and condition_rule (left : condition_rule) (right : condition_rule) =
  condition left.condition right.condition && left.remember = right.remember

and condition_side (left : Condition_side.t) (right : Condition_side.t) =
  match left with
  | Condition_side.Player -> ( match right with Condition_side.Player -> true | _ -> false)
  | Condition_side.Enemy -> ( match right with Condition_side.Enemy -> true | _ -> false)
  | Condition_side.Neutral -> ( match right with Condition_side.Neutral -> true | _ -> false)
  | Condition_side.Any -> ( match right with Condition_side.Any -> true | _ -> false)
  | Condition_side.Observer -> ( match right with Condition_side.Observer -> true | _ -> false)

and perception_effect (left : perception_effect) (right : perception_effect) =
  left.id = right.id
  && sequence ( = ) left.penetrates right.penetrates
  && sequence ( = ) left.concealment right.concealment
  && option ( = ) left.light right.light

and public_unit (left : public_unit) (right : public_unit) =
  left.faction = right.faction
  && sequence ( = ) left.revealed_to right.revealed_to
  && sequence perception_effect left.perception_effects right.perception_effects
  && sequence ( = ) left.items right.items
  && option (fun left right -> left = right) left.ai_group right.ai_group
  && left.id = right.id && left.type_ = right.type_ && Hex.equal left.tile right.tile
  && left.hp = right.hp && left.moved = right.moved
  && left.attack_spent = right.attack_spent
  && option (fun left right -> left = right) left.experience right.experience
  && option (fun left right -> left = right) left.character right.character
  && option (fun left right -> role left right) left.role right.role
  && option (fun left right -> left = right) left.retreating right.retreating
  && option (fun left right -> left = right) left.rebuilding right.rebuilding

and relation (left : Relation.t) (right : Relation.t) =
  match left with
  | Relation.Allied -> ( match right with Relation.Allied -> true | _ -> false)
  | Relation.Neutral -> ( match right with Relation.Neutral -> true | _ -> false)
  | Relation.Hostile -> ( match right with Relation.Hostile -> true | _ -> false)

and role (left : Role.t) (right : Role.t) =
  match left with
  | Role.Leader -> ( match right with Role.Leader -> true | _ -> false)
  | Role.Courier -> ( match right with Role.Courier -> true | _ -> false)
  | Role.Captive -> ( match right with Role.Captive -> true | _ -> false)

and side (left : Side.t) (right : Side.t) =
  match left with
  | Side.Player -> ( match right with Side.Player -> true | _ -> false)
  | Side.Ai -> ( match right with Side.Ai -> true | _ -> false)

and unit_faction (left : unit_faction) (right : unit_faction) =
  left.id = right.id && left.palette = right.palette
  && relation left.relation right.relation
  && left.controllable = right.controllable
  && side left.phase right.phase
