open Browser_time
module D = Phaser_input_dom
module Set = Host_collections.Set

let clamp value low high = max low (min high value)
let int = Performance_trace.int
let number = Js.Json.number
let bool = Js.Json.boolean
let string = Js.Json.string
let obj = Performance_trace.obj

type clamp_mode = Pan | Fit

type drag = {
  pointer : Phaser.pointer;
  mutable x : float;
  mutable y : float;
  left : float;
  top : float;
  mutable last_x : float;
  mutable last_y : float;
  mutable time : float;
  mutable vx : float;
  mutable vy : float;
}

type pinch = { distance : float; zoom : float; x : float; y : float }
type reveal = { ids : string array; center : bool }

class input (scene : Phaser.scene) (current : unit -> Battlefield_frame.t option)
  (actions : unit -> Battlefield_view.actions) (invalidate : unit -> unit) =
  let camera = scene##cameras##main in
  let picking = new Phaser_picking.picking camera in
  let canvas =
    match Js.Nullable.toOption scene##game##canvas with
    | Some canvas -> canvas
    | None -> failwith {js|Battlefield canvas is unavailable|js}
  in
  let host =
    match Js.Nullable.toOption canvas##parentElement with
    | Some host -> host
    | None -> failwith {js|Battlefield host is unavailable|js}
  in
  let workspace = Js.Nullable.toOption (D.closest host {js|.workspace|js}) in
  let renderer =
    match Js.Nullable.toOption scene##game##renderer with
    | Some renderer -> renderer
    | None -> failwith {js|Battlefield WebGL renderer is unavailable|js}
  in
  let viewport = Webgl.viewport_array (Webgl.viewport_parameter renderer##gl 0x0d3a) in
  let surface_limit =
    float_of_int
      (Array.fold_left min
         (min 4096
            (min (Webgl.parameter renderer##gl 0x0d33) (Webgl.parameter renderer##gl 0x84e8)))
         viewport)
  in
  object (self)
    val mutable zoom = 1.
    val mutable display_scale = 1.
    val mutable left = 0.
    val mutable top = 0.
    val mutable pending_top : float option = None
    val mutable viewport_width = 1.
    val mutable viewport_height = 1.
    val mutable content_width = 1.
    val mutable content_height = 1.
    val mutable pixel_x = 1.
    val mutable pixel_y = 1.
    val mutable layout_pending = true
    val mutable layout_ready = false
    val mutable previous_capacity = 0.
    val mutable previous_zoom = 1.
    val mutable pending_reveal : reveal option = None
    val mutable inspection = {js||js}
    val mutable inspection_view : Battlefield_view.view option = None
    val mutable surface_width = 0
    val mutable surface_height = 0
    val mutable density = 0.
    val mutable commander_game : Native.view option = None
    val mutable commander_tile : Battlefield_frame.tile option = None
    val mutable commander_hint : int option option = None
    val view_origin = Phaser.Geom.Vector2.create ()
    val pick_point = Phaser.Geom.Vector2.create ()
    val mutable frame_delta = 0.
    val mutable gesture_id = 0
    val mutable sample_trace = 0
    val mutable last_sample : (float * float * float * float * float) option = None
    val mutable gesture = false
    val mutable cancelled_contacts = false
    val mutable inertia : D.tween option = None
    val mutable drag : drag option = None
    val mutable pinch : pinch option = None
    val mutable visible_dirty = true
    val mutable visible = { Phaser_spatial.left = 0.; top = 0.; right = 1.; bottom = 1. }
    val mutable projection : float array option = None
    val mutable detach = fun () -> ()

    method private pointers =
      Array_ext.filter
        (fun (pointer : Phaser.pointer) -> pointer##isDown)
        scene##input##manager##pointers

    method private record ?(details = []) event =
      if Option.is_some (Performance_trace.configuration ()) then
        let frame = current () in
        let fields =
          [
            ({js|gestureId|js}, int gesture_id);
            ({js|observedAt|js}, number (now ()));
            ( {js|activePointerIds|js},
              Js.Json.array
                (Array.map (fun (pointer : Phaser.pointer) -> int pointer##id) self#pointers) );
            ({js|cancelledContacts|js}, bool cancelled_contacts);
            ({js|left|js}, number left);
            ({js|top|js}, number top);
            ({js|zoom|js}, number zoom);
            ({js|viewportWidth|js}, number viewport_width);
            ({js|viewportHeight|js}, number viewport_height);
            ({js|contentWidth|js}, number content_width);
            ({js|contentHeight|js}, number content_height);
          ]
        in
        let fields =
          match frame with
          | None -> fields
          | Some frame ->
              ({js|mapId|js}, string (Native.state frame.view.game).map_id)
              :: ({js|sessionVersion|js}, int frame.view.session_version)
              :: fields
        in
        Performance_trace.instant {j|Battlefield $(event)|j} ~args:(obj (details @ fields))

    method private pointer_record event (pointer : Phaser.pointer) extra =
      if Option.is_some (Performance_trace.configuration ()) then
        let point = D.position pointer in
        self#record event
          ~details:
            ([
               ({js|pointerId|js}, int pointer##id);
               ({js|touch|js}, bool pointer##wasTouch);
               ({js|x|js}, number point.x);
               ({js|y|js}, number point.y);
               ({js|eventTime|js}, number (D.event_time pointer##event));
             ]
            @ extra)

    method private stop_inertia reason =
      if Option.is_some inertia then
        self#record {js|inertia stop|js} ~details:[ ({js|reason|js}, string reason) ];
      Option.iter D.stop_tween inertia;
      inertia <- None

    method private visible_bounds =
      if visible_dirty then (
        visible_dirty <- false;
        let canvas_bounds = D.canvas_bounds canvas in
        let left = ref 0.
        and top = ref 0.
        and right = ref viewport_width
        and bottom = ref viewport_height in
        let rec ancestors = function
          | None -> ()
          | Some node ->
              let style = D.computed_style node in
              let clips overflow =
                List.mem overflow [ {js|auto|js}; {js|scroll|js}; {js|hidden|js}; {js|clip|js} ]
              in
              let x = clips (D.overflow_x style) and y = clips (D.overflow_y style) in
              if x || y then (
                let bounds = D.bounds node in
                if x then (
                  left :=
                    max !left
                      (bounds.left +. float_of_int (D.client_left node) -. canvas_bounds.left);
                  right :=
                    min !right
                      (bounds.left
                      +. float_of_int (D.client_left node + D.width node)
                      -. canvas_bounds.left));
                if y then (
                  top :=
                    max !top (bounds.top +. float_of_int (D.client_top node) -. canvas_bounds.top);
                  bottom :=
                    min !bottom
                      (bounds.top
                      +. float_of_int (D.client_top node + D.height node)
                      -. canvas_bounds.top)));
              ancestors (Js.Nullable.toOption (D.parent node))
        in
        ancestors (Js.Nullable.toOption (D.parent host));
        visible <- { left = !left; top = !top; right = !right; bottom = !bottom });
      visible

    method private vertical_limit =
      let bounds = self#visible_bounds in
      max (-.bounds.top) (content_height -. bounds.bottom)

    method private apply_scroll clamp_scroll =
      (* Clipping changes visibility only. Clamp on navigation, not on panel layout. *)
      Option.iter
        (fun mode ->
          let bounds = self#visible_bounds in
          (* Viewport growth can leave the preserved pose outside its new limits.
           Panning may return inward, but must not snap that pose into bounds. *)
          let current_left =
            if mode = Pan then camera##scrollX *. camera##zoomX /. pixel_x else -.bounds.left
          in
          let current_top =
            if mode = Pan then camera##scrollY *. camera##zoomY /. pixel_y else -.bounds.top
          in
          left <-
            clamp left
              (min (-.bounds.left) current_left)
              (max (max (-.bounds.left) (content_width -. bounds.right)) current_left);
          top <- clamp top (min (-.bounds.top) current_top) (max self#vertical_limit current_top))
        clamp_scroll;
      let x = left *. pixel_x /. camera##zoomX and y = top *. pixel_y /. camera##zoomY in
      let next = [| x; y; camera##zoomX; camera##zoomY; camera##width; camera##height |] in
      if not (Option.fold ~none:false ~some:(Array_ext.equal Float.equal next) projection) then (
        projection <- Some next;
        let _ = camera##setOrigin 0. 0. in
        let _ = camera##setScroll x y in
        Phaser.CameraMethods.pre_render camera;
        invalidate ())

    method private move_vertical ?(mode = Fit) next =
      top <- next;
      self#apply_scroll (Some mode)

    method private perform_layout =
      visible_dirty <- true;
      Option.iter
        (fun (frame : Battlefield_frame.t) ->
          let state = Native.state frame.view.game in
          let size =
            Geometry.dimensions ~cols:(Int32.to_int state.cols) ~rows:(Int32.to_int state.rows)
          in
          let old_width = viewport_width and old_height = viewport_height in
          let world_center_x = (left +. (previous_capacity /. 2.)) /. display_scale in
          viewport_width <- float_of_int (D.width host);
          let minimum =
            (if D.inner_width <= 540 then 650. else 590.) *. Int32.to_float state.cols /. 11.
          in
          let capacity_width =
            max 1.
              (Option.fold
                 ~none:(float_of_int (D.width host) +. 2.)
                 ~some:(fun node -> float_of_int (D.width node))
                 workspace
              -. 2.)
          and capacity_height =
            max 1.
              (Option.fold
                 ~none:(float_of_int (D.height host) +. 2.)
                 ~some:(fun node -> float_of_int (D.height node))
                 workspace
              -. 2.)
          in
          let base_width = max capacity_width minimum in
          content_width <- base_width *. zoom;
          display_scale <- content_width /. size.width;
          content_height <- size.height *. display_scale;
          viewport_height <- min (float_of_int (D.height host)) capacity_height;
          if
            layout_ready
            && (previous_capacity <> capacity_width
               || old_width <> viewport_width || density <> D.device_ratio
               || (old_height <> viewport_height && previous_zoom = zoom))
          then self#cancel {js|viewport layout|js};
          if layout_ready && previous_capacity <> capacity_width then
            left <- (world_center_x *. display_scale) -. (capacity_width /. 2.);
          previous_capacity <- capacity_width;
          previous_zoom <- zoom;
          layout_ready <- true;
          let ratio =
            min D.device_ratio
              (min
                 (sqrt (4194304. /. (capacity_width *. capacity_height)))
                 (min (surface_limit /. capacity_width) (surface_limit /. capacity_height)))
          in
          let pixels_wide = int_of_float (ceil (capacity_width *. ratio))
          and pixels_high = int_of_float (ceil (capacity_height *. ratio)) in
          pixel_x <- float_of_int pixels_wide /. capacity_width;
          pixel_y <- float_of_int pixels_high /. capacity_height;
          if surface_width <> pixels_wide || surface_height <> pixels_high then (
            let _ = scene##scale##resize pixels_wide pixels_high in
            surface_width <- pixels_wide;
            surface_height <- pixels_high;
            Performance_trace.instant {js|Battlefield surface allocation|js}
              ~args:
                (obj
                   [
                     ({js|width|js}, int pixels_wide);
                     ({js|height|js}, int pixels_high);
                     ({js|density|js}, number D.device_ratio);
                   ]));
          density <- D.device_ratio;
          D.css_width canvas##style
            (let js_text_part_0 = Js.Float.toString capacity_width in
             {j|$(js_text_part_0)px|j});
          D.css_height canvas##style
            (let js_text_part_0 = Js.Float.toString capacity_height in
             {j|$(js_text_part_0)px|j});
          Phaser.ScaleMethods.set_size
            (Phaser.ScaleMethods.display_size scene##scale)
            capacity_width capacity_height;
          let _ = scene##scale##updateBounds () in
          Phaser.ScaleMethods.set_vector
            (Phaser.ScaleMethods.display_scale scene##scale)
            pixel_x pixel_y;
          let _ =
            camera##setViewport 0. 0.
              (ceil (viewport_width *. pixel_x))
              (min (float_of_int pixels_high) (ceil (viewport_height *. pixel_y)))
          in
          Phaser.CameraMethods.remove_bounds camera;
          Performance_trace.instant {js|Battlefield viewport|js}
            ~args:
              (obj
                 [
                   ({js|width|js}, number viewport_width);
                   ({js|height|js}, number viewport_height);
                   ({js|worldScale|js}, number display_scale);
                 ]);
          Phaser.CameraMethods.zoom camera
            (content_width *. pixel_x
            /. (ceil (size.width *. Geometry.native_pixel_scale) /. Geometry.native_pixel_scale))
            (content_height *. pixel_y
            /. (ceil (size.height *. Geometry.native_pixel_scale) /. Geometry.native_pixel_scale));
          self#apply_scroll None;
          match pending_top with
          | None -> ()
          | Some next ->
              pending_top <- None;
              self#move_vertical next)
        (current ())

    method private refresh_bounds =
      let _ = scene##scale##updateBounds () in
      if layout_pending || density <> D.device_ratio then (
        layout_pending <- false;
        self#perform_layout);
      if layout_ready then self#apply_scroll None

    method private pan dx dy =
      left <- left +. dx;
      top <- top +. dy;
      self#apply_scroll (Some Pan)

    method private zoom_at next x y =
      pending_reveal <- None;
      let next = clamp next 0.5 3. in
      if next <> zoom then (
        let ratio = next /. zoom in
        left <- ((left +. x) *. ratio) -. x;
        pending_top <- Some (((top +. y) *. ratio) -. y);
        zoom <- next;
        layout_pending <- true)

    method private down (pointer : Phaser.pointer) =
      if cancelled_contacts || D.modal () then
        self#record {js|pointer rejected|js}
          ~details:
            [
              ( {js|reason|js},
                string (if cancelled_contacts then {js|cancelled contacts|js} else {js|modal|js}) );
              ({js|pointerId|js}, int pointer##id);
            ]
      else if pointer##wasTouch || pointer##leftButtonDown () || pointer##middleButtonDown () then (
        gesture_id <- gesture_id + 1;
        self#stop_inertia {js|pointer down|js};
        pending_reveal <- None;
        self#pointer_record {js|pointer down|js} pointer [];
        D.focus canvas [%mel.obj { preventScroll = true }];
        let active = self#pointers in
        if Array.length active >= 2 then (
          let a = D.position active.(0) and b = D.position active.(1) in
          let bounds = D.canvas_bounds canvas in
          let distance = hypot (a.x -. b.x) (a.y -. b.y) in
          pinch <-
            Some
              {
                distance;
                zoom;
                x = ((a.x +. b.x) /. 2.) -. bounds.left;
                y = ((a.y +. b.y) /. 2.) -. bounds.top;
              };
          self#record {js|pinch start|js} ~details:[ ({js|distance|js}, number distance) ];
          gesture <- true;
          drag <- None)
        else (
          gesture <- false;
          let point = D.position pointer in
          drag <-
            Some
              {
                pointer;
                x = point.x;
                y = point.y;
                left;
                top;
                last_x = point.x;
                last_y = point.y;
                time = D.event_time pointer##event;
                vx = 0.;
                vy = 0.;
              }))

    method private move (pointer : Phaser.pointer) resampled =
      if D.modal () then self#cancel {js|modal|js}
      else
        let active = self#pointers in
        if not resampled then self#pointer_record {js|pointer move|js} pointer [];
        match pinch with
        | Some pinch when Array.length active >= 2 ->
            let a = D.position active.(0) and b = D.position active.(1) in
            self#zoom_at
              (pinch.zoom *. hypot (a.x -. b.x) (a.y -. b.y) /. max 1. pinch.distance)
              pinch.x pinch.y
        | _ -> (
            match drag with
            | Some drag when pointer == drag.pointer && pointer##isDown ->
                let point = D.position pointer in
                let dx = ref (point.x -. drag.x) and dy = ref (point.y -. drag.y) in
                if gesture || hypot !dx !dy >= 6. then (
                  if (not gesture) && pointer##wasTouch then (
                    let distance = hypot !dx !dy in
                    drag.x <- drag.x +. (!dx /. distance *. 5.);
                    drag.y <- drag.y +. (!dy /. distance *. 5.);
                    dx := point.x -. drag.x;
                    dy := point.y -. drag.y);
                  if point.x <> drag.last_x || point.y <> drag.last_y then (
                    let at = D.event_time pointer##event in
                    let dt = max 8. (at -. drag.time) in
                    drag.vx <- (point.x -. drag.last_x) /. dt;
                    drag.vy <- (point.y -. drag.last_y) /. dt;
                    drag.last_x <- point.x;
                    drag.last_y <- point.y;
                    drag.time <- at);
                  gesture <- true;
                  D.cursor canvas##style {js|grabbing|js};
                  left <- drag.left -. !dx;
                  top <- drag.top -. !dy;
                  self#apply_scroll (Some Pan))
            | _ -> ())

    method private prepare_picking (frame : Battlefield_frame.t) =
      let state = Native.state frame.view.game in
      let key =
        Js.Array.join ~sep:{js|:|js}
          (Array.of_list
             [
               string_of_int frame.view.session_version;
               state.map_id;
               Js.Float.toString state.map_revision;
               Int32.to_string state.cols;
               Int32.to_string state.rows;
               string_of_int (Array.length frame.tiles);
             ])
      in
      picking#prepare key frame.terrain_tiles

    method private up (pointer : Phaser.pointer) =
      self#pointer_record {js|pointer up|js} pointer
        [
          ({js|cancelled|js}, bool pointer##wasCanceled);
          ({js|dragged|js}, bool gesture);
          ({js|contacts|js}, int (Array.length self#pointers));
        ];
      if cancelled_contacts then (
        if Array.length self#pointers = 0 then cancelled_contacts <- false)
      else (
        (match current () with
        | Some frame
          when (not (D.modal ()))
               && (not gesture)
               && (not pointer##wasCanceled)
               && pointer##getDistance () /. max pixel_x pixel_y < 6.
               && (pointer##wasTouch || pointer##button = 0) -> (
            if layout_pending || density <> D.device_ratio then (
              layout_pending <- false;
              self#perform_layout);
            self#apply_scroll None;
            let _ = camera##getWorldPoint pointer##x pointer##y pick_point in
            self#prepare_picking frame;
            match picking#pick pick_point##x pick_point##y with
            | Some tile -> (actions ()).choose tile
            | None -> (actions ()).clear_selection true)
        | _ -> ());
        if Array.length self#pointers = 0 then (
          (match (drag, pinch) with
          | Some drag, None
            when pointer##wasTouch
                 && (not pointer##wasCanceled)
                 && gesture
                 && D.event_time pointer##event -. drag.time < 160. ->
              let vx = -.drag.vx and vy = -.drag.vy in
              let speed = hypot vx vy in
              if speed > 0.1 then (
                let duration = min 900. (speed /. 0.004) and from_left = left and from_top = top in
                self#record {js|inertia start|js}
                  ~details:
                    [
                      ({js|vx|js}, number vx);
                      ({js|vy|js}, number vy);
                      ({js|duration|js}, number duration);
                    ];
                inertia <-
                  Some
                    (D.counter scene##tweens
                       {
                         D.from = 0.;
                         target = 1.;
                         duration;
                         ease = {js|Quad.easeOut|js};
                         on_update =
                           (fun[@u] tween ->
                             let progress =
                               Option.value ~default:0. (Js.Nullable.toOption (D.tween_value tween))
                             in
                             left <- from_left +. (vx *. duration /. 2. *. progress);
                             top <- from_top +. (vy *. duration /. 2. *. progress);
                             self#apply_scroll (Some Pan));
                         on_complete =
                           (fun[@u] () ->
                             self#record {js|inertia complete|js};
                             inertia <- None);
                       }))
          | _ -> ());
          drag <- None;
          pinch <- None;
          gesture <- false;
          D.cursor canvas##style {js||js})
        else drag <- None)

    method private wheel (pointer : Phaser.pointer) dx dy =
      let event = pointer##event in
      self#record {js|wheel|js}
        ~details:
          [
            ({js|dx|js}, number dx);
            ({js|dy|js}, number dy);
            ({js|deltaMode|js}, int (D.delta_mode event));
            ({js|zoomGesture|js}, bool (D.ctrl event || D.meta event));
          ];
      self#cancel {js|wheel|js};
      pending_reveal <- None;
      D.prevent event;
      if D.ctrl event || D.meta event then
        self#zoom_at (zoom *. exp (-.dy *. 0.002)) (pointer##x /. pixel_x) (pointer##y /. pixel_y)
      else
        let unit_ = match D.delta_mode event with 1 -> 16. | 2 -> viewport_height | _ -> 1. in
        if D.shift event && dx = 0. then self#pan (dy *. unit_) 0.
        else self#pan (dx *. unit_) (dy *. unit_)

    method private key event =
      if
        Option.fold ~none:false
          ~some:(fun element -> element == Phaser.Upcast.canvas_element canvas)
          (Js.Nullable.toOption D.active_element)
        && (not (D.modal ()))
        && Option.is_some (current ())
        && not (D.ctrl event || D.meta event || D.alt event)
      then
        let key = D.key event in
        if List.mem key [ {js|Escape|js}; {js|+|js}; {js|=|js}; {js|-|js}; {js|0|js} ] then (
          self#cancel {js|keyboard shortcut|js};
          pending_reveal <- None;
          D.prevent event;
          match key with
          | matched_text_0 when matched_text_0 = {js|Escape|js} -> (actions ()).clear_selection true
          | matched_text_1 when matched_text_1 = {js|+|js} || matched_text_1 = {js|=|js} ->
              self#zoom_at (zoom *. 1.2) (viewport_width /. 2.) (viewport_height /. 2.)
          | matched_text_2 when matched_text_2 = {js|-|js} ->
              self#zoom_at (zoom /. 1.2) (viewport_width /. 2.) (viewport_height /. 2.)
          | _ -> self#zoom_at 1. (viewport_width /. 2.) (viewport_height /. 2.))

    method private cancel reason =
      self#record {js|gesture cancel|js}
        ~details:
          [
            ({js|reason|js}, string reason);
            ({js|contacts|js}, int (Array.length self#pointers));
            ({js|dragging|js}, bool (Option.is_some drag));
            ({js|pinching|js}, bool (Option.is_some pinch));
          ];
      self#stop_inertia reason;
      cancelled_contacts <- Array.length self#pointers > 0;
      drag <- None;
      pinch <- None;
      gesture <- true;
      D.cursor canvas##style {js||js}

    method private apply_reveal request =
      Option.iter
        (fun (frame : Battlefield_frame.t) ->
          let clip = self#visible_bounds in
          let tiles =
            Array_ext.filter_map
              (fun id ->
                Array.find_opt
                  (fun (tile : Battlefield_frame.tile) -> tile.terrain.id = id)
                  frame.tiles)
              request.ids
          in
          if Array.length tiles = 0 then (
            if request.center then (
              left <- (content_width -. clip.left -. clip.right) /. 2.;
              self#move_vertical ((content_height -. clip.top -. clip.bottom) /. 2.)))
          else
            let scale_x = camera##zoomX /. pixel_x and scale_y = camera##zoomY /. pixel_y in
            let bounds =
              Array.map
                (fun (tile : Battlefield_frame.tile) ->
                  let visible_unit =
                    Option.fold ~none:false
                      ~some:(fun (occupant : Battlefield_frame.unit_view) ->
                        not
                          (Native.fogged frame.view.game tile.terrain.id
                          && occupant.faction.relation = Domain.Relation.Hostile))
                      tile.occupant
                  in
                  let radius = if visible_unit then Geometry.sprite_size /. 2. else 30.625 in
                  {
                    Phaser_spatial.left = (tile.terrain.x -. radius) *. scale_x;
                    right = (tile.terrain.x +. radius) *. scale_x;
                    top =
                      (tile.terrain.y -. if visible_unit then Geometry.sprite_size -. 20. else 35.)
                      *. scale_y;
                    bottom = (tile.terrain.y +. 35.) *. scale_y;
                  })
                tiles
            in
            let last = bounds.(Array.length bounds - 1) in
            let x1 = ref infinity
            and x2 = ref neg_infinity
            and y1 = ref infinity
            and y2 = ref neg_infinity in
            Array.iter
              (fun (b : Phaser_spatial.bounds) ->
                x1 := min !x1 b.left;
                x2 := max !x2 b.right;
                y1 := min !y1 b.top;
                y2 := max !y2 b.bottom)
              bounds;
            if !x2 -. !x1 +. 24. > clip.right -. clip.left then (
              x1 := last.left;
              x2 := last.right);
            if !y2 -. !y1 +. 24. > clip.bottom -. clip.top then (
              y1 := last.top;
              y2 := last.bottom);
            (if request.center then (
               left <- (!x1 +. !x2 -. clip.left -. clip.right) /. 2.;
               self#move_vertical ((!y1 +. !y2 -. clip.top -. clip.bottom) /. 2.))
             else
               let visible_top = top in
               let dx = min 0. (!x1 -. left -. clip.left -. 12.) in
               left <- (left +. if dx <> 0. then dx else max 0. (!x2 -. left -. clip.right +. 12.));
               let dy = min 0. (!y1 -. visible_top -. clip.top -. 12.) in
               self#move_vertical ~mode:Pan
                 (visible_top
                 +. if dy <> 0. then dy else max 0. (!y2 -. visible_top -. clip.bottom +. 12.)));
            Performance_trace.instant {js|Battlefield camera reveal|js}
              ~args:
                (obj
                   [
                     ({js|ids|js}, Js.Json.array (Array.map string request.ids));
                     ({js|center|js}, bool request.center);
                     ({js|left|js}, number left);
                     ({js|top|js}, number top);
                   ]))
        (current ())

    method private update_commander =
      Option.iter
        (fun (frame : Battlefield_frame.t) ->
          if
            not (Option.fold ~none:false ~some:(fun game -> game == frame.view.game) commander_game)
          then (
            commander_game <- Some frame.view.game;
            let commander =
              Array.find_opt
                (fun (unit : Domain.public_unit) ->
                  unit.role = Some Domain.Role.Leader && Native.controllable frame.view.game unit.id)
                (Native.units frame.view.game)
            in
            commander_tile <-
              Option.bind commander (fun unit ->
                  Array.find_opt
                    (fun (tile : Battlefield_frame.tile) -> tile.terrain.id = Hex.key unit.tile)
                    frame.tiles));
          let hint =
            match commander_tile with
            | Some tile when viewport_width > 0. && viewport_height > 0. ->
                let _ = camera##getWorldPoint 0. 0. view_origin in
                let x = (tile.terrain.x -. view_origin##x) *. camera##zoomX /. pixel_x
                and y = (tile.terrain.y -. view_origin##y) *. camera##zoomY /. pixel_y in
                let sx = camera##zoomX /. pixel_x and sy = camera##zoomY /. pixel_y in
                if
                  x -. (Geometry.sprite_size /. 2. *. sx) < 0.
                  || x +. (Geometry.sprite_size /. 2. *. sx) > viewport_width
                  || y +. ((20. -. Geometry.sprite_size) *. sy) < 0.
                  || y +. (20. *. sy) > viewport_height
                then
                  Some
                    (int_of_float
                       (floor
                          (atan2 (y -. (viewport_height /. 2.)) (x -. (viewport_width /. 2.))
                           *. 180. /. Float.pi
                          +. 90. +. 0.5)))
                else None
            | _ -> None
          in
          if commander_hint <> Some hint then (
            commander_hint <- Some hint;
            let _ =
              Battlefield_view.Commander.dispatch host
                (Battlefield_view.Commander.create {js|battlefieldcommander|js}
                   [%mel.obj { detail = Js.Nullable.fromOption hint }])
            in
            ()))
        (current ())

    method private inspection_tiles (view : Battlefield_view.view) =
      let unit_tile id =
        Option.map
          (fun (unit : Domain.public_unit) -> Hex.key unit.tile)
          (Option.bind id (Native.unit view.game))
      in
      Array.append
        (Array_ext.filter_map Fun.id
           [| unit_tile view.selected; unit_tile view.target; view.inspected_tile |])
        (if view.follow_ai then view.ai_focus else [||])

    method private prepare_layout delta =
      frame_delta <- delta;
      (match current () with
      | Some frame
        when not (Option.fold ~none:false ~some:(fun old -> old == frame.view) inspection_view) ->
          self#prepare_picking frame;
          inspection_view <- Some frame.view;
          let ids = self#inspection_tiles frame.view in
          let next = Js.Array.join ~sep:{js|||js} (Array.of_list (Array.to_list ids)) in
          if next <> inspection then (
            inspection <- next;
            if Array.length ids > 0 then (
              self#stop_inertia {js|selection reveal|js};
              let unique = Set.create () in
              Array.iter
                (fun id ->
                  let _ = Set.add unique id in
                  ())
                ids;
              pending_reveal <- Some { ids = Set.values_array unique; center = false };
              layout_pending <- true))
      | _ -> ());
      if layout_pending || density <> D.device_ratio then (
        layout_pending <- false;
        self#perform_layout);
      match pending_reveal with
      | None -> ()
      | Some request ->
          pending_reveal <- None;
          self#apply_reveal request

    method private sample_camera =
      self#update_commander;
      Option.iter
        (fun (tracing : Performance_trace.config) ->
          let sample = (left, top, zoom, viewport_width, viewport_height) in
          if
            sample_trace <> tracing.id || last_sample <> Some sample || Option.is_some drag
            || Option.is_some pinch || Option.is_some inertia
          then (
            sample_trace <- tracing.id;
            last_sample <- Some sample;
            let requested_left =
              Option.fold ~none:left
                ~some:(fun (drag : drag) -> drag.left -. (drag.last_x -. drag.x))
                drag
            and requested_top =
              Option.fold ~none:top
                ~some:(fun (drag : drag) -> drag.top -. (drag.last_y -. drag.y))
                drag
            in
            let visible = self#visible_bounds in
            let details =
              [
                ( {js|visibleBounds|js},
                  obj
                    [
                      ({js|left|js}, number visible.left);
                      ({js|top|js}, number visible.top);
                      ({js|right|js}, number visible.right);
                      ({js|bottom|js}, number visible.bottom);
                    ] );
                ({js|deltaMs|js}, number frame_delta);
                ({js|dragging|js}, bool (Option.is_some drag));
                ({js|pinching|js}, bool (Option.is_some pinch));
                ({js|inertia|js}, bool (Option.is_some inertia));
                ({js|sampledAt|js}, number (now ()));
                ({js|requestedLeft|js}, number requested_left);
                ({js|requestedTop|js}, number requested_top);
                ({js|overshootX|js}, number (requested_left -. left));
                ({js|overshootY|js}, number (requested_top -. top));
                ({js|canvasWidth|js}, int canvas##width);
                ({js|canvasHeight|js}, int canvas##height);
                ({js|pixelX|js}, number pixel_x);
                ({js|pixelY|js}, number pixel_y);
              ]
            in
            let details =
              match drag with
              | None -> details
              | Some drag ->
                  [
                    ({js|x|js}, number drag.last_x);
                    ({js|y|js}, number drag.last_y);
                    ({js|vx|js}, number drag.vx);
                    ({js|vy|js}, number drag.vy);
                    ({js|eventTime|js}, number drag.time);
                  ]
                  @ details
            in
            self#record {js|camera sample|js} ~details))
        (Performance_trace.configuration ())

    method layout = layout_pending <- true

    method return_to_commander =
      Option.iter
        (fun (tile : Battlefield_frame.tile) ->
          self#record {js|return commander|js} ~details:[ ({js|tile|js}, string tile.terrain.id) ];
          self#cancel {js|return commander|js};
          pending_reveal <- Some { ids = [| tile.terrain.id |]; center = true };
          layout_pending <- true)
        commander_tile

    method reset ids =
      Option.iter self#prepare_picking (current ());
      self#cancel {js|scene reset|js};
      inspection <- {js||js};
      inspection_view <- None;
      zoom <- 1.;
      left <- 0.;
      top <- 0.;
      layout_ready <- false;
      pending_reveal <- Some { ids; center = true };
      layout_pending <- true

    method destroy = detach ()

    initializer
      D.tab_index canvas 0;
      canvas##setAttribute {js|role|js} {js|application|js};
      canvas##setAttribute {js|aria-label|js} {js|Battlefield|js};
      canvas##setAttribute {js|aria-describedby|js}
        {js|battlefield-controls battlefield-description|js};
      let bound_events =
        [|
          {js|mousedown|js};
          {js|mousemove|js};
          {js|mouseup|js};
          {js|touchstart|js};
          {js|touchmove|js};
          {js|touchend|js};
          {js|wheel|js};
        |]
      in
      let refresh_bounds () = self#refresh_bounds
      and mark_visible () = visible_dirty <- true
      and resize () = layout_pending <- true in
      let down = fun[@u] pointer -> self#down pointer
      and move = fun[@u] pointer -> self#move pointer false
      and up = fun[@u] pointer -> self#up pointer
      and outside = fun[@u] () -> self#cancel {js|pointer up outside|js}
      and wheel = fun[@u] pointer _ dx dy _ -> self#wheel pointer dx dy
      and key = fun[@u] event -> self#key event
      and cancel event = self#cancel (D.event_type event)
      and continue_drag = fun[@u] () -> Option.iter (fun drag -> self#move drag.pointer true) drag
      and prepare_layout = fun[@u] _ delta -> self#prepare_layout delta
      and sample_camera = fun[@u] () -> self#sample_camera in
      let expected_release = Set.create () in
      let capture event =
        if D.pointer_type event = {js|mouse|js} && List.mem (D.button event) [ 0; 1 ] then
          D.capture canvas (D.pointer_id event)
      in
      let release event =
        let id = D.pointer_id event in
        if D.has_capture canvas id then
          let _ = Set.add expected_release id in
          D.release canvas id
      in
      let lost_capture event =
        if not (Set.delete expected_release (D.pointer_id event)) then
          self#cancel {js|lost pointer capture|js}
      in
      let observer = ref None and dialog_observer = ref None and cleaned = ref false in
      let cleanup () =
        if not !cleaned then (
          cleaned := true;
          picking#clear;
          Option.iter D.disconnect_resize !observer;
          Option.iter D.disconnect_mutation !dialog_observer;
          Array.iter (fun name -> D.off_bound canvas name refresh_bounds true) bound_events;
          D.off_step scene##events {js|preupdate|js} prepare_layout;
          let _ = scene##events##off {js|prerender|js} sample_camera in
          let _ = scene##events##off {js|preupdate|js} continue_drag in
          D.off_window {js|resize|js} resize;
          D.off_window_event {js|blur|js} cancel;
          D.off_scroll {js|scroll|js} mark_visible true;
          self#stop_inertia {js|interrupted|js};
          D.off_canvas canvas {js|pointerdown|js} capture;
          D.off_canvas canvas {js|pointerup|js} release;
          D.off_canvas canvas {js|pointercancel|js} cancel;
          D.off_canvas canvas {js|lostpointercapture|js} lost_capture;
          D.off_canvas canvas {js|blur|js} cancel;
          D.off_pointer scene##input {js|pointerdown|js} down;
          D.off_pointer scene##input {js|pointermove|js} move;
          D.off_pointer scene##input {js|pointerup|js} up;
          D.off_input scene##input {js|pointerupoutside|js} outside;
          D.off_wheel scene##input {js|wheel|js} wheel;
          Option.iter
            (fun keyboard -> D.off_key keyboard {js|keydown|js} key)
            (Js.Nullable.toOption (D.keyboard scene##input)))
      in
      detach <- cleanup;
      let _ = scene##events##once {js|shutdown|js} (fun[@u] () -> cleanup ()) in
      let _ = scene##events##once {js|destroy|js} (fun[@u] () -> cleanup ()) in
      Array.iter
        (fun name ->
          D.on_bound canvas name refresh_bounds [%mel.obj { capture = true; passive = true }])
        bound_events;
      D.add_pointer scene##input 1;
      D.on_canvas canvas {js|pointerdown|js} capture;
      D.on_canvas canvas {js|pointerup|js} release;
      D.on_canvas canvas {js|pointercancel|js} cancel;
      D.on_canvas canvas {js|lostpointercapture|js} lost_capture;
      D.on_canvas canvas {js|blur|js} cancel;
      D.on_pointer scene##input {js|pointerdown|js} down;
      D.on_pointer scene##input {js|pointermove|js} move;
      D.on_pointer scene##input {js|pointerup|js} up;
      D.on_input scene##input {js|pointerupoutside|js} outside;
      D.on_wheel scene##input {js|wheel|js} wheel;
      Option.iter
        (fun keyboard -> D.on_key keyboard {js|keydown|js} key)
        (Js.Nullable.toOption (D.keyboard scene##input));
      let _ = scene##events##on {js|preupdate|js} continue_drag in
      D.on_step scene##events {js|preupdate|js} prepare_layout;
      let _ = scene##events##on {js|prerender|js} sample_camera in
      let resized = D.resize_observer (fun[@u] () -> resize ()) in
      observer := Some resized;
      D.observe_resize resized host;
      Option.iter (D.observe_resize resized) workspace;
      let dialogs =
        D.mutation_observer (fun[@u] () -> if D.modal () then self#cancel {js|modal opened|js})
      in
      dialog_observer := Some dialogs;
      D.observe_dialogs dialogs D.body
        [%mel.obj
          {
            attributes = true;
            attributeFilter = [| {js|data-ui-modal|js} |];
            childList = true;
            subtree = true;
          }];
      D.on_window {js|resize|js} resize;
      D.on_window_event {js|blur|js} cancel;
      D.on_scroll {js|scroll|js} mark_visible [%mel.obj { capture = true; passive = true }]
  end
