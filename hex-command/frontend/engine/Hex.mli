val key : Domain.hex -> string
val of_tile : Domain.tile -> Domain.hex
val equal : Domain.hex -> Domain.hex -> bool
val matches : Domain.tile -> Domain.Tile_ref.t -> bool
