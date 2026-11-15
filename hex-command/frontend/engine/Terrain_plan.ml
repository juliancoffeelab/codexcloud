type tile = Terrain_painter.tile = {
  col : float;
  row : float;
  id : string;
  terrain : string;
  x : float;
  y : float;
  points : string;
  sprite_size : float;
  sprite_x : float;
  sprite_y : float;
  sprite_width : float;
  sprite_height : float;
  role : Domain.Terrain_role.t;
  fill : string;
  name : string;
  floor : string option;
  frame : Art.frame option;
}

type dimensions = Terrain_painter.dimensions = {
  width : float;
  height : float;
  display_width : float;
  display_height : float;
}

type t = {
  key : string;
  map_id : string;
  cols : float;
  rows : float;
  width : float;
  height : float;
  pixel_scale : float;
  tiles : tile array;
}

external layouts : Js.Json.t array Js.Dict.t = "pixelTerrainLayouts"
[@@mel.module "@assets/pixel-art"]

external has_own : 'a Js.Dict.t -> string -> bool = "hasOwn" [@@mel.scope "Object"]

let lookup dictionary key = if has_own dictionary key then Js.Dict.get dictionary key else None
let direction = Enum.string_of_hex_direction

external code_at : string -> int -> int = "charCodeAt" [@@mel.send]

let optional encode = function None -> Js.Json.null | Some value -> encode value

let tile_json (tile : Domain.tile) =
  Js.Json.array
    [|
      Js.Json.string tile.id;
      Js.Json.number (Int32.to_float tile.col);
      Js.Json.number (Int32.to_float tile.row);
      Js.Json.string tile.terrain;
      optional Js.Json.string tile.floor;
      optional
        (fun edges -> Js.Json.array (Array.map (fun edge -> Js.Json.string (direction edge)) edges))
        tile.connections;
      optional
        (fun edges -> Js.Json.array (Array.map (fun edge -> Js.Json.string (direction edge)) edges))
        tile.dead_ends;
      optional
        (fun edges -> Js.Json.array (Array.map (fun edge -> Js.Json.string (direction edge)) edges))
        tile.walls;
    |]

let same_tile (a : Domain.tile) (b : Domain.tile) =
  a == b
  || a.id = b.id && a.col = b.col && a.row = b.row && a.terrain = b.terrain && a.floor = b.floor
     && a.connections = b.connections && a.dead_ends = b.dead_ends && a.walls = b.walls

let key ~map_id ~cols ~rows tiles =
  let ordered = Array.copy tiles in
  Array.sort
    (fun (a : Domain.tile) b ->
      let row = Int32.compare a.row b.row in
      if row = 0 then Int32.compare a.col b.col else row)
    ordered;
  Js.Json.stringify
    (Js.Json.array
       [|
         Js.Json.string map_id;
         Js.Json.number cols;
         Js.Json.number rows;
         Js.Json.array (Array.map tile_json ordered);
       |])

let mask edges =
  Option.fold ~none:0
    ~some:(fun edges ->
      Array.fold_left
        (fun mask edge ->
          let bit =
            match direction edge with
            | matched_text_0 when matched_text_0 = {js|e|js} -> 0
            | matched_text_1 when matched_text_1 = {js|se|js} -> 1
            | matched_text_2 when matched_text_2 = {js|sw|js} -> 2
            | matched_text_3 when matched_text_3 = {js|w|js} -> 3
            | matched_text_4 when matched_text_4 = {js|nw|js} -> 4
            | matched_text_5 when matched_text_5 = {js|ne|js} -> 5
            | _ -> 6
          in
          mask lor (1 lsl bit))
        0 edges)
    edges

let connection_name terrain connections dead_ends walls =
  match terrain with
  | matched_text_6 when matched_text_6 = {js|castle-wall|js} || matched_text_6 = {js|castle-bell|js}
    ->
      let js_text_part_2 = string_of_int walls in
      {j|$(terrain)_$(js_text_part_2)|j}
  | matched_text_7 when matched_text_7 = {js|road|js} && dead_ends <> 0 ->
      let js_text_part_1 = string_of_int connections in
      let js_text_part_3 = string_of_int dead_ends in
      {j|road_$(js_text_part_1)-end-$(js_text_part_3)|j}
  | matched_text_8
    when matched_text_8 = {js|signpost|js} || matched_text_8 = {js|road|js}
         || matched_text_8 = {js|village|js} || matched_text_8 = {js|bridge|js}
         || matched_text_8 = {js|ford|js} || matched_text_8 = {js|landing|js}
         || matched_text_8 = {js|ferry|js}
         || matched_text_8 = {js|cave-passage|js} ->
      let js_text_part_2 = string_of_int connections in
      {j|$(terrain)_$(js_text_part_2)|j}
  | _ -> terrain

let connected_name (tile : Domain.tile) =
  connection_name tile.terrain (mask tile.connections) (mask tile.dead_ends) (mask tile.walls)

(* Match JavaScript's for-of hashing: surrogate pairs contribute the leading
   UTF-16 code unit once, while isolated code units remain independent. *)
let layout_hash source =
  let hash = ref (Int32.of_string {js|0x811c9dc5|js}) and index = ref 0 in
  while !index < Js.String.length source do
    let code = code_at source !index in
    hash := Int32.mul (Int32.logxor !hash (Int32.of_int code)) 16777619l;
    index :=
      !index
      +
      if
        code >= 0xd800 && code <= 0xdbff
        && !index + 1 < Js.String.length source
        &&
        let next = code_at source (!index + 1) in
        next >= 0xdc00 && next <= 0xdfff
      then 2
      else 1
  done;
  let mixed =
    Int32.mul
      (Int32.logxor !hash (Int32.shift_right_logical !hash 16))
      (Int32.of_string {js|0x85ebca6b|js})
  in
  let result = Int32.to_float (Int32.logxor mixed (Int32.shift_right_logical mixed 13)) in
  if result < 0. then result +. 4294967296. else result

let layout_name tile map_id =
  let name = connected_name tile in
  match lookup layouts name with
  | Some layouts when Array.length layouts > 0 ->
      let js_text_part_2 =
        string_of_int
          (int_of_float
             (mod_float
                (layout_hash
                   (let js_text_part_2 = tile.id in
                    {j|$(map_id):$(js_text_part_2)|j}))
                (float_of_int (Array.length layouts))))
      in
      {j|$(name)_layout_$(js_text_part_2)|j}
  | _ -> name

let prepare ~roles ~map_id ~cols ~rows source artwork =
  let dimensions = Geometry.dimensions ~cols:(int_of_float cols) ~rows:(int_of_float rows) in
  let tiles =
    Array.map
      (fun (tile : Domain.tile) ->
        let point = Geometry.center ~col:(Int32.to_int tile.col) ~row:(Int32.to_int tile.row) in
        let name = layout_name tile map_id in
        let fill = Geometry.preview_color tile.terrain in
        let frame = Art.terrain_frame artwork ?floor:tile.floor name in
        let scale = Geometry.sprite_size /. 64. in
        let width, height, anchor_x, anchor_y =
          match frame with
          | Some frame -> (frame.width, frame.height, frame.anchor_x, frame.anchor_y)
          | None -> (64., 64., 32., 32.)
        in
        let role =
          match Js.Dict.get roles tile.terrain with
          | Some role -> role
          | None ->
              invalid_arg
                (let js_text_part_1 = tile.terrain in
                 {j|Missing terrain role: $(js_text_part_1)|j})
        in
        {
          col = Int32.to_float tile.col;
          row = Int32.to_float tile.row;
          id = tile.id;
          terrain = tile.terrain;
          x = point.x;
          y = point.y;
          points = Geometry.hex_points point.x point.y;
          sprite_size = Geometry.sprite_size;
          sprite_x = point.x -. (anchor_x *. scale);
          sprite_y = point.y -. (anchor_y *. scale);
          sprite_width = width *. scale;
          sprite_height = height *. scale;
          role;
          fill;
          name;
          floor = tile.floor;
          frame;
        })
      source
  in
  {
    key = key ~map_id ~cols ~rows source;
    map_id;
    cols;
    rows;
    width = dimensions.width;
    height = dimensions.height;
    pixel_scale = Geometry.native_pixel_scale;
    tiles;
  }

let dimensions (plan : t) =
  Terrain_painter.dimensions ~width:plan.width ~height:plan.height ~pixel_scale:plan.pixel_scale

type context = Terrain_painter.context
type image = Terrain_painter.image

let configure = Terrain_painter.configure
let paint = Terrain_painter.paint

external radix : float -> int -> string = "toString" [@@mel.send]

let asset_key source =
  let first = ref (Int32.of_string {js|0x811c9dc5|js})
  and second = ref (Int32.of_string {js|0x85ebca77|js}) in
  for index = 0 to Js.String.length source - 1 do
    let code = Int32.of_int (code_at source index) in
    first := Int32.mul (Int32.logxor !first code) 16777619l;
    second := Int32.mul (Int32.logxor !second code) (Int32.of_string {js|0xc2b2ae3d|js})
  done;
  let hex value =
    let value = Int32.to_float value in
    let value = if value < 0. then value +. 4294967296. else value in
    let value = radix value 16 in
    let js_text_part_0 = Js.String.repeat ~count:(8 - Js.String.length value) {js|0|js} in
    {j|$(js_text_part_0)$(value)|j}
  in
  let js_text_part_0 = hex !first in
  let js_text_part_1 = hex !second in
  {j|$(js_text_part_0)$(js_text_part_1)|j}
