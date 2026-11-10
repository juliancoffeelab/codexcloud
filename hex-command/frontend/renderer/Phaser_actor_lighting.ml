module Map = Host_collections.Map
module Set = Host_collections.Set
module Weak_map = Host_collections.Weak_map

type data = {
  unlit : bool;
  light_anchor : Phaser.vector option;
  light_point : Phaser.vector option;
}

type sample = { x : float; y : float; mutable epoch : int; values : float array }

module Pipeline_data = Phaser_pipeline.Data (struct
  type t = data
end)

let identity = [| 1.; 1.; 1.; 0.; 0.; 0.; 0.; 0.; 0.; 0. |]
let daylight = { Domain.time = Domain.Time_of_day.Day; underground = [||]; lights = [||] }

let bucket_key x y =
  string_of_int (int_of_float (floor (x /. 128.)))
  ^ ":"
  ^ string_of_int (int_of_float (floor (y /. 128.)))

class actor_lighting (game : Phaser.game) =
  object (self)
    val mutable pipeline : Phaser_pipeline.t option = None
    val mutable lighting = daylight
    val mutable signature = ""
    val mutable epoch = 0
    val samples : (Phaser.vector, sample) Map.t = Map.create ()
    val owners : (Phaser.vector, Phaser.game_object Set.t) Map.t = Map.create ()
    val anchors : (Phaser.game_object, Phaser.vector) Weak_map.t = Weak_map.create ()
    val pending : Phaser.vector Set.t = Set.create ()
    val registered : (Phaser.game_object, bool) Weak_map.t = Weak_map.create ()
    val buckets : (string, int array) Map.t = Map.create ()

    initializer
      let attributes =
        Phaser_pipeline.attributes
          [|
            {
              Phaser.Pipeline.name = "inFactor";
              size = 4;
              kind = Phaser.Pipeline.float_type;
              normalized = false;
            };
            {
              Phaser.Pipeline.name = "inLuminance";
              size = 3;
              kind = Phaser.Pipeline.float_type;
              normalized = false;
            };
            {
              Phaser.Pipeline.name = "inOffset";
              size = 3;
              kind = Phaser.Pipeline.float_type;
              normalized = false;
            };
          |]
      in
      let created =
        Phaser_pipeline.create ~multi:true ~attributes ~game
          ~vertex:Phaser_shaders.actor_lighting_vertex
          ~fragment:Phaser_shaders.actor_lighting_fragment
          ~bind:(fun _ values object_ ->
            (* Phaser's shape renderer binds without an object. Shapes must not
               inherit the preceding sprite's lighting coefficients. *)
            values := Option.fold ~none:identity ~some:self#coefficients object_)
          ()
      in
      Phaser_pipeline.set_values created identity;
      pipeline <- Some created

    method native =
      match pipeline with
      | Some pipeline -> Phaser_pipeline.native pipeline
      | None -> failwith "Actor pipeline is not initialized"

    method configure (next : Domain.battlefield_lighting) =
      if Lighting.daylight_only next then (
        if not (Lighting.daylight_only lighting) then (
          epoch <- epoch + 1;
          Map.clear samples;
          Set.clear pending;
          Map.clear buckets);
        signature <- "day";
        lighting <- next)
      else
        let next_signature = Lighting.key next in
        if next_signature <> signature then (
          signature <- next_signature;
          let previous = lighting in
          lighting <- next;
          epoch <- epoch + 1;
          let bounds =
            Array.map
              (fun (light : Domain.battlefield_light) ->
                let x, y = Lighting.position light in
                {
                  Phaser_spatial.left = x -. light.radius;
                  right = x +. light.radius;
                  top = y -. (light.radius /. 1.12);
                  bottom = y +. (light.radius /. 1.12);
                })
              (Lighting.changed_lights previous.lights next.lights)
          in
          Map.each samples (fun[@u] sample anchor ->
              if
                previous.time <> next.time
                || previous.underground <> next.underground
                || Array.exists
                     (fun (bounds : Phaser_spatial.bounds) ->
                       sample.x >= bounds.left && sample.x <= bounds.right && sample.y >= bounds.top
                       && sample.y <= bounds.bottom)
                     bounds
              then
                let _ = Map.delete samples anchor in
                let _ = Set.add pending anchor in
                ()
              else sample.epoch <- epoch);
          Map.clear buckets;
          Array.iteri
            (fun index (light : Domain.battlefield_light) ->
              let x, y = Lighting.position light in
              for
                row = int_of_float (floor ((y -. (light.radius /. 1.12)) /. 128.))
                to int_of_float (floor ((y +. (light.radius /. 1.12)) /. 128.))
              do
                for
                  col = int_of_float (floor ((x -. light.radius) /. 128.))
                  to int_of_float (floor ((x +. light.radius) /. 128.))
                do
                  let key = string_of_int col ^ ":" ^ string_of_int row in
                  let bucket =
                    match Map.get buckets key with Some bucket -> bucket | None -> [||]
                  in
                  let _ = Array_ext.append_value bucket index in
                  let _ = Map.set buckets key bucket in
                  ()
                done
              done)
            next.lights)

    method prepare object_ =
      let _ = self#coefficients object_ in
      ()

    method prepare_pending =
      Set.each pending (fun[@u] anchor ->
          Option.iter
            (fun objects ->
              let values = Set.values_array objects in
              if Array.length values > 0 then self#prepare values.(0))
            (Map.get owners anchor));
      Set.clear pending

    method private forget anchor object_ =
      Option.iter
        (fun objects ->
          let _ = Set.delete objects object_ in
          if Set.size objects = 0 then
            let _ = Map.delete owners anchor in
            let _ = Map.delete samples anchor in
            let _ = Set.delete pending anchor in
            ())
        (Map.get owners anchor)

    method private coefficients (object_ : Phaser.game_object) =
      let data = Pipeline_data.get object_ in
      let point =
        match data.light_anchor with Some point -> Some point | None -> data.light_point
      in
      match point with
      | None -> identity
      | Some _ when data.unlit -> identity
      | Some point when Lighting.daylight_only lighting ->
          let values = Lighting.daylight point##x point##y in
          [|
            values.(0) *. 0.86;
            values.(1) *. 0.86;
            values.(2) *. 0.86;
            0.;
            values.(0) *. 0.14;
            values.(1) *. 0.14;
            values.(2) *. 0.14;
            values.(3);
            values.(4);
            values.(5);
          |]
      | Some point -> (
          (match Weak_map.get anchors object_ with
          | Some previous when previous == point -> ()
          | previous ->
              Option.iter (fun previous -> self#forget previous object_) previous;
              let objects =
                match Map.get owners point with Some objects -> objects | None -> Set.create ()
              in
              let _ = Set.add objects object_ in
              let _ = Map.set owners point objects in
              let _ = Weak_map.set anchors object_ point in
              ());
          (if not (Weak_map.has registered object_) then
             let _ = Weak_map.set registered object_ true in
             let _ =
               object_##once "destroy" (fun[@u] () ->
                   Option.iter
                     (fun anchor -> self#forget anchor object_)
                     (Weak_map.get anchors object_))
             in
             ());
          match Map.get samples point with
          | Some old when old.x = point##x && old.y = point##y && old.epoch = epoch -> old.values
          | _ ->
              let local =
                match Map.get buckets (bucket_key point##x point##y) with
                | Some local -> local
                | None -> [||]
              in
              let matrix =
                Lighting.matrix point##x point##y
                  { lighting with lights = Array.map (Array.get lighting.lights) local }
              in
              let lr = matrix.(1) /. 0.7152
              and lg = matrix.(5) /. 0.2126
              and lb = matrix.(10) /. 0.2126 in
              let values =
                [|
                  matrix.(0) -. (lr *. 0.2126);
                  matrix.(6) -. (lg *. 0.7152);
                  matrix.(12) -. (lb *. 0.0722);
                  0.;
                  lr;
                  lg;
                  lb;
                  matrix.(4);
                  matrix.(9);
                  matrix.(14);
                |]
              in
              let _ = Map.set samples point { x = point##x; y = point##y; epoch; values } in
              let _ = Set.delete pending point in
              values)
  end
