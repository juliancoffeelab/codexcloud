type entry

val create : Domain.saved_game -> Js.Json.t -> string -> float -> entry
val checkpoint : entry -> Domain.saved_game
val unit_id : entry -> string
val round : entry -> float
val restore : Js.Json.t -> entry list
val saved : Js.Json.t -> entry list -> Js.Json.t
