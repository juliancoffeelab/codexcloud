open Draft_layout

let woodland t ~siege ~exploration =
  let center = jitter t 7. ~x:0.63 ~y:0.49 ~dx:0.077 ~dy:0.087 in
  let radius = if siege then min 5 (max 2 (min t.cols t.rows / 8)) else 2 in
  let north = jitter t 5. ~x:0.32 ~y:0.18 ~dx:0.065 ~dy:0.06 in
  let south = jitter t 11. ~x:0.33 ~y:0.81 ~dx:0.065 ~dy:0.06 in
  iter t (fun point ->
      let height, moisture = noise t point in
      let clearing =
        distance center point <= radius + 2
        || distance north point <= 2
        || distance south point <= 2
      in
      let terrain =
        if clearing || moisture < -0.43 then if height > 0.30 then hill else plains
        else if height > 0.41 && moisture < -0.05 then hill
        else forest
      in
      paint t point terrain);
  let west_gate = gate_approach center radius (-forward t)
  and east_gate = gate_approach center radius (forward t) in
  let outer = List.nth west_gate 1 and inner = List.nth east_gate 1 in
  let entry = land_site t "woodland entry" (jitter t 4. ~x:0.04 ~y:0.48 ~dx:0.03 ~dy:0.10) 3 in
  let north = land_site t "north clearing" north 2
  and south = land_site t "south clearing" south 2 in
  let east =
    land_site t "woodland destination" (jitter t 12. ~x:0.91 ~y:0.51 ~dx:0.035 ~dy:0.15) 2
  in
  let approach feature start finish =
    let allowed p = edited_land t p && (p = finish || p = start || distance p center > radius) in
    path t feature ~allowed start finish
  in
  let western =
    if siege || exploration then
      [
        approach "northern woodland approach" entry north;
        approach "southern woodland approach" entry south;
        approach "northern fortress road" north outer;
        approach "southern fortress road" south outer;
      ]
    else
      [
        approach "woodland approach" entry outer;
        approach "north watch post" north outer;
        approach "south watch post" south outer;
      ]
  in
  let routes =
    west_gate :: east_gate :: approach "eastern woodland approach" inner east :: western
  in
  let setback site point = distance site point <= 2 && distance center point > radius in
  let outpost role site =
    let p =
      nearest t role site ~within:2 ~allowed:(fun p ->
          edited_land t p && setback site p && terrain t p <> Some (fst road))
    in
    settlement t role p;
    validate_route t role site p
  in
  if exploration then (
    let shrine = land_site t "woodland shrine" (jitter t 18. ~x:0.85 ~y:0.19 ~dx:0.04 ~dy:0.05) 2 in
    let secret = approach "shrine trail" east shrine in
    roads t (routes @ [ secret ]);
    paint t shrine ("monastery", "green-plain");
    mark t "shrine" shrine;
    validate_route t "shrine approach" east shrine)
  else roads t routes;
  outpost "north-outpost" north;
  outpost "south-outpost" south;
  castle t ~radius center;
  mark t "entry" entry;
  mark t "objective" east;
  validate_route t "woodland fortress approach" entry center;
  validate_route t "woodland route" center east

let ridge t ~near_edge ~alternate =
  let centers = Array.make t.rows (t.cols / 2) in
  Array.iteri
    (fun row _ ->
      let y = yfrac t (0, row) in
      let ridge = 0.47 +. (0.15 *. t.variation 0.43 (1.7 *. y)) +. (0.12 *. (y -. 0.5)) in
      let target = index t.cols ridge in
      centers.(row) <-
        (if row = 0 then target
         else
           let prev = centers.(row - 1) in
           max (prev - 1) (min (prev + 1) target)))
    centers;
  let offset p =
    let col, row = p in
    local_col t col - centers.(row)
  in
  let width row = max 2 (t.cols / 15) + if vary t (3.0 *. yfrac t (0, row)) > 0.18 then 1 else 0 in
  let narrows p = abs (offset p) <= width (snd p) in
  iter t (fun point ->
      let height, moisture = noise t point in
      let material =
        if narrows point then mountain
        else if abs (offset point) <= width (snd point) + 2 || height > 0.30 then hill
        else if moisture > 0.06 then forest
        else plains
      in
      paint t point material);
  let location = vary t 5.5 in
  let first =
    if near_edge then (if location < 0. then 0.24 else 0.76) +. (0.055 *. location)
    else if alternate then 0.29 +. (0.07 *. location)
    else 0.52 +. (0.16 *. location)
  in
  let passes = if alternate then [ first; 0.72 +. (0.08 *. vary t 14.) ] else [ first ] in
  let exit = if near_edge then 1. else 0.93 in
  let site role x y =
    nearest t role (at t x y) ~within:3 ~allowed:(fun p ->
        edited_land t p && (x <> 1. || xfrac t p = 1.))
  in
  let start = site "ridge entry" 0.07 (first -. (0.13 *. vary t 2.)) in
  let finish = site "ridge exit" exit (first +. (0.14 *. vary t 17.)) in
  let crossing_cells = Hashtbl.create 16 in
  let routes =
    List.mapi
      (fun i gate_position ->
        let row = row t gate_position in
        let middle = centers.(row) and wing = width row in
        let pass =
          List.init ((2 * wing) + 3) (fun i -> (local_col t (middle - wing - 1 + i), row))
        in
        let west = List.hd pass and east = List.hd (List.rev pass) in
        List.iter
          (fun p ->
            require t "mountain pass" p plains;
            if narrows p then Hashtbl.add crossing_cells p ())
          pass;
        mark t ("pass-" ^ string_of_int (i + 1)) (local_col t middle, row);
        let side sign p = editable t p && sign * offset p > width (snd p) in
        [
          path t "western pass approach" ~allowed:(side (-1)) start west;
          pass;
          path t "eastern pass approach" ~allowed:(side 1) east finish;
        ])
      passes
    |> List.concat
  in
  roads t routes;
  mark t "entry" start;
  mark t "exit" finish;
  validate_route t "route through mountain pass" start finish;
  validate_cut t "mountain pass" [ start ] finish ~blocked:(Hashtbl.mem crossing_cells)

let badlands t ~near_edge ~exploration =
  iter t (fun point -> paint t point (arid_terrain t point));
  let radius = max 1 (min t.rows t.cols / 17) in
  let make_oasis i (x, y) =
    let center = jitter t (float_of_int (i + 4)) ~x ~y ~dx:0.05 ~dy:0.10 in
    let grove_edge p = distance p center > radius in
    region t center (radius + 2) (fun p ->
        let height, moisture = noise t p and distance = distance p center in
        let material =
          if distance < radius || (distance = radius && height < 0.20) then shallows
          else if distance <= radius + 1 && moisture < 0.18 then marsh
          else if moisture > -0.12 then forest
          else plains
        in
        paint t p material);
    require t "oasis pool" center water;
    let west, row = center in
    let shore = (west - (forward t * (radius + 1)), row) in
    let station =
      nearest t "oasis bank" shore ~within:2 ~allowed:(fun p -> edited_land t p && grove_edge p)
    in
    (center, station)
  in
  let stages = List.mapi make_oasis [ (0.27, 0.29); (0.48, 0.70); (0.72, 0.39) ] in
  let remote = if exploration then Some (snd (make_oasis 10 (0.80, 0.80))) else None in
  let inner_exit = if near_edge then 1. else 0.94 in
  let entry = land_site t "badlands entry" (jitter t 13. ~x:0.04 ~y:0.54 ~dx:0.03 ~dy:0.16) 4 in
  let finish =
    nearest t "badlands exit" (jitter t 15. ~x:inner_exit ~y:0.53 ~dx:0. ~dy:0.18) ~within:5
      ~allowed:(fun p -> edited_land t p && ((not near_edge) || xfrac t p = 1.))
  in
  let layout_road start finish =
    path t "oasis trail" ~allowed:(fun p -> editable t p && not (wet t p)) start finish
  in
  let rec itinerary = function
    | a :: (b :: _ as next) -> layout_road a b :: itinerary next
    | _ -> []
  in
  let camp_sites = List.map snd stages in
  let main = itinerary ((entry :: camp_sites) @ [ finish ]) in
  let extra =
    match remote with
    | None -> ([], [])
    | Some remote ->
        let middle = List.nth camp_sites 1 in
        ([ layout_road middle remote; layout_road remote finish ], [ remote ])
  in
  let detour, extra_sites = extra in
  roads t (main @ detour) ~floor:"autumn-forest";
  List.iteri
    (fun i p ->
      mark t ("oasis-" ^ string_of_int (i + 1)) p;
      validate_route t "oasis trail" entry p)
    (camp_sites @ extra_sites);
  mark t "entry" entry;
  mark t "exit" finish;
  validate_route t "route through the badlands" entry finish
