(* 2D simplex noise. Hash lattice corners directly so the field is independent
   of tile traversal order and works with the drafter's clipped maps. *)
let simplex seed =
  let gradients =
    [| (1., 1.); (-1., 1.); (1., -1.); (-1., -1.); (1., 0.); (-1., 0.); (0., 1.); (0., -1.) |]
  in
  let skew = (sqrt 3. -. 1.) /. 2. and unskew = (3. -. sqrt 3.) /. 6. in
  let gradient i j =
    let open Int32 in
    let h = logxor seed (logxor (mul (of_int i) 0x9e3779b1l) (mul (of_int j) 0x85ebca77l)) in
    let h = mul (logxor h (shift_right_logical h 16)) 0x85ebca6bl in
    let h = mul (logxor h (shift_right_logical h 13)) 0xc2b2ae35l in
    gradients.(to_int (logand (logxor h (shift_right_logical h 16)) 7l))
  in
  let corner i j x y =
    let weight = 0.5 -. (x *. x) -. (y *. y) in
    if weight <= 0. then 0.
    else
      let gx, gy = gradient i j in
      let w2 = weight *. weight in
      w2 *. w2 *. ((gx *. x) +. (gy *. y))
  in
  fun x y ->
    let s = (x +. y) *. skew in
    let i = int_of_float (floor (x +. s)) and j = int_of_float (floor (y +. s)) in
    let t = float_of_int (i + j) *. unskew in
    let x0 = x -. float_of_int i +. t and y0 = y -. float_of_int j +. t in
    let di, dj = if x0 > y0 then (1, 0) else (0, 1) in
    70.
    *. (corner i j x0 y0
       +. corner (i + di) (j + dj)
            (x0 -. float_of_int di +. unskew)
            (y0 -. float_of_int dj +. unskew)
       +. corner (i + 1) (j + 1) (x0 -. 1. +. (2. *. unskew)) (y0 -. 1. +. (2. *. unskew)))

let fields ~seed ~cols ~rows =
  let elevation = simplex seed and moisture = simplex (Int32.logxor seed 0x68bc21ebl) in
  let frequency = 3. /. float_of_int (max 12 (min cols rows)) in
  fun ~col ~row ->
    let center = Geometry.center ~col ~row in
    let x = center.x *. frequency /. Geometry.column_spacing
    and y = center.y *. frequency /. Geometry.column_spacing in
    let height =
      (0.72 *. elevation x y)
      +. (0.21 *. elevation ((2. *. x) +. 19.) (2. *. y))
      +. (0.07 *. elevation (4. *. x) ((4. *. y) +. 31.))
    in
    (height, moisture (0.83 *. x) (0.83 *. y))
