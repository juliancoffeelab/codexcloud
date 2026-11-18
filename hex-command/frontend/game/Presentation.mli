type ability = { name : string; description : string; facts : string list }
(** Pure display policy over detached native publications. All legality, movement costs, recruitment
    destinations and combat forecasts come from Rust's inspection; these projections never call or
    mutate a live battle. *)

type fact = { label : string; value : float }

type terrain_effect = {
  terrain : string;
  name : string;
  label : string;
  facts : fact list;
  class_name : string;
}

type unit_details = {
  unit : Domain.public_unit;
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
  | Combat of Domain.public_unit * Domain.public_unit * string

val unit_panel :
  Catalog.t ->
  Native.view ->
  selected:string option ->
  target:string option ->
  revision:float ->
  panel
(** Selection, winner and target policy; the combat key resets optional details for each published
    render revision. *)

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
  weapon : Domain.attack_card;
  name : string;
  response_name : string;
  button_label : string;
  result : combat_result option;
}

val attacks : Catalog.t -> Native.view -> Domain.public_unit -> Domain.public_unit -> attack list
(** One entry per authored weapon. [result = None] is an unavailable attack; forecasts and equations
    use the native legal current-position preview. *)

val damage_equation : Domain.damage_detail -> string
val signed_value : float -> string
val terrain_damage_facts : int32 array array -> fact list
val weapons : Native.view -> Domain.public_unit -> Domain.attack_card list
val ability : Native.view -> Domain.public_unit -> ability

type modifier = { name : string; label : string; class_name : string }

val armor : Native.view -> Domain.public_unit -> modifier list
val weapon_damage : Domain.attack_card -> string
val weapon_range : Domain.attack_card -> string
val damage_name : Domain.Damage_type.t -> string

type recruit = { definition : Domain.unit_details; cost : float; enabled : bool }
type recruitment = { options : recruit list; hint : string }

type terrain_panel = {
  tile : Domain.tile;
  name : string;
  description : string;
  ownership : (string * string) option;
  facts : string list;
  recruitment : recruitment option;
}

val terrain_panel :
  Catalog.t ->
  Native.view ->
  selected:string option ->
  inspected:string option ->
  terrain_panel option
(** Inspection facts and recruitment choices share the same revision and selection. A selected unit
    suppresses recruitment into inspected ground. *)

type tile_display = {
  tile : Domain.tile;
  occupant : Domain.public_unit option;
  classes : string;
  label : string;
  owner_color : string;
  health_width : float;
}

(** Accessible tile descriptions and CSS states. Artwork and geometry are rendered separately, so
    unit movement cannot replace the terrain layer. *)

type battlefield_cache
(** Per-component display reuse. Previously returned cells remain detached; labels are formatted
    only when a cell's visible facts change. *)

val create_battlefield_cache : unit -> battlefield_cache

val battlefield :
  ?cache:battlefield_cache ->
  Catalog.t ->
  Native.view ->
  selected:string option ->
  inspected:string option ->
  tile_display list

(** Feedback describes publication differences without DOM or audio effects. *)
type motion = Move | Attack | Recruit

type changes = {
  tile_effects : (string * motion) list;
  captured_tiles : string list;
  turn_changed : bool;
  new_winner : Domain.Battle_result.t option;
  attacked : bool;
  recruited : bool;
  advanced : bool;
  moved_unit : Domain.public_unit option;
  attacker : Domain.public_unit option;
}

val changes : Domain.state -> Domain.state -> changes

type exchange_entry = {
  id : string;
  good : bool;
  text : string;
  details : string list;
  strike : int;
}

val exchange_entries : Catalog.t -> Domain.combat_animation -> exchange_entry list
val loss : float -> string

val treasury :
  Catalog.t -> Native.view -> selected:string option -> inspected:string option -> string

val recruitment_context :
  Catalog.t ->
  Native.view ->
  string option ->
  Domain.tile ->
  (Domain.public_unit option * bool) option
