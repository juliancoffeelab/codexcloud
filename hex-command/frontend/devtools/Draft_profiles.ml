type profile = Natural | River | Lantern_crossing | Lantern_wetlands

let parse = function
  | "natural" -> Natural
  | "river" -> River
  | "lantern-crossing" -> Lantern_crossing
  | "lantern-wetlands" -> Lantern_wetlands
  | _ ->
      invalid_arg
        {js|Unknown terrain profile. Choose natural, river, lantern-crossing or lantern-wetlands.|js}

let plains = ({js|plains|js}, {js|green-plain|js})
let forest = ({js|forest|js}, {js|green-forest|js})
let hill = ({js|hill|js}, {js|green-plain|js})
let mountain = ({js|mountain|js}, {js|mountain-rock|js})
let marsh = ({js|marsh|js}, {js|marsh|js})
let water = ({js|water|js}, {js|dark-water|js})
let ford = ({js|ford|js}, {js|shallow-water|js})

let upland height wetness =
  if height > 0.48 then mountain
  else if height > 0.24 then hill
  else if wetness > 0.08 then forest
  else plains

type channel = Water | Ford | Bank | Island | Land

type plan = {
  tile : col:int -> row:int -> string * string;
  validate : (int * int, string) Hashtbl.t -> unit;
}

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

let validate_river ~rows ~channel ~crossings tiles =
  let waterway = Hashtbl.create 64 in
  let is_channel col row =
    match channel ~col ~row with Water | Ford -> true | Bank | Island | Land -> false
  in
  Hashtbl.iter
    (fun (col, row) terrain ->
      if is_channel col row && List.mem terrain [ {js|water|js}; {js|ford|js}; {js|bridge|js} ] then
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
    | Some terrain -> List.mem terrain [ fst plains; fst forest; fst hill; fst marsh; {js|road|js} ]
    | None -> false
  in
  List.iter
    (fun (left, right, row) ->
      let blocked () =
        invalid_arg
          {js|The template blocks a planned ford or its banks. Use clear terrain around the river crossings.|js}
      in
      for col = left to right do
        if Hashtbl.find_opt tiles (col, row) <> Some (fst ford) then blocked ()
      done;
      if not (clear_bank (left - 1, row) && clear_bank (right + 1, row)) then blocked ())
    crossings

let river ~seed ~cols ~rows ~island ~fields =
  let min_cols, min_rows = if island then (13, 9) else (7, 5) in
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
         else max (centers.(row - 1) - 1) (min (centers.(row - 1) + 1) target)))
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
  let river_radius = max 1 (cols / 16) in
  let west_row, east_row = (at 0.36, at 0.57) in
  let crossings =
    if island then
      let west = centers.(west_row) - island_radius west_row - 1 in
      let east = centers.(east_row) + island_radius east_row + 1 in
      [ (west - branch_width + 1, west, west_row); (east, east + branch_width - 1, east_row) ]
    else
      let row = at 0.48 in
      [ (centers.(row) - river_radius, centers.(row) + river_radius, row) ]
  in
  let channel ~col ~row =
    let center = centers.(row) in
    let distance = abs (col - center) in
    let is_ford =
      List.exists (fun (left, right, y) -> y = row && col >= left && col <= right) crossings
    in
    let at_bank =
      List.exists (fun (left, right, y) -> y = row && (col = left - 1 || col = right + 1)) crossings
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
      else river_radius
    in
    if is_ford then Ford
    else if at_bank then Bank
    else if distance <= inner then Island
    else if distance <= outer then Water
    else Land
  in
  let tile ~col ~row =
    let height, wetness = fields ~col ~row in
    match channel ~col ~row with
    | Water -> water
    | Ford -> ford
    | Bank -> plains
    | Island -> if height > 0.24 then hill else if wetness > -0.30 then forest else plains
    | Land ->
        if island && progress row < 0.09 then mountain
        else if island && progress row < 0.17 then upland (height +. 0.23) wetness
        else if wetness > 0.25 && height < -0.22 then marsh
        else upland height wetness
  in
  { tile; validate = validate_river ~rows ~channel ~crossings }

let create profile ~seed ~cols ~rows =
  let fields = Draft_noise.fields ~seed ~cols ~rows in
  match profile with
  | River | Lantern_crossing -> river ~seed ~cols ~rows ~island:(profile = Lantern_crossing) ~fields
  | Natural ->
      let tile ~col ~row =
        let height, wetness = fields ~col ~row in
        if height < -0.30 then water
        else if height < -0.20 && wetness > 0. then marsh
        else upland height wetness
      in
      { tile; validate = (fun _ -> ()) }
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
      { tile; validate = (fun _ -> ()) }
