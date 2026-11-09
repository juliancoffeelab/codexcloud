(** Valid browser preferences. Every value serializes exactly the supported policy and overrides
    executed by [ai_options]. Imported preferences retain independent policy/override combinations
    from valid saves. *)
type mode = Skirmish | Campaign

type policy = Relaxed | Standard | Veteran
type difficulty_choice = Preset of policy | Custom
type first_turn = Random | First of Domain.Side.t
type t

type difficulty = private {
  policy : policy;
  id : string;
  name : string;
  planner : Domain.Planner_kind.t;
  recruiter : Domain.Recruiter_kind.t;
}

val default : t
val first : t -> Domain.Side.t
val campaign : ?overrides:Domain.Planner_kind.t * Domain.Recruiter_kind.t -> policy -> t

val skirmish :
  first:Domain.Side.t -> planner:Domain.Planner_kind.t -> recruiter:Domain.Recruiter_kind.t -> t

val ai_options : t -> Domain.ai_options
val planners : Domain.Planner_kind.t list
val recruiters : Domain.Recruiter_kind.t list
val difficulties : difficulty list
val preset : policy -> difficulty
val difficulty : Domain.Planner_kind.t -> Domain.Recruiter_kind.t -> difficulty_choice
val completed_turns : t -> Domain.state -> float
val of_wire : Save_format.setup -> t
val to_wire : t -> Save_format.setup
val of_json : Js.Json.t -> t
val to_json : t -> Js.Json.t
val opponent : t -> string * string * string
