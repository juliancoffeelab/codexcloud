type profile = Natural | River | River_valley | Lantern_crossing | Lantern_wetlands

let parse = function
  | "natural" -> Natural
  | "river" -> River
  | "river-valley" -> River_valley
  | "lantern-crossing" -> Lantern_crossing
  | "lantern-wetlands" -> Lantern_wetlands
  | _ -> invalid_arg {js|Unknown terrain profile. See --help for terrain and campaign profiles.|js}

let plains = ({js|plains|js}, {js|green-plain|js})
let forest = ({js|forest|js}, {js|green-forest|js})
let hill = ({js|hill|js}, {js|green-plain|js})
let mountain = ({js|mountain|js}, {js|mountain-rock|js})
let marsh = ({js|marsh|js}, {js|marsh|js})
let water = ({js|water|js}, {js|dark-water|js})
let ford = ({js|ford|js}, {js|shallow-water|js})
let bridge = ({js|bridge|js}, {js|dark-water|js})
let road = ({js|road|js}, {js|green-plain|js})

let upland height wetness =
  if height > 0.48 then mountain
  else if height > 0.24 then hill
  else if wetness > 0.08 then forest
  else plains

type crossing_kind = Shallow | Bridged
type crossing = { left : int; right : int; row : int; kind : crossing_kind }
type channel = Water | Crossing of crossing_kind | Approach of string list | Bank | Island | Land

type plan = {
  tile : col:int -> row:int -> string * string;
  connections : col:int -> row:int -> string list;
  validate : (int * int, string) Hashtbl.t -> unit;
}

let crossing_tile = function Shallow -> ford | Bridged -> bridge
let across = [ {js|e|js}; {js|w|js} ]

let neighbors (col, row) =
  let west = col - 1 + (row mod 2) in
  [
    (col - 1, row);
    (col + 1, row);
    (west, row - 1);
    (west + 1, row - 1);
    (west, row + 1);
    (west + 1, row + 1);
  ]

let is_channel channel ~col ~row =
  match channel ~col ~row with
  | Water | Crossing _ -> true
  | Approach _ | Bank | Island | Land -> false

let waterway_terrain terrain =
  List.mem terrain [ {js|water|js}; {js|shallow-water|js}; {js|ford|js}; {js|bridge|js} ]

let floodplain ~channel ~fields ~tiles ~editable ~lowland =
  let marshes = Hashtbl.create 64 in
  let frontier = Queue.create () in
  Hashtbl.iter
    (fun ((col, row) as point) terrain ->
      if is_channel channel ~col ~row && (editable point || waterway_terrain terrain) then
        Queue.add (point, 0) frontier)
    tiles;
  (* Flood only contiguous low ground, up to three hexes inland from the channel. *)
  while not (Queue.is_empty frontier) do
    let point, distance = Queue.take frontier in
    if distance < 3 then
      List.iter
        (fun ((col, row) as neighbor) ->
          if
            editable neighbor
            && (not (Hashtbl.mem marshes neighbor))
            && lowland row
            && channel ~col ~row = Land
          then
            let height, moisture = fields ~col ~row in
            if height < -0.12 && moisture > -0.24 then (
              Hashtbl.add marshes neighbor ();
              Queue.add (neighbor, distance + 1) frontier))
        (neighbors point)
  done;
  marshes

let validate_river ~rows ~channel ~crossings tiles =
  let waterway = Hashtbl.create 64 in
  Hashtbl.iter
    (fun (col, row) terrain ->
      if is_channel channel ~col ~row && waterway_terrain terrain then
        Hashtbl.add waterway (col, row) ())
    tiles;
  let frontier = Queue.create () in
  let visit point =
    if Hashtbl.mem waterway point then (
      Hashtbl.remove waterway point;
      Queue.add point frontier)
  in
  let starts =
    Hashtbl.fold
      (fun ((_, row) as point) () starts -> if row = 0 then point :: starts else starts)
      waterway []
  in
  let reached_bottom = ref false in
  (* Start from one headwater so disconnected strips on a clipped board fail. *)
  (match starts with [] -> () | point :: _ -> visit point);
  while not (Queue.is_empty frontier) do
    let ((_, row) as point) = Queue.take frontier in
    if row = rows - 1 then reached_bottom := true;
    List.iter visit (neighbors point)
  done;
  if (not !reached_bottom) || Hashtbl.length waterway > 0 then
    invalid_arg
      {js|The template interrupts the river. Use a footprint with a clear channel between the north and south edges.|js};
  let clear_bank point =
    match Hashtbl.find_opt tiles point with
    | Some terrain ->
        List.mem terrain
          [
            fst plains;
            fst forest;
            fst hill;
            fst marsh;
            fst road;
            {js|village|js};
            {js|keep|js};
            {js|fort|js};
            {js|grove|js};
            {js|ruins|js};
          ]
    | None -> false
  in
  List.iter
    (fun { left; right; row; kind } ->
      let blocked () =
        invalid_arg
          {js|The template blocks a planned crossing or its banks. Use clear terrain around the river crossings.|js}
      in
      for col = left to right do
        if Hashtbl.find_opt tiles (col, row) <> Some (fst (crossing_tile kind)) then blocked ()
      done;
      if not (clear_bank (left - 1, row) && clear_bank (right + 1, row)) then blocked ())
    crossings

let river ~seed ~cols ~rows ~profile ~fields ~tiles ~editable =
  let island = profile = Lantern_crossing and valley = profile = River_valley in
  let min_cols, min_rows = if island then (13, 9) else if valley then (11, 12) else (7, 5) in
  (if cols < min_cols || rows < min_rows then
     let min_cols = string_of_int min_cols and min_rows = string_of_int min_rows in
     invalid_arg {j|The river profile requires at least $(min_cols) columns and $(min_rows) rows.|j});
  let meander = Draft_noise.simplex (Int32.logxor seed 0x3a76281dl) in
  let progress row = float_of_int row /. float_of_int (rows - 1) in
  let centers = Array.make rows (cols / 2) in
  Array.iteri
    (fun row _ ->
      let offset = meander 0.61 (1.8 *. progress row) *. float_of_int cols *. 0.14 in
      let target = (cols / 2) + int_of_float (Float.round offset) in
      centers.(row) <-
        (if row = 0 then target
         else
           let previous = centers.(row - 1) in
           if valley then
             (* A single-hex channel must step to an adjacent hex in the next row. *)
             let west = previous - (row mod 2) in
             max west (min (west + 1) target)
           else max (previous - 1) (min (previous + 1) target)))
    centers;
  let island_radius row =
    let t = (progress row -. 0.20) /. 0.49 in
    if island && t > 0. && t < 1. then
      let radius = min (float_of_int cols *. 0.14) (float_of_int rows *. 0.11) in
      max 1 (int_of_float (Float.round (sin (Float.pi *. t) *. radius)))
    else -1
  in
  let at frac = int_of_float (Float.round (frac *. float_of_int (rows - 1))) in
  let branch_width = max 2 (cols / 12) in
  let river_radius = if valley then cols / 26 else max 1 (cols / 16) in
  let north_bridge, south_bridge = (at 0.30, at 0.68) in
  let valley_radius row =
    let t = progress row in
    let near_bridge = abs (row - north_bridge) <= 1 || abs (row - south_bridge) <= 1 in
    if (not near_bridge) && (t < 0.16 || t > 0.86 || Float.abs (meander 1.44 (t *. 3.)) > 0.45) then
      river_radius + 1
    else river_radius
  in
  let west_row, east_row = (at 0.36, at 0.57) in
  let span kind row radius =
    { left = centers.(row) - radius; right = centers.(row) + radius; row; kind }
  in
  let crossings =
    if island then
      let west = centers.(west_row) - island_radius west_row - 1 in
      let east = centers.(east_row) + island_radius east_row + 1 in
      [
        { left = west - branch_width + 1; right = west; row = west_row; kind = Shallow };
        { left = east; right = east + branch_width - 1; row = east_row; kind = Shallow };
      ]
    else if valley then
      let shallows row =
        let crossing = span Shallow row (valley_radius row) in
        { crossing with right = max (centers.(row) + 1) crossing.right }
      in
      [
        span Bridged north_bridge river_radius;
        shallows ((rows / 2) - 1);
        shallows (rows / 2);
        span Bridged south_bridge river_radius;
      ]
    else [ span Shallow (at 0.48) river_radius ]
  in
  let channel ~col ~row =
    let center = centers.(row) in
    let distance = abs (col - center) in
    let crossing =
      List.find_map
        (fun { left; right; row = y; kind } ->
          if y <> row then None
          else if col >= left && col <= right then Some (Crossing kind)
          else if kind = Bridged then
            if col = left - 1 || col = right + 1 then Some (Approach across)
            else if col = left - 2 then Some (Approach [ {js|e|js} ])
            else if col = right + 2 then Some (Approach [ {js|w|js} ])
            else None
          else if col = left - 1 || col = right + 1 then Some Bank
          else None)
        crossings
    in
    let inner = island_radius row in
    let outer =
      if island then
        if inner >= 0 then inner + branch_width
        else
          let at_tip =
            (row > 0 && island_radius (row - 1) >= 0)
            || (row < rows - 1 && island_radius (row + 1) >= 0)
          in
          max (if at_tip then 4 else 2) (cols / 9)
      else if valley then valley_radius row
      else river_radius
    in
    match crossing with
    | Some kind -> kind
    | None -> if distance <= inner then Island else if distance <= outer then Water else Land
  in
  let lowland row = (not island) || progress row >= 0.17 in
  let marshes = floodplain ~channel ~fields ~tiles ~editable ~lowland in
  let tile ~col ~row =
    let height, wetness = fields ~col ~row in
    match channel ~col ~row with
    | Water -> water
    | Crossing kind -> crossing_tile kind
    | Approach _ -> road
    | Bank -> plains
    | Island -> if height > 0.24 then hill else if wetness > -0.30 then forest else plains
    | Land ->
        if island && progress row < 0.09 then mountain
        else if island && progress row < 0.17 then upland (height +. 0.23) wetness
        else if Hashtbl.mem marshes (col, row) then marsh
        else if valley then
          let edge = float_of_int (abs (col - centers.(row))) /. (float_of_int cols /. 2.) in
          let slope = max 0. (edge -. 0.25) in
          if (height *. 0.55) +. (slope *. 0.36) > 0.24 then hill
          else if wetness +. (slope *. 0.55) > 0.15 then forest
          else plains
        else upland height wetness
  in
  let connections ~col ~row =
    match channel ~col ~row with
    | Crossing _ -> across
    | Approach directions -> directions
    | Water | Bank | Island | Land -> []
  in
  { tile; connections; validate = validate_river ~rows ~channel ~crossings }

let create profile ~seed ~cols ~rows ~tiles ~editable =
  let fields = Draft_noise.fields ~seed ~cols ~rows in
  match profile with
  | River | River_valley | Lantern_crossing ->
      river ~seed ~cols ~rows ~profile ~fields ~tiles ~editable
  | Natural ->
      let tile ~col ~row =
        let height, wetness = fields ~col ~row in
        if height < -0.30 then water
        else if height < -0.20 && wetness > 0. then marsh
        else upland height wetness
      in
      { tile; connections = (fun ~col:_ ~row:_ -> []); validate = (fun _ -> ()) }
  | Lantern_wetlands ->
      let tile ~col ~row =
        let height, wetness = fields ~col ~row in
        let south = float_of_int row /. float_of_int (max 1 (rows - 1)) in
        let ridge = max 0. (0.36 -. south) *. 1.55 in
        let wetland = wetness +. ((south -. 0.52) *. 1.8) -. (0.5 *. height) in
        if wetland > 0.56 && height < -0.40 then water
        else if wetland > 0.26 && height < 0.23 then marsh
        else upland (height +. ridge) (wetness +. 0.08)
      in
      { tile; connections = (fun ~col:_ ~row:_ -> []); validate = (fun _ -> ()) }
