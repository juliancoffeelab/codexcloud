type point = { x : float; y : float }
type dimensions = { width : float; height : float }

let sprite_size = 70.
let native_pixel_scale = 64. /. sprite_size
let column_spacing = sprite_size *. 56. /. 64.
let row_spacing = sprite_size *. 3. /. 4.

(* Keep the origin on native pixel (39, 38), not only the spacing between tiles. *)
let center ~col ~row =
  {
    x =
      (39. +. (56. *. (float_of_int col +. (float_of_int (row mod 2) /. 2.)))) *. sprite_size /. 64.;
    y = (38. +. (48. *. float_of_int row)) *. sprite_size /. 64.;
  }

let hex_points x y =
  Array.init 6 (fun i ->
      let angle = float_of_int ((60 * i) - 30) *. Float.pi /. 180. in
      let js_text_part_0 = Js.Float.toString (x +. (column_spacing /. sqrt 3. *. cos angle)) in
      let js_text_part_2 = Js.Float.toString (y +. (sprite_size /. 2. *. sin angle)) in
      {j|$(js_text_part_0),$(js_text_part_2)|j})
  |> Array.to_list
  |> fun values -> Js.Array.join ~sep:{js| |js} (Array.of_list values)

let dimensions ~cols ~rows =
  {
    width = 758. +. (column_spacing *. float_of_int (cols - 11));
    height = 452. +. (row_spacing *. float_of_int (rows - 7));
  }

let preview_dimensions ~cols ~rows =
  {
    width = 85. +. (column_spacing *. (float_of_int cols -. 0.5));
    height = 84. +. (row_spacing *. float_of_int (rows - 1));
  }

let preview_color = function
  | matched_text_0
    when matched_text_0 = {js|monastery|js}
         || matched_text_0 = {js|church|js}
         || matched_text_0 = {js|castle-wall|js}
         || matched_text_0 = {js|castle-bell|js} ->
      {js|#b9b597|js}
  | matched_text_1
    when matched_text_1 = {js|graveyard|js}
         || matched_text_1 = {js|ritual-star|js}
         || matched_text_1 = {js|ritual-spent|js}
         || matched_text_1 = {js|burned-village|js} ->
      {js|#8d9f4f|js}
  | matched_text_2 when matched_text_2 = {js|mountain|js} || matched_text_2 = {js|mountain-mine|js}
    ->
      {js|#727063|js}
  | matched_text_3 when matched_text_3 = {js|cave-passage|js} -> {js|#505358|js}
  | matched_text_4 when matched_text_4 = {js|cave-wall|js} -> {js|#2c3037|js}
  | matched_text_5 when matched_text_5 = {js|house|js} -> {js|#d4b476|js}
  | matched_text_6 when matched_text_6 = {js|landing|js} || matched_text_6 = {js|ferry|js} ->
      {js|#78a4a6|js}
  | matched_text_7 when matched_text_7 = {js|signpost|js} -> {js|#8fb337|js}
  | matched_text_8 when matched_text_8 = {js|marsh|js} -> {js|#809678|js}
  | matched_text_9 when matched_text_9 = {js|ruins|js} -> {js|#a2a18c|js}
  | matched_text_10 when matched_text_10 = {js|fort|js} -> {js|#b4a277|js}
  | matched_text_11 when matched_text_11 = {js|grove|js} -> {js|#4d884e|js}
  | matched_text_12 when matched_text_12 = {js|plains|js} -> {js|#8d9f4f|js}
  | matched_text_13 when matched_text_13 = {js|road|js} -> {js|#8fb337|js}
  | matched_text_14 when matched_text_14 = {js|forest|js} -> {js|#536e3d|js}
  | matched_text_15 when matched_text_15 = {js|hill|js} -> {js|#85866c|js}
  | matched_text_16 when matched_text_16 = {js|water|js} -> {js|#407899|js}
  | kind when kind = {js|shallow-water|js} -> {js|#44ada8|js}
  | matched_text_17 when matched_text_17 = {js|bridge|js} -> {js|#b3ab8d|js}
  | matched_text_18 when matched_text_18 = {js|ford|js} -> {js|#78a4a6|js}
  | matched_text_19 when matched_text_19 = {js|village|js} -> {js|#d4b476|js}
  | matched_text_20 when matched_text_20 = {js|keep|js} -> {js|#b9b597|js}
  | _ -> {js|#8d9f4f|js}
