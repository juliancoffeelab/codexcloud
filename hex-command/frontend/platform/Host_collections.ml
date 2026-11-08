(* Native identity collections match Phaser/JavaScript ownership. In particular,
   keys containing live scene objects must never be structurally hashed. *)
module Map = struct
  type ('key, 'value) t
  type 'value iterator

  external create : unit -> ('key, 'value) t = "Map" [@@mel.new]
  external read : ('key, 'value) t -> 'key -> 'value Js.Undefined.t = "get" [@@mel.send]
  external set : ('key, 'value) t -> 'key -> 'value -> ('key, 'value) t = "set" [@@mel.send]
  external has : ('key, 'value) t -> 'key -> bool = "has" [@@mel.send]
  external delete : ('key, 'value) t -> 'key -> bool = "delete" [@@mel.send]
  external clear : ('key, 'value) t -> unit = "clear" [@@mel.send]
  external size : ('key, 'value) t -> int = "size" [@@mel.get]

  external each : ('key, 'value) t -> (('value -> 'key -> unit)[@u]) -> unit = "forEach"
  [@@mel.send]

  external values : ('key, 'value) t -> 'value iterator = "values" [@@mel.send]
  external keys : ('key, 'value) t -> 'key iterator = "keys" [@@mel.send]
  external array : 'value iterator -> 'value array = "from" [@@mel.scope "Array"]

  let get map key = Js.Undefined.toOption (read map key)
  let values_array map = array (values map)
  let keys_array map = array (keys map)
end

module Set = struct
  type 'value t
  type 'value iterator

  external create : unit -> 'value t = "Set" [@@mel.new]
  external add : 'value t -> 'value -> 'value t = "add" [@@mel.send]
  external has : 'value t -> 'value -> bool = "has" [@@mel.send]
  external delete : 'value t -> 'value -> bool = "delete" [@@mel.send]
  external clear : 'value t -> unit = "clear" [@@mel.send]
  external size : 'value t -> int = "size" [@@mel.get]
  external each : 'value t -> (('value -> unit)[@u]) -> unit = "forEach" [@@mel.send]
  external values : 'value t -> 'value iterator = "values" [@@mel.send]
  external array : 'value iterator -> 'value array = "from" [@@mel.scope "Array"]

  let values_array set = array (values set)
end

module Weak_map = struct
  type ('key, 'value) t

  external create : unit -> ('key, 'value) t = "WeakMap" [@@mel.new]
  external read : ('key, 'value) t -> 'key -> 'value Js.Undefined.t = "get" [@@mel.send]
  external set : ('key, 'value) t -> 'key -> 'value -> ('key, 'value) t = "set" [@@mel.send]
  external has : ('key, 'value) t -> 'key -> bool = "has" [@@mel.send]
  external delete : ('key, 'value) t -> 'key -> bool = "delete" [@@mel.send]

  let get map key = Js.Undefined.toOption (read map key)
end

external sort : 'a array -> (('a -> 'a -> int)[@u]) -> 'a array = "sort" [@@mel.send]
