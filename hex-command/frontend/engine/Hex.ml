let key (hex : Domain.hex) =
  let js_text_part_0 = Int32.to_string hex.col in
  let js_text_part_2 = Int32.to_string hex.row in
  {j|$(js_text_part_0),$(js_text_part_2)|j}

let of_tile (tile : Domain.tile) : Domain.hex = { col = tile.col; row = tile.row }
let equal (left : Domain.hex) (right : Domain.hex) = left.col = right.col && left.row = right.row

let matches (tile : Domain.tile) = function
  | Domain.Tile_ref.Location name -> tile.location = Some name
  | Domain.Tile_ref.Hex hex -> equal (of_tile tile) hex
  | Domain.Tile_ref.Binding _ -> false
