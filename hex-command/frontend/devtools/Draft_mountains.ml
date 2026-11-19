let relief ~variation ~spacing ~along ~across =
  let distance = abs_float across in
  let salt = if across < 0. then 31. else 47. in
  let sample scale lean phase =
    variation
      (((along -. (lean *. distance)) /. scale) +. salt +. phase)
      ((distance /. (3. *. scale)) -. salt -. phase)
  in
  (* Stretch the coarse ridges down the slope. The smaller ridges run at
     different angles and split the shoulders into spurs and gullies. *)
  let spine = sample spacing 0.72 0. in
  let branch = sample (0.5 *. spacing) (-0.38) 11. in
  let gully = sample (0.25 *. spacing) 0.95 23. in
  spacing
  *. ((0.38 *. spine)
     +. (0.20 *. spine *. abs_float spine)
     +. ((0.17 +. (0.08 *. max 0. spine)) *. branch)
     +. (0.08 *. gully))

let ridge_flanks t ~offset ~width =
  let open Draft_layout in
  let spacing = max 8. (min 18. (float_of_int (min t.cols t.rows) /. 3.)) in
  iter t (fun ((_, row) as point) ->
      match Draft_boundary.surface t point with
      | Some ("mountain" | "hill" | "plains" | "forest") ->
          let core = width row in
          let delta = offset point in
          if abs delta >= core then
            let stagger = 0.5 *. float_of_int (row mod 2 * forward t) in
            let across = float_of_int delta +. stagger in
            let along = 0.866 *. float_of_int row in
            let edge = float_of_int core +. relief ~variation:t.variation ~spacing ~along ~across in
            let slope = float_of_int (abs delta) -. edge in
            let height, moisture = noise t point in
            let material =
              if slope <= 0. then mountain
              else if slope <= 2. || height > 0.30 then hill
              else if moisture > 0.06 then forest
              else plains
            in
            paint t point material
      | _ -> ())
