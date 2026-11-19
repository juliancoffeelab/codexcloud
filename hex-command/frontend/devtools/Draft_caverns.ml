open Draft_layout

let create t ~frontier ~rift ~exploration ~siege =
  let at_col col row = (local_col t col, row) in
  let gate_row = row t (0.48 +. (0.18 *. vary t 10.)) in
  let ridge = Array.make t.rows (t.cols / 2) in
  Array.iteri
    (fun row _ ->
      let y = yfrac t (0, row) in
      let x = 0.51 +. (0.073 *. t.variation 0.79 (1.68 *. y)) in
      let target = index t.cols x in
      ridge.(row) <-
        (if row = 0 then target
         else
           let previous = ridge.(row - 1) in
           max (previous - 1) (min (previous + 1) target)))
    ridge;
  let width = if rift then max 1 (t.cols / 18) else 1 in
  let offset (col, row) = local_col t col - ridge.(row) in
  let west p = offset p < -width and east p = offset p > width in
  let rift_forks = if rift then [ (0.20, -1); (0.81, 1) ] else [] in
  let flooded p =
    let x = float_of_int (offset p) and y = yfrac t p in
    abs_float x <= float_of_int width
    || List.exists
         (fun (mouth, sign) ->
           let down = (y -. mouth) *. float_of_int sign in
           let reach = float_of_int (min t.cols t.rows) *. 0.11 in
           down >= 0. && down <= 0.13
           && x *. float_of_int sign >= 0.
           && abs_float x <= float_of_int width +. (reach *. ((down /. 0.13) ** 0.7)))
         rift_forks
  in
  iter t (fun point ->
      let terrain =
        if rift && flooded point then water
        else if frontier && east point then arid_terrain t point
        else cave_rock
      in
      paint t point terrain);
  let in_cave side p = editable t p && (not (wet t p)) && side p in
  let carve side feature a b =
    let course = path t feature ~allowed:(in_cave side) a b in
    List.iter (fun p -> paint t p cave_floor) course;
    course
  in
  let chamber ~side ?(radius_delta = 0) role salt x y =
    let anchor = jitter t salt ~x ~y ~dx:0.028 ~dy:0.068 in
    let center = nearest t role anchor ~within:3 ~allowed:(in_cave side) in
    let radius = max 1 (min 5 (min t.cols t.rows / 11) + radius_delta) in
    region t center (radius + 1) (fun p ->
        let d = distance center p and height, moisture = noise t p in
        if in_cave side p && (d <= radius || height +. (0.8 *. moisture) > 0.21) then
          paint t p cave_floor);
    center
  in
  let left = at_col (ridge.(gate_row) - width - 1) gate_row
  and right = at_col (ridge.(gate_row) + width + 1) gate_row in
  let entry = chamber ~side:west ~radius_delta:(-1) "cave entry" 4. 0.08 0.54 in
  let hub = chamber ~side:west ~radius_delta:1 "cave hub" 7. 0.26 0.52 in
  let key_site = chamber ~side:west "side chamber" 14. 0.21 0.19 in
  ignore (carve west "entry tunnel" entry hub);
  ignore (carve west "gate approach" hub left);
  ignore (carve west "key chamber tunnel" hub key_site);
  mark t "entry" entry;
  mark t "hub" hub;
  mark t "key-site" key_site;
  if exploration then (
    let lower = chamber ~side:west "lower gallery" 13. 0.23 0.83 in
    let upper = chamber ~side:west ~radius_delta:(-1) "upper gallery" 15. 0.38 0.30 in
    ignore (carve west "lower gallery tunnel" hub lower);
    ignore (carve west "upper gallery tunnel" key_site upper);
    ignore (carve west "gallery loop" upper left);
    mark t "lower-gallery" lower;
    mark t "upper-gallery" upper);
  let gate_cells =
    List.init ((2 * width) + 1) (fun i -> at_col (ridge.(gate_row) - width + i) gate_row)
  in
  List.iter
    (fun point ->
      if rift then require t "rift bridge" point bridge ~geometry:across
      else require t "cave gate" point cave_floor)
    gate_cells;
  let gate = at_col ridge.(gate_row) gate_row in
  mark t "gate" gate;
  let finish =
    if frontier then (
      require t "cave mouth" right plains;
      mark t "cave-mouth" right;
      let mouth_apron p = editable t p && east p in
      region t right 2 (fun p -> if mouth_apron p then paint t p ("plains", "autumn-forest"));
      let site role x y =
        nearest t role
          (jitter t (9. +. (10. *. x)) ~x ~y ~dx:0.025 ~dy:0.05)
          ~within:3
          ~allowed:(fun p -> east p && edited_land t p)
      in
      let station = site "frontier waystation" 0.69 0.72 in
      let ruin = site "frontier ruins" 0.79 0.24 in
      let exit =
        nearest t "frontier exit" (at t 1. 0.52) ~within:4 ~allowed:(fun p ->
            xfrac t p = 1. && edited_land t p)
      in
      let trail = path t "frontier trail" ~allowed:(fun p -> editable t p && east p) in
      let main = [ trail right station; trail station exit ] in
      let routes = if exploration then trail right ruin :: trail ruin exit :: main else main in
      roads t routes ~floor:"autumn-forest";
      mark t "waystation" station;
      if exploration then (
        paint t ruin ("ruins", "autumn-forest");
        mark t "frontier-ruins" ruin);
      mark t "exit" exit;
      exit)
    else
      let heart =
        chamber ~side:east ~radius_delta:(if siege then 2 else 1) "deep chamber" 8. 0.75 0.49
      in
      let goal = chamber ~side:east "inner sanctum" 12. 0.92 0.59 in
      let lower = chamber ~side:east "deep gallery" 17. 0.75 0.83 in
      ignore (carve east "deep gate approach" right heart);
      ignore (carve east "sanctum tunnel" heart goal);
      ignore (carve east "deep gallery tunnel" heart lower);
      mark t "deep-gallery" lower;
      if exploration then (
        let upper = chamber ~side:east "deep refuge" 20. 0.77 0.19 in
        ignore (carve east "refuge tunnel" heart upper);
        mark t "refuge" upper);
      if siege then
        let radius = min 3 (max 2 (min t.cols t.rows / 13)) in
        castle t ~radius heart
      else mark t "deep-hub" heart;
      mark t "objective" goal;
      goal
  in
  let classify p =
    match Draft_boundary.surface t p with
    | Some "cave-wall" -> Some false
    | Some "cave-passage" -> Some true
    | _ -> None
  in
  Draft_boundary.refine t ~classify ~replacement:(fun p open_cave ->
      if open_cave && abs (offset p) > width + 1 then Some cave_floor else None);
  if rift then Draft_water.refine_shores t ~bank:(( = ) (fst cave_rock)) ~shelf:cave_rock;
  cave_openings t ~allowed:(walkable t);
  let reachable = connected [ entry ] ~allowed:(walkable t) in
  iter t (fun p ->
      if editable t p && terrain t p = Some (fst cave_floor) && not (Hashtbl.mem reachable p) then
        feature_error "connected cave chambers");
  validate_route t "route through the cave gate" entry finish;
  validate_route t "route to the key chamber" entry key_site;
  validate_cut t "cave gate" [ entry ] finish ~blocked:(( = ) gate);
  validate_cut t "key location before the gate" [ finish ] key_site ~blocked:(( = ) gate);
  if rift then
    let north = at_col ridge.(0) 0 and last = t.rows - 1 in
    let south = at_col ridge.(last) last in
    Draft_water.contiguous_water t ~feature:"continuous cavern rift" ~starts:[ north ]
      ~required:(fun reached -> Hashtbl.mem reached south)
      ~candidate:(fun p -> editable t p || flooded p)
