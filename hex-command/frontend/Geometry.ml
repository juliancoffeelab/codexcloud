type point = { x : float; y : float }
type dimensions = { width : float; height : float }
let sprite_size = 70.
let native_pixel_scale = 64. /. sprite_size
let column_spacing = sprite_size *. sqrt 3. /. 2.
let row_spacing = sprite_size *. 3. /. 4.
let center ~col ~row =
  { x = 43. +. column_spacing *. (float_of_int col +. float_of_int (row mod 2) /. 2.);
    y = 42. +. row_spacing *. float_of_int row }
let hex_points x y =
  Array.init 6 (fun i ->
      let angle = float_of_int (60 * i - 30) *. Float.pi /. 180. in
      Js.Float.toString (x +. sprite_size /. 2. *. cos angle) ^ "," ^
      Js.Float.toString (y +. sprite_size /. 2. *. sin angle)) |> Array.to_list |> String.concat " "
let dimensions ~cols ~rows =
  { width = 758. +. column_spacing *. float_of_int (cols - 11);
    height = 452. +. row_spacing *. float_of_int (rows - 7) }
let preview_dimensions ~cols ~rows =
  { width = 85. +. column_spacing *. (float_of_int cols -. 0.5);
    height = 84. +. row_spacing *. float_of_int (rows - 1) }
let preview_color = function
  | "marsh" -> "#809678" | "ruins" -> "#a2a18c" | "fort" -> "#b4a277"
  | "grove" -> "#4d884e" | "plains" -> "#8d9f4f" | "road" -> "#8fb337" | "forest" -> "#536e3d"
  | "hill" -> "#85866c" | "water" -> "#407899" | "bridge" -> "#b3ab8d"
  | "ford" -> "#78a4a6" | "village" -> "#d4b476" | "keep" -> "#b9b597"
  | _ -> "#8d9f4f"

