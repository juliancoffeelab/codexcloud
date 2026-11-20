(* The campaign planner works on the supplied footprint. Protected tiles take
   part in connectivity checks but never receive generated paint or metadata. *)
type point = int * int
type facing = East | West
type material = string * string
type geometry = Surface | Exits of string list | Walls of string list
type paint = { material : material; geometry : geometry; landmark : string option }
type cell = { original : string; mutable paint : paint option }

type t = {
  cols : int;
  rows : int;
  facing : facing;
  cells : (point, cell) Hashtbl.t;
  ids : (string, unit) Hashtbl.t;
  fields : col:int -> row:int -> float * float;
  variation : float -> float -> float;
}

let create ~seed ~cols ~rows ~facing ~tiles ~editable ~ids =
  let cells = Hashtbl.create (Hashtbl.length tiles) in
  Hashtbl.iter
    (fun point original ->
      let paint =
        if editable point then
          Some { material = (original, ""); geometry = Surface; landmark = None }
        else None
      in
      Hashtbl.add cells point { original; paint })
    tiles;
  {
    cols;
    rows;
    facing;
    cells;
    ids;
    fields = Draft_noise.fields ~seed ~cols ~rows;
    variation = Draft_noise.simplex (Int32.logxor seed 0x35fa94c1l);
  }

let plains = ("plains", "green-plain")
let forest = ("forest", "green-forest")
let hill = ("hill", "green-plain")
let mountain = ("mountain", "mountain-rock")
let marsh = ("marsh", "marsh")
let water = ("water", "dark-water")
let shallows = ("shallow-water", "shallow-water")
let road = ("road", "green-plain")
let fort = ("fort", "green-plain")
let ruins = ("ruins", "green-plain")
let cave_floor = ("cave-passage", "cave-stone")
let cave_rock = ("cave-wall", "cave-stone")
let bridge = ("bridge", "dark-water")
let ford = ("ford", "shallow-water")
let across = Exits [ "e"; "w" ]

let adjacent (col, row) =
  let west = col - 1 + (row mod 2) in
  [
    ("e", (col + 1, row));
    ("se", (west + 1, row + 1));
    ("sw", (west, row + 1));
    ("w", (col - 1, row));
    ("nw", (west, row - 1));
    ("ne", (west + 1, row - 1));
  ]

let distance (col, row) (other_col, other_row) =
  let dx = col - (row / 2) - other_col + (other_row / 2) and dy = row - other_row in
  max (abs (dx + dy)) (max (abs dx) (abs dy))

let terrain t point =
  Option.map
    (fun { original; paint } -> Option.fold ~none:original ~some:(fun p -> fst p.material) paint)
    (Hashtbl.find_opt t.cells point)

let editable t point =
  match Hashtbl.find_opt t.cells point with Some { paint = Some _; _ } -> true | _ -> false

let walkable t point =
  match terrain t point with
  | None | Some ("water" | "mountain" | "cave-wall") -> false
  | Some _ -> true

let wet t point =
  match terrain t point with
  | Some ("water" | "shallow-water" | "bridge" | "ford") -> true
  | _ -> false

let change t point update =
  match Hashtbl.find_opt t.cells point with
  | Some ({ paint = Some previous; _ } as cell) -> cell.paint <- Some (update previous)
  | _ -> ()

let paint ?(geometry = Surface) t point material =
  change t point (fun previous -> { previous with material; geometry })

let feature_error feature =
  invalid_arg
    {j|The template cannot fit the planned $(feature). Keep an editable footprint around it, or use another seed or facing.|j}

let require ?(geometry = Surface) t feature point material =
  if not (editable t point) then feature_error feature;
  paint ~geometry t point material

let mark t role point =
  if not (editable t point) then feature_error role;
  let rec free suffix =
    let id = if suffix = 1 then "draft-" ^ role else "draft-" ^ role ^ "-" ^ string_of_int suffix in
    if Hashtbl.mem t.ids id then free (suffix + 1) else id
  in
  let id = free 1 in
  Hashtbl.add t.ids id ();
  change t point (fun previous -> { previous with landmark = Some id })

let iter t f =
  (* Use stable spatial order for all generation and choice of candidate sites. *)
  for row = 0 to t.rows - 1 do
    for col = 0 to t.cols - 1 do
      let point = (col, row) in
      if Hashtbl.mem t.cells point then f point
    done
  done

let export t f = Hashtbl.iter (fun point cell -> Option.iter (f point) cell.paint) t.cells
let local_col t col = match t.facing with East -> col | West -> t.cols - 1 - col
let forward t = match t.facing with East -> 1 | West -> -1
let xfrac t (col, _) = float_of_int (local_col t col) /. float_of_int (t.cols - 1)
let yfrac t (_, row) = float_of_int row /. float_of_int (t.rows - 1)
let index span fraction = int_of_float (Float.round (fraction *. float_of_int (span - 1)))
let at t x y = (local_col t (index t.cols x), index t.rows y)
let row t y = index t.rows y
let noise t (col, row) = t.fields ~col ~row
let vary t salt = t.variation (salt +. 0.37) (0.68 +. (salt *. 0.27))

let jitter t salt ~x ~y ~dx ~dy =
  at t (x +. (dx *. vary t salt)) (y +. (dy *. vary t (salt +. 8.31)))

let nearest t feature anchor ~within ~allowed =
  let best = ref None in
  iter t (fun point ->
      let d = distance anchor point in
      if d <= within && allowed point then
        match !best with Some (cost, _) when cost <= d -> () | _ -> best := Some (d, point));
  match !best with Some (_, point) -> point | None -> feature_error feature

let region t anchor radius visit =
  let col, row = anchor in
  for y = max 0 (row - radius) to min (t.rows - 1) (row + radius) do
    for x = max 0 (col - radius) to min (t.cols - 1) (col + radius) do
      let point = (x, y) in
      if Hashtbl.mem t.cells point && distance anchor point <= radius then visit point
    done
  done

let connected starts ~allowed =
  let reached = Hashtbl.create 32 in
  let frontier = Queue.create () in
  let visit point =
    if allowed point && not (Hashtbl.mem reached point) then (
      Hashtbl.add reached point ();
      Queue.add point frontier)
  in
  List.iter visit starts;
  while not (Queue.is_empty frontier) do
    List.iter (fun (_, point) -> visit point) (adjacent (Queue.take frontier))
  done;
  reached

let validate_route t feature a b =
  let reached = connected [ a ] ~allowed:(walkable t) in
  if not (Hashtbl.mem reached b) then feature_error feature

let validate_cut t feature starts destination ~blocked =
  let reached = connected starts ~allowed:(fun p -> walkable t p && not (blocked p)) in
  if Hashtbl.mem reached destination then
    invalid_arg
      {j|The template provides a route around the planned $(feature). Remove the bypass or choose another layout.|j}

let validate_waterway t feature ~starts ~destination =
  let reached = connected starts ~allowed:(wet t) in
  if not (destination reached) then feature_error feature

module Frontier = Set.Make (struct
  type t = float * float * point

  let compare = compare
end)

let route feature ~allowed ~cost start finish =
  if not (allowed start && allowed finish) then feature_error feature;
  let scores = Hashtbl.create 64 and previous = Hashtbl.create 64 in
  let estimate point = float_of_int (distance point finish) in
  let frontier = ref (Frontier.singleton (estimate start, 0., start)) in
  Hashtbl.add scores start 0.;
  let rec search () =
    match Frontier.min_elt_opt !frontier with
    | None -> feature_error feature
    | Some ((_, score, point) as next) ->
        frontier := Frontier.remove next !frontier;
        if point = finish then ()
        else (
          if Hashtbl.find scores point = score then
            List.iter
              (fun (_, candidate) ->
                if allowed candidate then
                  let next_score = score +. cost candidate in
                  let old = Option.value ~default:infinity (Hashtbl.find_opt scores candidate) in
                  if next_score < old then (
                    Hashtbl.replace scores candidate next_score;
                    Hashtbl.replace previous candidate point;
                    frontier :=
                      Frontier.add
                        (next_score +. estimate candidate, next_score, candidate)
                        !frontier))
              (adjacent point);
          search ())
  in
  search ();
  let rec trace point path =
    let path = point :: path in
    if point = start then path else trace (Hashtbl.find previous point) path
  in
  trace finish []

let path t feature ~allowed start finish =
  let cost point =
    let height, moisture = noise t point in
    1. +. (2.8 *. abs_float (height -. 0.18)) +. (0.45 *. abs_float moisture)
  in
  route feature ~allowed ~cost start finish

let roads ?(floor = snd road) t paths =
  let links = Hashtbl.create 64 in
  let add from destination =
    let neighbors = Option.value ~default:[] (Hashtbl.find_opt links from) in
    Hashtbl.replace links from (destination :: neighbors)
  in
  let rec connect = function
    | a :: (b :: _ as tail) ->
        add a b;
        add b a;
        connect tail
    | _ -> ()
  in
  List.iter connect paths;
  Hashtbl.iter
    (fun point links ->
      let directions =
        List.filter_map (fun (d, p) -> if List.mem p links then Some d else None) (adjacent point)
      in
      match terrain t point with
      | Some ("bridge" | "ford" | "landing" | "ferry" | "castle-wall" | "castle-bell") -> ()
      | _ -> paint t point (fst road, floor) ~geometry:(Exits directions))
    links

let cave_openings t ~allowed =
  iter t (fun point ->
      if editable t point && terrain t point = Some (fst cave_floor) then
        let directions =
          List.filter_map
            (fun (direction, next) -> if allowed next then Some direction else None)
            (adjacent point)
        in
        paint t point cave_floor ~geometry:(Exits directions))

let edited_land t p = editable t p && walkable t p && not (wet t p)

let land_site t feature anchor radius =
  nearest t feature anchor ~within:radius ~allowed:(edited_land t)

let settlement t feature point =
  require t feature point ("village", "green-plain") ~geometry:(Exits []);
  mark t feature point

let open_terrain t point =
  let height, moisture = noise t point in
  if height > 0.40 then mountain
  else if height > 0.19 then hill
  else if moisture > 0.06 then forest
  else plains

let arid_terrain t point =
  let height, moisture = noise t point in
  if height > 0.41 then mountain
  else if height > 0.17 then ("hill", "autumn-forest")
  else if moisture > 0.39 then ("plains", "mountain-rock")
  else ("plains", "autumn-forest")

let gate_approach (col, row) radius direction =
  [ (col + (direction * radius), row); (col + (direction * (radius + 1)), row) ]

let castle t ~radius center =
  let x, y = center in
  let inside point = distance center point <= radius in
  let gate direction (col, row) =
    row = y && ((direction = "w" && col = x - radius) || (direction = "e" && col = x + radius))
  in
  let count = ref 0 in
  region t center radius (fun point ->
      if not (edited_land t point) then feature_error "fortress courtyard";
      incr count);
  if !count <> 1 + (3 * radius * (radius + 1)) then feature_error "fortress courtyard";
  region t center radius (fun point ->
      let walls =
        if point = center then List.map fst (adjacent point)
        else
          List.filter_map
            (fun (d, next) -> if inside next || gate d point then None else Some d)
            (adjacent point)
      in
      let terrain = if point = center then "castle-bell" else "castle-wall" in
      paint t point (terrain, "castle-paving") ~geometry:(Walls walls));
  mark t "fortress" center
