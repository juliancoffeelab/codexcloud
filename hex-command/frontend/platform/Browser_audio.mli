type context
type buffer
type source
type gain
type destination
type parameter
type response
type array_buffer
type controller
type signal

external context : < latencyHint : string > Js.t -> context = "AudioContext" [@@mel.new]
external state : context -> string = "state" [@@mel.get]
external resume : context -> unit Js.Promise.t = "resume" [@@mel.send]
external close_context : context -> unit Js.Promise.t = "close" [@@mel.send]
external decode : context -> array_buffer -> buffer Js.Promise.t = "decodeAudioData" [@@mel.send]
external new_controller : unit -> controller = "AbortController" [@@mel.new]
external signal : controller -> signal = "signal" [@@mel.get]
external abort : controller -> unit = "abort" [@@mel.send]
external deadline : int -> signal = "timeout" [@@mel.scope "AbortSignal"]
external any_signal : signal array -> signal = "any" [@@mel.scope "AbortSignal"]
external fetch : string -> < signal : signal > Js.t -> response Js.Promise.t = "fetch"
external ok : response -> bool = "ok" [@@mel.get]
external status : response -> int = "status" [@@mel.get]
external status_text : response -> string = "statusText" [@@mel.get]
external array_buffer : response -> array_buffer Js.Promise.t = "arrayBuffer" [@@mel.send]
external create_source : context -> source = "createBufferSource" [@@mel.send]
external create_gain : context -> gain = "createGain" [@@mel.send]
external destination : context -> destination = "destination" [@@mel.get]
external gain_parameter : gain -> parameter = "gain" [@@mel.get]
external gain_value : parameter -> float -> unit = "value" [@@mel.set]
external source_buffer : source -> buffer -> unit = "buffer" [@@mel.set]
external connect_source : source -> gain -> unit = "connect" [@@mel.send]
external connect_gain : gain -> destination -> unit = "connect" [@@mel.send]
external disconnect_source : source -> unit = "disconnect" [@@mel.send]
external disconnect_gain : gain -> unit = "disconnect" [@@mel.send]
external stop_source : source -> unit = "stop" [@@mel.send]
external start_source : source -> unit = "start" [@@mel.send]
external ended : source -> ((unit -> unit)[@u]) -> unit = "onended" [@@mel.set]
external trusted : Dom.event -> bool = "isTrusted" [@@mel.get]

external listen : string -> (Dom.event -> unit) -> < capture : bool ; passive : bool > Js.t -> unit
  = "addEventListener"
[@@mel.scope "document"]

external unlisten : string -> (Dom.event -> unit) -> bool -> unit = "removeEventListener"
[@@mel.scope "document"]
