type point = { x : float; y : float }
type dimensions = { width : float; height : float }
let center ~col ~row =
  { x = 43. +. 61. *. (float_of_int col +. float_of_int (row mod 2) /. 2.);
    y = 42. +. 53. *. float_of_int row }
let hex_points x y =
  Array.init 6 (fun i ->
      let angle = float_of_int (60 * i - 30) *. Float.pi /. 180. in
      Js.Float.toString (x +. 35. *. cos angle) ^ "," ^
      Js.Float.toString (y +. 35. *. sin angle)) |> Array.to_list |> String.concat " "
let dimensions ~cols ~rows =
  { width = 758. +. 61. *. float_of_int (cols - 11);
    height = 452. +. 53. *. float_of_int (rows - 7) }
let color = function
  | "marsh" -> "#536d59" | "ruins" -> "#697064" | "fort" -> "#7a7659"
  | "grove" -> "#426b47" | "plains" -> "#39564f" | "forest" -> "#244c43"
  | "hill" -> "#5a6050" | "water" | "bridge" | "ford" -> "#1b3c52"
  | "village" -> "#69745b" | "keep" -> "#526d74" | _ -> "#39564f"
let preview_color = function
  | "marsh" -> "#809678" | "ruins" -> "#a2a18c" | "fort" -> "#b4a277"
  | "grove" -> "#4d884e" | "plains" -> "#8d9f4f" | "forest" -> "#536e3d"
  | "hill" -> "#85866c" | "water" -> "#407899" | "bridge" -> "#b3ab8d"
  | "ford" -> "#78a4a6" | "village" -> "#d4b476" | "keep" -> "#b9b597"
  | _ -> "#8d9f4f"

