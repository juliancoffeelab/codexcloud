type pose = { left : float; top : float; zoom : float }
type size = { width : float; height : float }
type bounds = { left : float; top : float; right : float; bottom : float }
type layout = { map : size; base_width : float; capacity_width : float; viewport : size }
type snapshot = { pose : pose; layout : layout; content : size; display_scale : float }
type navigation = Pan | Fit
type t = { mutable view : snapshot; mutable bounds : bounds; mutable initialized : bool }

let clamp value low high = max low (min high value)

let resolve layout (pose : pose) =
  let width = layout.base_width *. pose.zoom in
  let display_scale = width /. layout.map.width in
  { pose; layout; content = { width; height = layout.map.height *. display_scale }; display_scale }

let origin = { left = 0.; top = 0.; zoom = 1. }

let create () =
  let unit_size = { width = 1.; height = 1. } in
  {
    view =
      resolve { map = unit_size; base_width = 1.; capacity_width = 0.; viewport = unit_size } origin;
    bounds = { left = 0.; top = 0.; right = 1.; bottom = 1. };
    initialized = false;
  }

let snapshot t = t.view
let initialized t = t.initialized

let configure t layout =
  let old = t.view in
  let next = resolve layout old.pose in
  let pose =
    if t.initialized && old.layout.capacity_width <> layout.capacity_width then
      let center = (old.pose.left +. (old.layout.capacity_width /. 2.)) /. old.display_scale in
      { old.pose with left = (center *. next.display_scale) -. (layout.capacity_width /. 2.) }
    else old.pose
  in
  t.view <- { next with pose };
  t.initialized <- true

let set_visible_bounds t bounds = t.bounds <- bounds

let constrained t navigation (next : snapshot) =
  let b = t.bounds and current = t.view.pose in
  let minimum_left = -.b.left and minimum_top = -.b.top in
  let maximum_left = max minimum_left (next.content.width -. b.right)
  and maximum_top = max minimum_top (next.content.height -. b.bottom) in
  let limit value minimum maximum old =
    match navigation with
    | Fit -> clamp value minimum maximum
    | Pan -> clamp value (min minimum old) (max maximum old)
  in
  {
    next with
    pose =
      {
        next.pose with
        left = limit next.pose.left minimum_left maximum_left current.left;
        top = limit next.pose.top minimum_top maximum_top current.top;
      };
  }

let move_to t navigation ~left ~top =
  let next = { t.view with pose = { t.view.pose with left; top } } in
  t.view <- constrained t navigation next

let pan_by t ~dx ~dy = move_to t Pan ~left:(t.view.pose.left +. dx) ~top:(t.view.pose.top +. dy)

let zoom_at t ~zoom ~x ~y =
  let old = t.view.pose in
  let zoom = clamp zoom 0.5 3. in
  if zoom = old.zoom then false
  else
    let ratio = zoom /. old.zoom in
    let pose =
      { zoom; left = ((old.left +. x) *. ratio) -. x; top = ((old.top +. y) *. ratio) -. y }
    in
    t.view <- constrained t Fit (resolve t.view.layout pose);
    true

let reset t =
  t.view <- resolve t.view.layout origin;
  t.initialized <- false
