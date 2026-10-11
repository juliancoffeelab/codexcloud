type tile = { id : string; terrain : string; x : float; y : float;
              points : string; clip_id : string; sprite_size : float;
              fill : string; name : string; floor : string option;
              frame : Art.frame option }
type dimensions = { width : float; height : float; display_width : float;
                    display_height : float }
type t = { key : string; map_id : string; cols : float; rows : float;
           width : float; height : float; pixel_scale : float; tiles : tile array }
external floors : string Js.Dict.t = "pixelDefaultFloors" [@@mel.module "@assets/pixel-art"]
external colors : string Js.Dict.t = "pixelFloorColors" [@@mel.module "@assets/pixel-art"]
external layouts : Js.Json.t array Js.Dict.t = "pixelTerrainLayouts" [@@mel.module "@assets/pixel-art"]
external freeze : 'a -> 'a = "freeze" [@@mel.scope "Object"]
external direction : Domain.hex_direction -> string = "%identity"
external code_at : string -> int -> int = "charCodeAt" [@@mel.send]
let optional encode = function None -> Js.Json.null | Some value -> encode value
let tile_json (tile : Domain.tile) = Js.Json.array [|Js.Json.string tile.id;
                                                     Js.Json.number tile.col; Js.Json.number tile.row; Js.Json.string tile.terrain;
                                                     optional Js.Json.string tile.floor;
                                                     optional (fun edges -> Js.Json.array (Array.map (fun edge -> Js.Json.string (direction edge)) edges)) tile.connections;
                                                     optional (fun edges -> Js.Json.array (Array.map (fun edge -> Js.Json.string (direction edge)) edges)) tile.deadEnds|]
let shape tile = Js.Json.stringify (tile_json tile)
let same_tile (a : Domain.tile) (b : Domain.tile) =
  a == b || (a.id = b.id && a.col = b.col && a.row = b.row &&
             a.terrain = b.terrain && a.floor = b.floor &&
             a.connections = b.connections && a.deadEnds = b.deadEnds)
let key ~map_id ~cols ~rows tiles =
  let ordered = Array.copy tiles in
  Array.sort (fun (a : Domain.tile) b -> let row = Float.compare a.row b.row in if row = 0 then Float.compare a.col b.col else row) ordered;
  Js.Json.stringify (Js.Json.array [|Js.Json.string map_id; Js.Json.number cols;
                                     Js.Json.number rows; Js.Json.array (Array.map tile_json ordered)|])
let mask edges = Option.fold ~none:0 ~some:(fun edges -> Array.fold_left (fun mask edge ->
    let bit = match direction edge with "e" -> 0 | "se" -> 1 | "sw" -> 2 | "w" -> 3 | "nw" -> 4 | "ne" -> 5 | _ -> 6 in
    mask lor (1 lsl bit)) 0 edges) edges
let connected_name (tile : Domain.tile) =
  let connections = mask tile.connections and dead_ends = mask tile.deadEnds in
  match tile.terrain with
  | "road" when dead_ends <> 0 -> "road_" ^ string_of_int connections ^ "-end-" ^ string_of_int dead_ends
  | "village" when connections = 0 -> "village_0"
  | ("road" | "village") as name when connections <> 0 && connections <> 9 -> name ^ "_" ^ string_of_int connections
  | ("bridge" | "ford") as name when connections = 18 -> name ^ "_nw-se"
  | ("bridge" | "ford") as name when connections = 36 -> name ^ "_ne-sw"
  | name -> name
(* Match JavaScript's for-of hashing: surrogate pairs contribute the leading
   UTF-16 code unit once, while isolated code units remain independent. *)
let layout_hash source =
  let hash = ref (Int32.of_string "0x811c9dc5") and index = ref 0 in
  while !index < String.length source do
    let code = code_at source !index in
    hash := Int32.mul (Int32.logxor !hash (Int32.of_int code)) 16777619l;
    index := !index + (if code >= 0xd800 && code <= 0xdbff && !index + 1 < String.length source &&
                          let next = code_at source (!index + 1) in next >= 0xdc00 && next <= 0xdfff then 2 else 1)
  done;
  let mixed = Int32.mul (Int32.logxor !hash (Int32.shift_right_logical !hash 16)) (Int32.of_string "0x85ebca6b") in
  let result = Int32.to_float (Int32.logxor mixed (Int32.shift_right_logical mixed 13)) in
  if result < 0. then result +. 4294967296. else result
let layout_name tile map_id =
  let name = connected_name tile in
  match Js.Dict.get layouts name with
  | Some layouts when Array.length layouts > 0 -> name ^ "_layout_" ^ string_of_int (int_of_float (mod_float (layout_hash (map_id ^ ":" ^ tile.id)) (float_of_int (Array.length layouts))))
  | _ -> name
let prepare ~map_id ~cols ~rows source artwork =
  let dimensions = Geometry.dimensions ~cols:(int_of_float cols) ~rows:(int_of_float rows) in
  let tiles = Array.map (fun (tile : Domain.tile) ->
      let point = Geometry.center ~col:(int_of_float tile.col) ~row:(int_of_float tile.row) in
      let name = layout_name tile map_id in
      let default_floor = Option.value (Js.Dict.get floors tile.terrain) ~default:"grass" in
      let floor = Option.value tile.floor ~default:default_floor in
      let fill = Option.value (Js.Dict.get colors floor) ~default:(Option.value (Js.Dict.get colors default_floor) ~default:"#8d9f4f") in
      freeze { id = tile.id; terrain = tile.terrain; x = point.x; y = point.y;
               points = Geometry.hex_points point.x point.y;
               clip_id = "hex-clip-" ^ Js.Float.toString tile.col ^ "-" ^ Js.Float.toString tile.row;
               sprite_size = Geometry.sprite_size; fill; name; floor = tile.floor;
               frame = Art.terrain_frame artwork ?floor:tile.floor name }) source in
  freeze {key = key ~map_id ~cols ~rows source; map_id; cols; rows;
          width = dimensions.width; height = dimensions.height; pixel_scale = Geometry.native_pixel_scale; tiles = freeze tiles}
let dimensions (plan : t) =
  let width = ceil (plan.width *. plan.pixel_scale) and height = ceil (plan.height *. plan.pixel_scale) in
  {width; height; display_width = width /. plan.pixel_scale; display_height = height /. plan.pixel_scale}
type context
type image
external save : context -> unit = "save" [@@mel.send]
external restore : context -> unit = "restore" [@@mel.send]
external begin_path : context -> unit = "beginPath" [@@mel.send]
external move_to : context -> float -> float -> unit = "moveTo" [@@mel.send]
external line_to : context -> float -> float -> unit = "lineTo" [@@mel.send]
external close_path : context -> unit = "closePath" [@@mel.send]
external fill : context -> unit = "fill" [@@mel.send]
external clip : context -> unit = "clip" [@@mel.send]
external set_fill : context -> string -> unit = "fillStyle" [@@mel.set]
external set_filter : context -> string -> unit = "filter" [@@mel.set]
external set_smoothing : context -> bool -> unit = "imageSmoothingEnabled" [@@mel.set]
external scale : context -> float -> float -> unit = "scale" [@@mel.send]
external draw : context -> image -> float -> float -> float -> float -> unit = "drawImage" [@@mel.send]
let configure context pixel_scale = set_smoothing context false; scale context pixel_scale pixel_scale
let paint context tiles images =
  Array.iter (fun (tile : tile) ->
      save context;
      try
        begin_path context;
        String.split_on_char ' ' tile.points |> List.iteri (fun index point ->
            match String.split_on_char ',' point with
            | [x; y] -> (if index = 0 then move_to else line_to) context (float_of_string x) (float_of_string y)
            | _ -> invalid_arg "Malformed terrain polygon");
        close_path context; set_fill context tile.fill; fill context; clip context;
        (match tile.frame with
         | Some frame -> (match Js.Dict.get images frame.url with
             | Some image -> set_filter context (if tile.terrain = "forest" then "brightness(0.93)" else "none");
               draw context image (tile.x -. tile.sprite_size /. 2.) (tile.y -. tile.sprite_size /. 2.) tile.sprite_size tile.sprite_size
             | None -> invalid_arg "Missing decoded terrain image")
         | None -> ());
        restore context
      with error -> restore context; raise error) tiles
