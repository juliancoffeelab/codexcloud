module Map = Host_collections.Map

type cell = { id : string; x : float; y : float; order : int }

class picking (camera : Phaser.camera) =
  let layer =
    Phaser.Tilemaps.layer
      [%mel.obj { baseTileWidth = Geometry.column_spacing; baseTileHeight = Geometry.sprite_size }]
  in
  let origin = Geometry.center ~col:0 ~row:0 in
  let offsets =
    Array.init 6 (fun i ->
        let angle = float_of_int ((60 * i) - 30) *. Float.pi /. 180. in
        [%mel.obj
          {
            x = Geometry.column_spacing /. sqrt 3. *. cos angle;
            y = Geometry.sprite_size /. 2. *. sin angle;
          }])
  in
  let polygon =
    Phaser.Geom.Polygon.create (Array.map (fun p -> [%mel.obj { x = p##x; y = p##y }]) offsets)
  in
  object (self)
    val mutable identity = {js||js}
    val rows : (int, (int, cell) Map.t) Map.t = Map.create ()
    val coordinate = Phaser.Geom.Vector2.create ()

    initializer
      Phaser.Tilemaps.stagger_axis layer {js|y|js};
      Phaser.Tilemaps.stagger_index layer {js|odd|js}

    method prepare key (tiles : Terrain_plan.tile array) =
      if key <> identity then (
        self#clear;
        Array.iteri
          (fun order (tile : Terrain_plan.tile) ->
            let row =
              match Map.get rows (int_of_float tile.row) with
              | Some row -> row
              | None ->
                  let row = Map.create () in
                  let _ = Map.set rows (int_of_float tile.row) row in
                  row
            in
            match Map.get row (int_of_float tile.col) with
            | Some old when old.order <= order -> ()
            | _ ->
                let _ =
                  Map.set row (int_of_float tile.col)
                    { id = tile.id; x = tile.x; y = tile.y; order }
                in
                ())
          tiles;
        identity <- key)

    method pick x y =
      let _ =
        Phaser.Tilemaps.world_to_tile
          (x -. origin.x +. (Geometry.column_spacing /. 2.))
          (y -. origin.y +. (Geometry.sprite_size /. 2.))
          true coordinate camera layer
      in
      let best = ref None in
      for dy = -1 to 1 do
        for dx = -1 to 1 do
          Option.iter
            (fun row ->
              Option.iter
                (fun (cell : cell) ->
                  if
                    not
                      (Option.fold ~none:false
                         ~some:(fun (old : cell) -> old.order < cell.order)
                         !best)
                  then (
                    Array.iteri
                      (fun i point ->
                        Phaser.Geom.Vector2.set_x (polygon##points).(i) (cell.x +. point##x);
                        Phaser.Geom.Vector2.set_y (polygon##points).(i) (cell.y +. point##y))
                      offsets;
                    if Phaser.Geom.Polygon.contains polygon x y then best := Some cell))
                (Map.get row (int_of_float coordinate##x + dx)))
            (Map.get rows (int_of_float coordinate##y + dy))
        done
      done;
      Option.map (fun (cell : cell) -> cell.id) !best

    method clear =
      Map.clear rows;
      identity <- {js||js}
  end
