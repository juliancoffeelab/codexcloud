type asset = { map_id : string; plan : Terrain_plan.t; key : string; asset_key : string }

let prepare factory =
  Fun.protect
    ~finally:(fun () -> Domain.NativeEngineFactory.dispose factory)
    (fun () ->
      let catalog = Catalog.of_view (Domain.NativeEngineFactory.catalog factory) in
      let roles = Catalog.terrain_roles catalog in
      Array.map
        (fun (map : Domain.map_definition) ->
          let plan =
            Terrain_plan.prepare ~roles ~map_id:map.id ~cols:(Int32.to_float map.cols)
              ~rows:(Int32.to_float map.rows) map.terrain (Art.current ())
          in
          {
            map_id = map.id;
            plan;
            key = plan.key;
            asset_key = Terrain_plan.asset_key (map.id ^ ":" ^ Js.Float.toString map.revision);
          })
        (Catalog.raw catalog).maps)

let factory = function
  | Ok factory -> factory
  | Error error -> Js.Exn.raiseError (Utf8.to_js (Native.message (Native.Engine error)))

let plans () = prepare (factory (Domain.NativeEngineFactory.create ()))
