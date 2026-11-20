open Draft_layout

let surface t point =
  match Hashtbl.find_opt t.cells point with
  | Some { paint = Some { material = kind, _; geometry = Surface; landmark = None }; _ } ->
      Some kind
  | _ -> None

let can_shift classify point =
  let ring = List.filter_map (fun (_, neighbor) -> classify neighbor) (adjacent point) in
  let rec transitions first previous = function
    | [] -> if previous = first then 0 else 1
    | next :: rest -> (if previous = next then 0 else 1) + transitions first next rest
  in
  (* Each side must form one connected arc in the hex ring. Moving the
     boundary here then preserves the components on both sides. *)
  match ring with
  | [ first; b; c; d; e; f ] -> transitions first first [ b; c; d; e; f ] = 2
  | _ -> false

let refine t ~classify ~replacement =
  let passes = [ (9., 3, 0.30); (4., 2, 0.41); (2., 1, 0.53) ] in
  let apply (scale, rounds, threshold) =
    let bias (col, row) =
      let x = float_of_int (local_col t col) +. (0.5 *. float_of_int (row mod 2)) in
      let y = 0.866 *. float_of_int row in
      let sample size offset = t.variation ((x /. size) +. offset) ((y /. size) -. offset) in
      (0.72 *. sample scale (scale +. 11.)) +. (0.28 *. sample (scale *. 2.) 23.)
    in
    for _round = 1 to rounds do
      let candidates = ref [] in
      iter t (fun point ->
          match classify point with
          | Some side ->
              let score = bias point *. if side then -1. else 1. in
              if score > threshold && can_shift classify point then
                Option.iter
                  (fun material -> candidates := (score, point, material) :: !candidates)
                  (replacement point (not side))
          | None -> ());
      List.sort (fun (a, _, _) (b, _, _) -> Float.compare b a) !candidates
      |> List.iter (fun (_, point, material) ->
          if can_shift classify point then paint t point material)
    done
  in
  List.iter apply passes
