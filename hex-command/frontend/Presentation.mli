(** Pure display policy over detached native publications. All legality, movement
    costs, recruitment destinations and combat forecasts come from Rust's
    inspection; these projections never call or mutate a live battle. *)
type ability = { name : string; description : string; facts : string array }
type fact = { label : string; value : float }
type terrain_effect = { terrain : string; name : string; label : string;
                        facts : fact array; class_name : string }
type unit_details = { unit : Domain.public_unit; definition : Domain.unit_rule;
                      terrain_name : string; action_status : string;
                      promotions : Domain.unit_rule array; ability : ability;
                      terrain_effects : terrain_effect array }
type panel = Hidden | Outcome of string * string | Details of unit_details
           | Combat of Domain.public_unit * Domain.public_unit * string
(** Selection, winner and target policy; the combat key resets optional details
    for each published render revision. *)
val unit_panel : Catalog.t -> Native.view -> selected:string option ->
  target:string option -> revision:float -> panel

type forecast = { side : Domain.side; remaining_hp : float; current_hp : float;
                  maximum_hp : float; health_label : string; current_percent : float;
                  remaining_percent : float; impact_label : string }
type calculation = { description : string; equation : string }
type combat_result = { player : forecast; enemy : forecast; attack : calculation; retaliation : calculation }
type attack = { weapon : Domain.weapon; button_label : string;
                result : combat_result option }
(** One entry per authored weapon. [result = None] is an unavailable attack;
    forecasts and equations use the native legal current-position preview. *)
val attacks : Catalog.t -> Native.view -> Domain.public_unit -> Domain.public_unit -> attack array
val damage_equation : Domain.damage_detail -> string
val signed_value : float -> string
val terrain_damage_facts : float array array -> fact array
val ability : Catalog.t -> Domain.public_unit -> ability

type modifier = { name : string; label : string; class_name : string }
val armor : Catalog.t -> Domain.public_unit -> modifier array
val weapon_range : Domain.weapon -> string
val damage_name : Domain.damage_type -> string

type recruit = { definition : Domain.unit_rule; enabled : bool }
type recruitment = { options : recruit array; hint : string }
type terrain_panel = { tile : Domain.tile; name : string; description : string;
                       ownership : (string * string) option; facts : string array;
                       recruitment : recruitment option }
(** Inspection facts and recruitment choices share the same revision and
    selection. A selected unit suppresses recruitment into inspected ground. *)
val terrain_panel : Catalog.t -> Native.view -> selected:string option ->
  inspected:string option -> terrain_panel option

type tile_display = { tile : Domain.tile; occupant : Domain.public_unit option;
                      classes : string; label : string; owner_color : string;
                      health_width : float }
(** Accessible tile descriptions and CSS states. Artwork and geometry are
    rendered separately, so unit movement cannot replace the terrain layer. *)
val battlefield : Catalog.t -> Native.view -> selected:string option ->
  inspected:string option -> tile_display array

(** Feedback describes publication differences without DOM or audio effects. *)
type motion = Move | Attack | Recruit
type changes = { tile_effects : (string * motion) array; captured_tiles : string array;
                 turn_changed : bool; new_winner : Domain.side option;
                 attacked : bool; recruited : bool; advanced : bool;
                 moved_unit : Domain.public_unit option; attacker : Domain.public_unit option }
val changes : Domain.state -> Domain.state -> changes
