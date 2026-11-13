type t
type owner

external owner : unit -> owner = "Object" [@@mel.new]
external named_owner : string -> owner = "%identity"
external game_owner : Native.view -> owner = "%identity"
external combat_owner : Domain.combat_animation -> owner = "%identity"

type interaction = {
  owner : owner option;
  attack_type : string option;
  attack_id : string option;
  attack_sound : string option;
  attack_category : string option;
  attack_damage_type : string option;
  unit_type : string option;
  terrain : string option;
}

val interaction : interaction
val sound_for_interaction : interaction -> string

type readiness = Audio_state.readiness

val readiness : readiness Observable.reader
val hud_readiness : readiness Observable.reader
val report_readiness : t -> bool -> string option -> unit

type sample = { family : string; key : string; url : string }

val samples : sample array
val action_families : string array
val battle_sounds : string array -> string array -> string array

type channel = Feedback | Combat
type timing = { impact_at : float; end_at : float; played : bool }

val create : unit -> t
val close : t -> unit
val enabled : t -> bool
val set_enabled : t -> bool -> unit
val attach : t -> Phaser.game -> (unit -> bool Js.Promise.t) -> unit -> unit
val sound_ready : t -> Phaser.game -> unit
val prepare : t -> string array -> unit
val prepare_hud : t -> unit Js.Promise.t

type hud_cue = Action | Choice | Disclosure | Human | Woodland | Skeletons | Merfolk

val faction_hud_cue : string -> hud_cue
val play_hud : ?cue:hud_cue -> t -> bool Js.Promise.t
val unlock : t -> trusted:bool -> bool Js.Promise.t
val cancel : t -> unit
val stop_owner : ?key:string -> t -> owner -> unit
val pending : t -> owner -> bool
val watch : t -> owner -> (unit -> unit) -> unit -> unit
val play : ?owner:owner -> t -> string -> bool Js.Promise.t
val play_interaction : ?owner:owner -> t -> interaction -> bool Js.Promise.t

val schedule_cue :
  ?use_impact:bool ->
  ?pitch_spread:float ->
  ?owner:owner ->
  ?key:string ->
  ?replace:bool ->
  t ->
  string ->
  float ->
  channel ->
  timing

val schedule_movement : ?owner:owner -> t -> float -> timing
val schedule_death : ?owner:owner -> ?key:string -> t -> string -> float -> timing
