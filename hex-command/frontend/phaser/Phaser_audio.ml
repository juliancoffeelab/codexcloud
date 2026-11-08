external context_state : Phaser.audio_context -> string = "state" [@@mel.get]
external resume_context : Phaser.audio_context -> unit Js.Promise.t = "resume" [@@mel.send]

external create_context : Phaser.sound_manager -> Phaser.game -> Phaser.audio_context
  = "createAudioContext"
[@@mel.send]

external set_context : Phaser.sound_manager -> Phaser.audio_context -> unit = "setAudioContext"
[@@mel.send]

external clear_context : Phaser.sound_manager -> Phaser.audio_context Js.Nullable.t -> unit
  = "context"
[@@mel.set]

external unlocked : Phaser.sound_manager -> bool -> unit = "unlocked" [@@mel.set]
external locked : Phaser.sound_manager -> bool -> unit = "locked" [@@mel.set]
external context_time : Phaser.audio_context -> float = "currentTime" [@@mel.get]
external now : unit -> float = "now" [@@mel.scope "performance"]

external add_sound :
  Phaser.sound_manager -> string -> < volume : float ; rate : float > Js.t -> Phaser.sound = "add"
[@@mel.send]

external play_delay : Phaser.sound -> < delay : float > Js.t -> bool = "play" [@@mel.send]
external start_time : Phaser.sound -> float = "startTime" [@@mel.get]

type gain_node
type gain_parameter

external volume_node : Phaser.sound -> gain_node Js.Undefined.t = "volumeNode" [@@mel.get]
external gain : gain_node -> gain_parameter = "gain" [@@mel.get]
external value_at : gain_parameter -> float -> float -> unit = "setValueAtTime" [@@mel.send]
external ramp : gain_parameter -> float -> float -> unit = "linearRampToValueAtTime" [@@mel.send]

type output = {
  context_time : float option; [@mel.as "contextTime"]
  performance_time : float option; [@mel.as "performanceTime"]
}

external has_output : Phaser.audio_context -> unit Js.Undefined.t = "getOutputTimestamp" [@@mel.get]
external output : Phaser.audio_context -> output = "getOutputTimestamp" [@@mel.send]
