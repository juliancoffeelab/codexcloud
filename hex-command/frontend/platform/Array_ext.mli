val filter : ('a -> bool) -> 'a array -> 'a array
val filter_map : ('a -> 'b option) -> 'a array -> 'b array
val equal : ('a -> 'a -> bool) -> 'a array -> 'a array -> bool
val append_value : 'a array -> 'a -> int
