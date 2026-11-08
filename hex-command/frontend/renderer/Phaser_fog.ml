module Map = Host_collections.Map
module Set = Host_collections.Set

let interior = 510
let guard = 14

type page = {
  x : int;
  y : int;
  width : int;
  height : int;
  mutable target : Phaser.render_texture option;
  mutable display : Phaser.image option;
}

let key page = "fog:" ^ string_of_int page.x ^ ":" ^ string_of_int page.y

let target scene width height =
  let target = Phaser.GameObjects.render_texture scene 0. 0. width height false in
  let _ = target##setOrigin 0. 0. in
  target

let points text =
  Array.of_list
    (List.map
       (fun pair ->
         match String.split_on_char ',' pair with
         | [ x; y ] -> [%mel.obj { x = float_of_string x; y = float_of_string y }]
         | _ -> invalid_arg "Invalid authored hex polygon")
       (String.split_on_char ' ' text))

class fog (scene : Phaser.scene) (manifest : Terrain_pages.sprite) (tiles : Terrain_plan.tile array)
  (resources : Phaser_resources.resources) =
  let width = (manifest.width + 1) / 2 and height = (manifest.height + 1) / 2 in
  let pixel_x = float_of_int manifest.width /. Geometry.native_pixel_scale /. float_of_int width
  and pixel_y =
    float_of_int manifest.height /. Geometry.native_pixel_scale /. float_of_int height
  in
  let scratch_width = min interior width + (guard * 2)
  and scratch_height = min interior height + (guard * 2) in
  object (self)
    val root = scene##add##container 0. 0.
    val mutable visibility_signature : (Phaser_spatial.bounds * int) option = None
    val mutable version = 0
    val pages : page array = [||]
    val mutable active : page Set.t = Set.create ()
    val pages_index : page Phaser_spatial.t = Phaser_spatial.create ~size:512. ()
    val tiles_index : Terrain_plan.tile Phaser_spatial.t = Phaser_spatial.create ~size:128. ()
    val mutable fog : string Set.t = Set.create ()
    val dirty : page Set.t = Set.create ()
    val mutable mask : Phaser.render_texture option = None
    val mutable scratch : Phaser.render_texture option = None
    val mutable initialized = false

    initializer
      let _ = root##setDepth 9500. in
      let _ =
        resources#check ~target:true "padded half-native fog" (float_of_int scratch_width)
          (float_of_int scratch_height)
      in
      let y = ref 0 in
      while !y < height do
        let x = ref 0 in
        while !x < width do
          let page =
            {
              x = !x;
              y = !y;
              width = min interior (width - !x);
              height = min interior (height - !y);
              target = None;
              display = None;
            }
          in
          let _ = Array_ext.append_value pages page in
          Phaser_spatial.set pages_index page
            {
              left = float_of_int (!x - guard);
              top = float_of_int (!y - guard);
              right = float_of_int (!x + page.width + guard);
              bottom = float_of_int (!y + page.height + guard);
            };
          x := !x + interior
        done;
        y := !y + interior
      done;
      Array.iter
        (fun (tile : Terrain_plan.tile) ->
          Phaser_spatial.set tiles_index tile
            {
              left = tile.x -. 36.;
              top = tile.y -. 35.;
              right = tile.x +. 36.;
              bottom = tile.y +. 35.;
            })
        tiles

    method root = root

    method update ids (tiles : Terrain_plan.tile array) =
      let next_fog = Set.create () and changed = Set.create () in
      Array.iter
        (fun id ->
          let _ = Set.add next_fog id in
          ())
        ids;
      (if not initialized then (
         Array.iter
           (fun page ->
             let _ = Set.add changed page in
             ())
           pages;
         initialized <- true)
       else
         let by_id = Map.create () and union = Set.create () in
         Array.iter
           (fun (tile : Terrain_plan.tile) ->
             let _ = Map.set by_id tile.id tile in
             ())
           tiles;
         Set.each next_fog (fun[@u] id ->
             let _ = Set.add union id in
             ());
         Set.each fog (fun[@u] id ->
             let _ = Set.add union id in
             ());
         Set.each union (fun[@u] id ->
             if Set.has next_fog id <> Set.has fog id then
               Option.iter
                 (fun (tile : Terrain_plan.tile) ->
                   Array.iter
                     (fun page ->
                       let _ = Set.add changed page in
                       ())
                     (Phaser_spatial.query pages_index
                        {
                          left = (tile.x -. 36.) /. pixel_x;
                          top = (tile.y -. 35.) /. pixel_y;
                          right = (tile.x +. 36.) /. pixel_x;
                          bottom = (tile.y +. 35.) /. pixel_y;
                        }))
                 (Map.get by_id id)));
      Set.each changed (fun[@u] page ->
          let _ = Set.add dirty page in
          ());
      fog <- next_fog;
      let _ = root##setVisible (Set.size fog > 0) in
      if Set.size dirty > 0 && Set.size fog > 0 then (
        if Option.is_none mask then (
          let next_mask = target scene scratch_width scratch_height
          and next_scratch = target scene scratch_width scratch_height in
          let _ = next_mask##texture##setFilter 0 in
          let _ = next_scratch##texture##setFilter 0 in
          let _ = next_scratch##camera##setOrigin 0. 0. in
          let _ =
            next_scratch##camera##setScroll
              (-.float_of_int (width mod 2) /. 2.)
              (-.float_of_int (height mod 2) /. 2.)
          in
          mask <- Some next_mask;
          scratch <- Some next_scratch;
          resources#set "fog-scratch" "fog scratch" 0.
            (float_of_int scratch_width *. float_of_int scratch_height *. 8.));
        Set.each dirty (fun[@u] page ->
            Performance_trace.measure "Phaser fog page rebuild" (fun () -> self#rebuild page));
        Set.clear dirty)

    method private rebuild page =
      version <- version + 1;
      let x = page.x - guard and y = page.y - guard in
      let world =
        {
          Phaser_spatial.left = float_of_int x *. pixel_x;
          top = float_of_int y *. pixel_y;
          right = float_of_int (x + scratch_width) *. pixel_x;
          bottom = float_of_int (y + scratch_height) *. pixel_y;
        }
      in
      let mask = Option.get mask and scratch = Option.get scratch in
      let brush = Phaser.GameObjects.graphics scene in
      let _ = brush##fillStyle 0xffffff 1. in
      let _ =
        Phaser_atmosphere.Mask.graphics brush "BattlefieldFogMask"
          {
            width = float_of_int scratch_width *. pixel_x;
            height = float_of_int scratch_height *. pixel_y;
            origin_x = world.left;
            origin_y = world.top;
            extent = [| manifest.display_width; manifest.display_height |];
          }
          false
      in
      Fun.protect
        ~finally:(fun () -> brush##destroy ())
        (fun () ->
          Array.iter
            (fun (tile : Terrain_plan.tile) ->
              if Set.has fog tile.id then
                let _ = brush##fillPoints (points tile.points) true false in
                ())
            (Phaser_spatial.query tiles_index world);
          let _ = mask##camera##setOrigin 0. 0. in
          let _ = mask##camera##setZoom 1. in
          let _ = mask##clear () in
          let _ = mask##draw (Phaser.Upcast.graphics brush) 0. 0. in
          ());
      let region =
        [|
          float_of_int x; float_of_int y; float_of_int scratch_width; float_of_int scratch_height;
        |]
      and extent = [| float_of_int width; float_of_int height |]
      and capture =
        Some [| float_of_int (width mod 2) /. 2.; float_of_int (height mod 2) /. 2. |]
      in
      let _ =
        Phaser_atmosphere.Blur.target mask "BattlefieldFogBlur"
          { x = 1. /. float_of_int scratch_width; y = 0.; region; extent; capture }
          false
      in
      let _ = scratch##clear () in
      let _ = scratch##draw (Phaser.Upcast.render_texture mask) 0. 0. in
      let _ =
        Phaser_atmosphere.Blur.target scratch "BattlefieldFogBlur"
          { x = 0.; y = 1. /. float_of_int scratch_height; region; extent; capture }
          false
      in
      let final =
        match page.target with
        | Some target -> target
        | None ->
            let target = target scene scratch_width scratch_height in
            let _ = target##texture##setFilter 0 in
            let _ = target##camera##setOrigin 0. 0. in
            let _ =
              target##camera##setScroll
                (-.float_of_int (width mod 2) /. 2.)
                (-.float_of_int (height mod 2) /. 2.)
            in
            let display = Phaser.TextureImage.create scene world.left world.top target##texture in
            let _ = display##setOrigin 0. 0. in
            let _ =
              display##setDisplaySize
                (float_of_int scratch_width *. pixel_x)
                (float_of_int scratch_height *. pixel_y)
            in
            let _ =
              display##setCrop (float_of_int guard) (float_of_int guard) (float_of_int page.width)
                (float_of_int page.height)
            in
            let _ = Phaser_pipeline.composite display "BattlefieldFogComposite" in
            page.target <- Some target;
            page.display <- Some display;
            resources#set (key page) "fog finals" 0.
              (float_of_int scratch_width *. float_of_int scratch_height *. 4.);
            target
      in
      let _ = final##clear () in
      let _ = final##draw (Phaser.Upcast.render_texture scratch) 0. 0. in
      ()

    method visible (bounds : Phaser_spatial.bounds) =
      let same =
        match visibility_signature with
        | Some (old, old_version) -> old_version = version && Phaser_spatial.same_bounds old bounds
        | None -> false
      in
      if not same then (
        visibility_signature <- Some (bounds, version);
        let records =
          Phaser_spatial.query pages_index
            {
              left = bounds.left /. pixel_x;
              top = bounds.top /. pixel_y;
              right = bounds.right /. pixel_x;
              bottom = bounds.bottom /. pixel_y;
            }
        in
        let visible = Set.create () in
        Array.iter
          (fun page ->
            let _ = Set.add visible page in
            ())
          records;
        Set.each active (fun[@u] page ->
            if not (Set.has visible page) then
              Option.iter
                (fun (display : Phaser.image) ->
                  if not (Js.Nullable.isNullable display##parentContainer) then
                    Phaser_visibility.detach_root root (Phaser.Upcast.image display))
                page.display);
        Array.iter
          (fun page ->
            Option.iter
              (fun (display : Phaser.image) ->
                if Js.Nullable.isNullable display##parentContainer then
                  let _ = root##add (Phaser.Upcast.image display) in
                  ())
              page.display)
          records;
        active <- visible)

    method destroy =
      Option.iter (fun (target : Phaser.render_texture) -> target##destroy ()) mask;
      Option.iter (fun (target : Phaser.render_texture) -> target##destroy ()) scratch;
      resources#remove "fog-scratch";
      Array.iter
        (fun page ->
          Option.iter (fun (image : Phaser.image) -> image##destroy ()) page.display;
          Option.iter (fun (target : Phaser.render_texture) -> target##destroy ()) page.target;
          resources#remove (key page))
        pages;
      root##destroy ()
  end
