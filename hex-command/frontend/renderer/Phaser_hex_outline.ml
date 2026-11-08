(* Port of #88's shared idle grid and vector action outlines. *)
let points =
  Geometry.hex_points 0. 0. |> String.split_on_char ' '
  |> List.map (fun point ->
      match String.split_on_char ',' point with
      | [ x; y ] -> Phaser.Geom.Point.create (float_of_string x) (float_of_string y)
      | _ -> assert false)
  |> Array.of_list

type edge = { a : Phaser.vector; b : Phaser.vector; length : float }

let edges =
  Array.mapi
    (fun i (a : Phaser.vector) ->
      let b = points.((i + 1) mod Array.length points) in
      let dx = b##x -. a##x and dy = b##y -. a##y in
      { a; b; length = sqrt ((dx *. dx) +. (dy *. dy)) })
    points

let perimeter = Array.fold_left (fun sum edge -> sum +. edge.length) 0. edges
let idle_line_width = 0.7
let idle_texture_key = "battlefield-idle-hex"
let native_scale = 64. /. Geometry.sprite_size
let idle_texture_size = 72

(* One shared native-resolution outline, created and uploaded by Phaser. *)
let idle_texture (scene : Phaser.scene) =
  (if not (scene##textures##exists idle_texture_key) then
     let graphics = scene##make##graphics [%mel.obj { x = 0.; y = 0. }] false in
     Fun.protect
       ~finally:(fun () -> graphics##destroy ())
       (fun () ->
         let _ = graphics##lineStyle (idle_line_width *. native_scale) 0xffffff 1. in
         let _ =
           Phaser.Graphics.stroke_points graphics
             (Array.map
                (fun (point : Phaser.vector) ->
                  [%mel.obj
                    {
                      x = (point##x *. native_scale) +. (float_of_int idle_texture_size /. 2.);
                      y = (point##y *. native_scale) +. (float_of_int idle_texture_size /. 2.);
                    }])
                points)
             true
         in
         let _ = graphics##generateTexture idle_texture_key idle_texture_size idle_texture_size in
         let _ = (scene##textures##get idle_texture_key)##setFilter Phaser.Textures.nearest in
         ()));
  idle_texture_key

class outline (scene : Phaser.scene) =
  let root = scene##add##container 0. 0. in
  let idle = scene##add##image 0. 0. (idle_texture scene) in
  let _ =
    idle##setDisplaySize
      (float_of_int idle_texture_size /. native_scale)
      (float_of_int idle_texture_size /. native_scale)
  in
  let graphics = scene##add##graphics () in
  let _ =
    Phaser.SceneMethods.add_many root
      [| Phaser.Upcast.image idle; Phaser.Upcast.graphics graphics |]
  in
  object (self)
    val mutable underlay : Phaser.graphics option = None
    val mutable style = ""
    method root = root

    method update x y color width alpha highlighted dash =
      let _ = root##setPosition x y in
      let is_idle = not highlighted in
      let _ = idle##setVisible is_idle in
      let _ = idle##setTint color in
      let _ = idle##setAlpha alpha in
      let _ = graphics##setVisible (not is_idle) in
      let _ = graphics##setAlpha alpha in
      Option.iter
        (fun (g : Phaser.graphics) ->
          let _ = g##setVisible ((not is_idle) && Option.is_some dash) in
          ())
        underlay;
      if not is_idle then
        let signature =
          string_of_int color ^ ":" ^ Js.Float.toString width ^ ":"
          ^ Option.fold ~none:"undefined"
              ~some:(fun (a, b) -> Js.Float.toString a ^ ":" ^ Js.Float.toString b)
              dash
        in
        if signature <> style then (
          style <- signature;
          let _ = graphics##clear () in
          Option.iter
            (fun dash ->
              let background =
                match underlay with
                | Some background -> background
                | None ->
                    let background = scene##add##graphics () in
                    underlay <- Some background;
                    let _ = root##addAt (Phaser.Upcast.graphics background) 0 in
                    background
              in
              let _ = background##clear () in
              self#line background 7. 0x172015 (Some dash))
            dash;
          Option.iter
            (fun (g : Phaser.graphics) ->
              let _ = g##setVisible (Option.is_some dash) in
              ())
            underlay;
          self#line graphics width color dash)

    method private line (graphics : Phaser.graphics) width color dash =
      let _ = graphics##lineStyle width color 1. in
      match dash with
      | None ->
          let _ = Phaser.Graphics.stroke_points graphics points true in
          ()
      | Some (paint, gap) ->
          let start = ref 0. in
          while !start < perimeter do
            let finish = min (!start +. paint) perimeter in
            let offset = ref 0. and begun = ref false in
            let _ = graphics##beginPath () in
            Array.iter
              (fun edge ->
                let from = max !start !offset and to_ = min finish (!offset +. edge.length) in
                if to_ > from then (
                  let x at =
                    edge.a##x +. ((edge.b##x -. edge.a##x) *. (at -. !offset) /. edge.length)
                  in
                  let y at =
                    edge.a##y +. ((edge.b##y -. edge.a##y) *. (at -. !offset) /. edge.length)
                  in
                  (if not !begun then
                     let _ = graphics##moveTo (x from) (y from) in
                     ());
                  let _ = graphics##lineTo (x to_) (y to_) in
                  begun := true);
                offset := !offset +. edge.length)
              edges;
            let _ = graphics##strokePath () in
            start := !start +. (paint +. gap)
          done
  end
