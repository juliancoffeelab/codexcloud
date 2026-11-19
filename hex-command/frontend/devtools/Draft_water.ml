open Draft_layout

type crossing = { row : int; left : int; right : int; material : material }

let relative t col row = (local_col t col, row)
let band t ~row left right = List.init (right - left + 1) (fun i -> relative t (left + i) row)

let surface_path t feature ~allowed a b =
  path t feature ~allowed:(fun p -> edited_land t p && allowed p) a b

let use_crossing t number crossing =
  let { row; left; right; material } = crossing in
  let cells = band t ~row left right in
  let start = relative t (left - 1) row and finish = relative t (right + 1) row in
  if not (edited_land t start && edited_land t finish) then feature_error "crossing banks";
  List.iter (fun point -> require t "river crossing" point material ~geometry:across) cells;
  mark t ("crossing-" ^ string_of_int number) (relative t ((left + right) / 2) row);
  start :: (cells @ [ finish ])

let contiguous_water t ~feature ~starts ~required ~candidate =
  validate_waterway t feature ~starts ~destination:(fun reached ->
      let valid = ref (required reached) in
      iter t (fun point ->
          if candidate point && wet t point && not (Hashtbl.mem reached point) then valid := false);
      !valid)

let soften_shores t =
  let shores = ref [] in
  iter t (fun point ->
      if
        terrain t point = Some (fst water)
        && List.exists
             (fun (_, neighbor) -> walkable t neighbor && not (wet t neighbor))
             (adjacent point)
      then shores := point :: !shores);
  List.iter (fun point -> paint t point shallows) !shores

let spread_marsh t ~depth ~dense =
  let visited = Hashtbl.create 64 in
  let frontier = Queue.create () in
  iter t (fun point ->
      if wet t point then (
        Hashtbl.add visited point ();
        Queue.add (point, 0) frontier));
  while not (Queue.is_empty frontier) do
    let point, step = Queue.take frontier in
    if step < depth then
      List.iter
        (fun (_, next) ->
          let lowland =
            match terrain t next with Some ("plains" | "forest" | "hill") -> true | _ -> false
          in
          if lowland && editable t next && not (Hashtbl.mem visited next) then (
            Hashtbl.add visited next ();
            let height, moisture = noise t next in
            let cutoff = if dense then 0.24 -. (0.10 *. float_of_int step) else -0.18 in
            if height < cutoff && moisture > if dense then -0.34 else 0.02 then (
              paint t next marsh;
              Queue.add (next, step + 1) frontier)))
        (adjacent point)
  done

let river t ~island ~flood ~near_edge ~additional ~defense =
  let broad = max 2 (t.cols / if flood then 9 else 13) in
  let centers = Array.make t.rows (t.cols / 2) in
  Array.iteri
    (fun row _ ->
      let progress = yfrac t (0, row) in
      let target = index t.cols (0.50 +. (0.12 *. t.variation 0.78 (1.95 *. progress))) in
      centers.(row) <-
        (if row = 0 then target
         else
           let previous = centers.(row - 1) in
           max (previous - 1) (min (previous + 1) target)))
    centers;
  let island_size = max 3 (min (t.cols / 6) (t.rows / 4)) in
  let first = row t 0.20 and last = row t 0.81 in
  let island_radii = Array.make t.rows (-1) in
  if island then (
    for y = first to last do
      let phase = float_of_int (y - first) /. float_of_int (last - first) in
      let desired =
        max 1 (int_of_float (Float.round (float_of_int island_size *. sin (Float.pi *. phase))))
      in
      let limit = if y = first then 1 else island_radii.(y - 1) + 1 in
      island_radii.(y) <- min desired limit
    done;
    for y = last - 1 downto first do
      island_radii.(y) <- min island_radii.(y) (island_radii.(y + 1) + 1)
    done);
  let island_radius row = island_radii.(row) in
  let branch = max 3 (t.cols / 18) in
  let water_band row =
    let inner = island_radius row in
    if inner >= 0 then inner + branch else broad + if island then 1 else 0
  in
  let channel (col, row) =
    let delta = abs (local_col t col - centers.(row)) in
    delta <= water_band row && delta > island_radius row
  in
  iter t (fun point ->
      let col, row = point in
      let delta = abs (local_col t col - centers.(row)) in
      let material =
        if channel point then water
        else if delta <= island_radius row || delta <= water_band row + 2 then
          let _, moisture = noise t point in
          if moisture > 0.18 then forest else plains
        else open_terrain t point
      in
      paint t point material);
  let main_row =
    let offset = vary t 8. in
    if near_edge then
      row t (if offset < 0. then 0.18 -. (0.06 *. offset) else 0.82 -. (0.06 *. offset))
    else row t (0.50 +. (0.13 *. offset))
  in
  let crossing row material side =
    let center = centers.(row) and inner = island_radius row and outer = water_band row in
    let left, right =
      match side with
      | `West -> (center - outer, center - inner - 1)
      | `East -> (center + inner + 1, center + outer)
      | `Both -> (center - outer, center + outer)
    in
    { row; left; right; material }
  in
  let crossings =
    if island then
      let west = row t (0.35 +. (0.05 *. vary t 5.))
      and east = row t (0.61 +. (0.05 *. vary t 7.)) in
      let main = [ crossing west bridge `West; crossing east bridge `East ] in
      if additional then main @ [ crossing (row t (0.67 +. (0.03 *. vary t 3.))) bridge `West ]
      else main
    else if additional then
      [
        crossing (row t (0.32 +. (0.05 *. vary t 5.))) bridge `Both;
        crossing (row t (0.66 +. (0.05 *. vary t 7.))) ford `Both;
      ]
    else if near_edge then [ crossing main_row bridge `Both ]
    else [ crossing main_row ford `Both; crossing (main_row + 1) ford `Both ]
  in
  let tributaries = Hashtbl.create 32 in
  if flood then (
    let on_crossing_rows (_, row) = List.exists (fun c -> abs (c.row - row) <= 1) crossings in
    let tributary salt from_x from_y end_y =
      let tail = row t end_y in
      let head = jitter t salt ~x:from_x ~y:from_y ~dx:0.04 ~dy:0.04 in
      let tail = relative t centers.(tail) tail in
      let head =
        nearest t "floodplain tributary" head ~within:2 ~allowed:(fun p ->
            editable t p && not (on_crossing_rows p))
      in
      let tail =
        nearest t "floodplain tributary" tail ~within:3 ~allowed:(fun p ->
            wet t p && editable t p && not (on_crossing_rows p))
      in
      let allowed p = editable t p && not (on_crossing_rows p) in
      let course = path t "floodplain tributary" ~allowed head tail in
      List.iter
        (fun point ->
          Hashtbl.replace tributaries point ();
          paint t point water)
        course
    in
    if main_row >= row t 0.35 then tributary 17. 0.22 0.10 0.17;
    if main_row <= row t 0.65 then tributary 19. 0.79 0.88 0.81;
    if near_edge && main_row < row t 0.35 then tributary 20. 0.23 0.77 0.68;
    if near_edge && main_row > row t 0.65 then tributary 21. 0.79 0.18 0.29);
  soften_shores t;
  let crossing_paths = List.mapi (fun i c -> use_crossing t (i + 1) c) crossings in
  let crossing_cells = Hashtbl.create 16 in
  List.iter
    (fun ({ row; left; right; _ } : crossing) ->
      List.iter (fun p -> Hashtbl.replace crossing_cells p ()) (band t ~row left right))
    crossings;
  let north = List.filter (fun p -> channel p && wet t p) (band t ~row:0 0 (t.cols - 1)) in
  let head = match north with point :: _ -> point | [] -> feature_error "river headwater" in
  contiguous_water t ~feature:"continuous river channel" ~starts:[ head ]
    ~required:(fun reached ->
      List.exists (Hashtbl.mem reached) (band t ~row:(t.rows - 1) 0 (t.cols - 1)))
    ~candidate:(fun p -> channel p || Hashtbl.mem tributaries p);
  let first = List.hd crossing_paths in
  let start_bank = List.hd first in
  let finish_path = if island then List.nth crossing_paths 1 else first in
  let finish_bank = List.hd (List.rev finish_path) in
  let bank side point =
    editable t point
    && (not (wet t point))
    &&
    let col, row = point in
    side * (local_col t col - centers.(row)) > island_radius row
  in
  let left_path = path t "entry approach" ~allowed:(bank (-1)) in
  let right_path = path t "exit approach" ~allowed:(bank 1) in
  let from_start = connected [ start_bank ] ~allowed:(bank (-1)) in
  let from_finish = connected [ finish_bank ] ~allowed:(bank 1) in
  let approach_row = yfrac t start_bank in
  let start =
    nearest t "entry site"
      (jitter t 27. ~x:0.12 ~y:approach_row ~dx:0.04 ~dy:0.09)
      ~within:4 ~allowed:(Hashtbl.mem from_start)
  in
  let finish_y = if island then yfrac t finish_bank else approach_row in
  let exit_anchor =
    if near_edge then at t 1. (max 0.04 (min 0.96 (finish_y +. (0.12 *. vary t 14.))))
    else jitter t 29. ~x:0.88 ~y:finish_y ~dx:0.04 ~dy:0.10
  in
  let finish =
    nearest t "exit site" exit_anchor ~within:4 ~allowed:(fun p ->
        Hashtbl.mem from_finish p && ((not near_edge) || xfrac t p = 1.))
  in
  let enter = left_path start start_bank and leave = right_path finish_bank finish in
  let paths, city =
    if island then (
      let center = relative t centers.(row t 0.50) (row t 0.50) in
      let inside (col, row) = abs (local_col t col - centers.(row)) <= island_radius row in
      let streets =
        List.map2
          (fun crossing route ->
            let east = crossing.left > centers.(crossing.row) in
            if east then (forward t, List.hd route) else (-forward t, List.hd (List.rev route)))
          crossings crossing_paths
      in
      let outside radius p = edited_land t p && inside p && distance center p > radius in
      let entrance radius side = List.nth (gate_approach center radius side) 1 in
      let room_for radius =
        List.for_all
          (fun (side, from) ->
            let reached = connected [ entrance radius side ] ~allowed:(outside radius) in
            Hashtbl.mem reached from)
          streets
      in
      let radius = if island_size >= 5 && room_for 2 then 2 else 1 in
      region t center radius (fun p -> if not (channel p) then paint t p plains);
      let city_route = path t "island street" ~allowed:(outside radius) in
      let west_gate = gate_approach center radius (-forward t)
      and east_gate = gate_approach center radius (forward t) in
      let links = List.map (fun (side, from) -> city_route from (entrance radius side)) streets in
      let west_detour =
        List.filter_map
          (fun crossing ->
            if crossing.left < centers.(crossing.row) && crossing.row <> (List.hd crossings).row
            then Some (left_path start (relative t (crossing.left - 1) crossing.row))
            else None)
          crossings
      in
      (west_gate :: east_gate :: (links @ west_detour), Some (center, radius)))
    else if additional then
      let alternate = List.nth crossing_paths 1 in
      ( [ left_path start (List.hd alternate); right_path (List.hd (List.rev alternate)) finish ],
        None )
    else ([], None)
  in
  let defense_site =
    if defense then
      let anchor = jitter t 31. ~x:0.73 ~y:0.50 ~dx:0.04 ~dy:0.06 in
      let post =
        nearest t "river guard post" anchor ~within:4 ~allowed:(fun p ->
            Hashtbl.mem from_finish p && p <> finish && p <> finish_bank)
      in
      let defense_road = right_path finish_bank post in
      Some (post, defense_road)
    else None
  in
  let paths = Option.fold ~none:paths ~some:(fun (_, course) -> course :: paths) defense_site in
  roads t ((enter :: leave :: crossing_paths) @ paths);
  Option.iter
    (fun (post, _) ->
      paint t post fort;
      mark t "guard-post" post;
      validate_route t "guard post approach" finish post)
    defense_site;
  mark t "entry" start;
  mark t (if near_edge then "exit" else "objective") finish;
  Option.iter (fun (center, radius) -> castle t ~radius center) city;
  spread_marsh t ~depth:(if flood then 5 else 2) ~dense:flood;
  validate_route t "river crossing route" start finish;
  validate_cut t "river crossings" [ start ] finish ~blocked:(Hashtbl.mem crossing_cells)

let coast t ~siege ~more_islands =
  let shore row =
    let y = yfrac t (0, row) in
    let coast = 0.35 +. (0.10 *. t.variation 1.4 (y *. 2.7)) in
    let inlet = max 0. (1. -. (abs_float (y -. 0.44) /. 0.24)) in
    index t.cols (coast -. (0.06 *. inlet))
  in
  let body = max 2 (min t.cols t.rows / 10) in
  let islands =
    [
      (jitter t 4. ~x:0.64 ~y:0.21 ~dx:0.04 ~dy:0.05, body);
      (jitter t 3. ~x:0.80 ~y:0.50 ~dx:0.045 ~dy:0.04, body + 1);
      (jitter t 6. ~x:0.61 ~y:0.79 ~dx:0.04 ~dy:0.04, body);
    ]
    @
    if more_islands then [ (jitter t 13. ~x:0.92 ~y:0.80 ~dx:0.02 ~dy:0.05, max 1 (body - 1)) ]
    else []
  in
  let mainland (col, row) = row >= 0 && row < t.rows && local_col t col <= shore row in
  let offshore point =
    (not (mainland point)) && not (List.exists (fun (_, p) -> mainland p) (adjacent point))
  in
  let island point center radius =
    let d = distance center point in
    let height, _ = noise t point in
    offshore point && (d < radius || (d = radius && height > -0.26))
  in
  iter t (fun point ->
      let col, row = point in
      let height, moisture = noise t point in
      let material =
        if mainland point then
          if shore row - local_col t col <= 1 then ("plains", "autumn-forest")
          else if height > 0.28 then hill
          else if moisture > -0.15 then forest
          else plains
        else if List.exists (fun (center, radius) -> island point center radius) islands then
          if height > 0.22 then hill else if moisture > 0.20 then forest else plains
        else water
      in
      paint t point material);
  let harbors = if siege then [ 0.48 ] else [ 0.32; 0.70 ] in
  let ports =
    List.mapi
      (fun i y ->
        let anchor = row t (y +. (0.05 *. vary t (7. +. float_of_int i))) in
        let options =
          List.init 7 (fun n -> if n mod 2 = 0 then anchor + (n / 2) else anchor - ((n + 1) / 2))
        in
        let usable row =
          let landing = relative t (shore row) row and ferry = relative t (shore row + 1) row in
          editable t landing && editable t ferry && wet t ferry
          && not (List.exists (fun (_, p) -> walkable t p && not (mainland p)) (adjacent ferry))
        in
        let row =
          match List.find_opt usable options with
          | Some row -> row
          | None -> feature_error "coastal harbor"
        in
        let x = shore row in
        let landing = relative t x row and ferry = relative t (x + 1) row in
        let outbound, inbound = if forward t = 1 then ("e", "w") else ("w", "e") in
        require t "harbor landing" landing ("landing", "green-plain") ~geometry:(Exits [ outbound ]);
        require t "moored ferry" ferry ("ferry", "dark-water") ~geometry:(Exits [ inbound ]);
        mark t ("harbor-" ^ string_of_int (i + 1)) landing;
        landing)
      harbors
  in
  soften_shores t;
  let coast_bound p = mainland p && edited_land t p in
  let entry =
    nearest t "coastal entry"
      (jitter t 23. ~x:0.08 ~y:0.48 ~dx:0.025 ~dy:0.13)
      ~within:4 ~allowed:coast_bound
  in
  let routes = List.map (surface_path t "port road" ~allowed:mainland entry) ports in
  roads t routes;
  mark t "entry" entry;
  List.iteri
    (fun index (center, radius) ->
      let objective =
        nearest t "offshore landmark" center ~within:1 ~allowed:(fun point ->
            edited_land t point && island point center radius)
      in
      if siege && index = 1 then castle t ~radius:(max 1 (radius - 1)) objective
      else (
        paint t objective (if index = 0 then fort else ruins);
        mark t ("offshore-" ^ string_of_int (index + 1)) objective);
      validate_cut t "offshore separation" [ entry ] objective ~blocked:(fun _ -> false))
    islands;
  let sea_edge = band t ~row:0 0 (t.cols - 1) |> List.filter (wet t) in
  let start = match List.rev sea_edge with head :: _ -> head | [] -> feature_error "open sea" in
  contiguous_water t ~feature:"open sea around islands" ~starts:[ start ]
    ~required:(fun reached -> Hashtbl.mem reached (at t 1. 1.))
    ~candidate:(fun point -> not (mainland point));
  List.iter (validate_route t "port approach" entry) ports
