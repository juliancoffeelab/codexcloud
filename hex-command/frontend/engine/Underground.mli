type mask = { x : float; y : float; width : int; height : int; step : float; data : float array }

val key : Domain.hex array -> string
val get : Domain.hex array -> mask option
val sample : mask option -> float -> float -> float
