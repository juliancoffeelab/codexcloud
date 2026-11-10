(* A small, immutable ambience field, rebuilt only when authored cells change.
   One texel spans four native pixels; CPU and GPU sample identical bytes. *)
type mask = { x : float; y : float; width : int; height : int; step : float; data : float array }

let serialize_key cells =
  Array.to_list
    (Array.map
       (fun (cell : Domain.hex) -> Int32.to_string cell.col ^ "," ^ Int32.to_string cell.row)
       cells)
  |> String.concat ";"

let last_cells : (Domain.hex array * string) option ref = ref None

let key cells =
  match !last_cells with
  | Some (previous, key) when previous == cells -> key
  | _ ->
      let key = serialize_key cells in
      last_cells := Some (cells, key);
      key

let cached : (string * mask option) option ref = ref None
let round x = floor (x +. 0.5)

let build cells =
  if Array.length cells = 0 then None
  else
    let positions = Hashtbl.create (Array.length cells) in
    let left = ref infinity
    and top = ref infinity
    and right = ref neg_infinity
    and bottom = ref neg_infinity in
    Array.iter
      (fun (cell : Domain.hex) ->
        let col = Int32.to_int cell.col and row = Int32.to_int cell.row in
        Hashtbl.replace positions (col, row) ();
        let x = 39. +. (56. *. (float_of_int col +. (float_of_int (row mod 2) /. 2.)))
        and y = 38. +. (48. *. float_of_int row) in
        left := min !left (x -. 44.);
        right := max !right (x +. 44.);
        top := min !top (y -. 48.);
        bottom := max !bottom (y +. 48.))
      cells;
    let x = floor (!left /. 4.) *. 4. and y = floor (!top /. 4.) *. 4. in
    let width = int_of_float (ceil ((!right -. x) /. 4.))
    and height = int_of_float (ceil ((!bottom -. y) /. 4.)) in
    let data =
      Array.init (width * height) (fun i ->
          let px = x +. ((float_of_int (i mod width) +. 0.5) *. 4.)
          and py = y +. ((float_of_int (i / width) +. 0.5) *. 4.) in
          let r = (py -. 38.) /. 48. in
          let q = ((px -. 39.) /. 56.) -. (r /. 2.) in
          let s = -.q -. r in
          let rq = round q and rr = round r and rs = round s in
          let dq = abs_float (rq -. q) and dr = abs_float (rr -. r) and ds = abs_float (rs -. s) in
          let rq, rr =
            if dq > dr && dq > ds then (-.rr -. rs, rr)
            else if dr > ds then (rq, -.rq -. rs)
            else (rq, rr)
          in
          let col = int_of_float (rq +. floor (rr /. 2.)) and row = int_of_float rr in
          if Hashtbl.mem positions (col, row) then 255. else 0.)
    in
    let work = Array.make (Array.length data) 0. in
    for _ = 1 to 2 do
      for row = 0 to height - 1 do
        for col = 0 to width - 1 do
          let i = (row * width) + col in
          work.(i) <-
            (data.((row * width) + max 0 (col - 1))
            +. (2. *. data.(i))
            +. data.((row * width) + min (width - 1) (col + 1)))
            /. 4.
        done
      done;
      for row = 0 to height - 1 do
        for col = 0 to width - 1 do
          let i = (row * width) + col in
          data.(i) <-
            (work.((max 0 (row - 1) * width) + col)
            +. (2. *. work.(i))
            +. work.((min (height - 1) (row + 1) * width) + col))
            /. 4.
        done
      done
    done;
    Array.iteri (fun i value -> data.(i) <- round value) data;
    let scale = Geometry.native_pixel_scale in
    Some { x = x /. scale; y = y /. scale; width; height; step = 4. /. scale; data }

let get cells =
  let signature = key cells in
  match !cached with
  | Some (old, mask) when old = signature -> mask
  | _ ->
      let mask = build cells in
      cached := Some (signature, mask);
      mask

let sample mask x y =
  match mask with
  | None -> 0.
  | Some mask ->
      let x = ((x -. mask.x) /. mask.step) -. 0.5 and y = ((y -. mask.y) /. mask.step) -. 0.5 in
      let col = int_of_float (floor x) and row = int_of_float (floor y) in
      if col < 0 || row < 0 || col + 1 >= mask.width || row + 1 >= mask.height then 0.
      else
        let fx = x -. floor x and fy = y -. floor y in
        let at col row = mask.data.((row * mask.width) + col) in
        ((((at col row *. (1. -. fx)) +. (at (col + 1) row *. fx)) *. (1. -. fy))
        +. (((at col (row + 1) *. (1. -. fx)) +. (at (col + 1) (row + 1) *. fx)) *. fy))
        /. 255.
