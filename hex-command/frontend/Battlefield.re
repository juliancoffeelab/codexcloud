open Ui;
[@mel.module "virtual:terrain-prebuilt"] external sprites: Js.Dict.t(Terrain.sprite) = "default";

module Frame = {
  [@react.component]
  let make = (~frame: Art.frame, ~size, ~x, ~y, ~className, ~faction="") => {
    let props =
      ReactDOM.domProps(
        ~className="art-sprite pixel-art " ++ className,
        ~x=number(x),
        ~y=number(y),
        ~width=number(size),
        ~height=number(size),
        ~viewBox="0 0 " ++ number(frame.width) ++ " " ++ number(frame.height),
        ~ariaHidden=true,
        ~focusable="false",
        (),
      );
    if (faction != "") {
      dataFaction(props, faction);
    };
    ReactDOM.createElement(
      "svg",
      ~props,
      [|
        <image
          href={frame.url}
          x="0"
          y="0"
          width={number(frame.width)}
          height={number(frame.height)}
          preserveAspectRatio="none"
        />,
      |],
    );
  };
};

/** One static scene owns prebuilt previews and cancellable native canvas fallback.
    Unit and health nodes retain identity across movement and undo. */
[@react.component]
let make = (~view: Session.model, ~actions: Session.actions) => {
  let state = Native.state(view.game);
  let owner = React.useMemo0(() => Terrain.create(sprites));
  let prefetch = React.useMemo0(() => Terrain_prefetch.create());
  let scene =
    React.useMemo2(() => Terrain.prepare(owner, state, view.artwork), (state, view.artwork));
  let currentScene = React.useRef(scene);
  let (baked, setBaked) = React.useState(() => None);
  let (failed, setFailed) = React.useState(() => None);
  let (hovered, setHovered) = React.useState(() => None);
  let (ready, setReady) = React.useState(() => None);
  let map = React.useRef(Js.Nullable.null);
  let image = React.useRef(Js.Nullable.null);
  let canvasHost = React.useRef(Js.Nullable.null);
  let size = Terrain.size(scene);
  let backdrop =
    React.useMemo2(
      () =>
        Option.bind(baked, bitmap => Terrain.valid_bitmap(scene, bitmap) ? Some(bitmap) : None),
      (scene, baked),
    );
  let prebuilt =
    React.useMemo2(
      () => failed == Some(Terrain.key(scene)) ? None : Terrain.preview(owner, scene),
      (scene, failed),
    );
  let displays =
    React.useMemo4(
      () =>
        Presentation.battlefield(
          view.catalog,
          view.game,
          ~selected=view.selected,
          ~inspected=view.inspected,
        ),
      (view.catalog, view.game, view.selected, view.inspected),
    );
  let tiles =
    React.useMemo2(
      () =>
        Array.map(
          display =>
            (display, Option.get(Terrain.get_tile(scene, display.Presentation.tile.id))),
          displays,
        ),
      (displays, scene),
    );
  let occupied =
    React.useMemo1(
      () =>
        Array.fold_right(
          ((display, tile), result) =>
            switch (display.Presentation.occupant) {
            | Some(unit) => [(display, tile, unit), ...result]
            | None => result
            },
          tiles,
          [],
        )
        |> Array.of_list,
      [|tiles|],
    );
  let overlays =
    React.useMemo3(
      () =>
        filter(
          ((display, _)) =>
            backdrop == None
            && (
              switch (prebuilt) {
              | None => true
              | Some(preview) => Terrain.changed_contains(preview, display.Presentation.tile.id)
              }
            ),
          tiles,
        ),
      (tiles, backdrop, prebuilt),
    );
  let hasBackdrop = backdrop != None || prebuilt != None;
  let hover = Option.bind(hovered, id => Terrain.get_tile(scene, id));
  /* Rendering only prepares geometry. Commit owns cancellation, so an abandoned
     concurrent render cannot cancel the displayed map's fallback work. */
  React.useLayoutEffect1(
    () => {
      currentScene.current = scene;
      Terrain.activate(owner, scene);
      None;
    },
    [|scene|],
  );
  React.useEffect1(() => Some(() => Terrain.close(owner)), [|owner|]);
  React.useEffect1(() => Some(() => Terrain_prefetch.close(prefetch)), [|prefetch|]);
  React.useEffect2(
    () =>
      switch (ready) {
      | Some(displayed) when displayed === scene =>
        Some(Terrain_prefetch.start(prefetch, scene))
      | _ => None
      },
    (scene, ready),
  );
  React.useEffect2(
    () => {
      let exact =
        Option.fold(
          ~none=false,
          ~some=preview => Array.length(Terrain.changed(preview)) == 0,
          prebuilt,
        );
      if (exact
          || Array.exists((tile: Terrain_plan.tile) => tile.frame == None, Terrain.tiles(scene))) {
        None;
      } else {
        let active = ref(true);
        Terrain.bake(owner, scene)
        |> Js.Promise.then_(bitmap => {
             if (active.contents && currentScene.current === scene) {
               setBaked(_ => bitmap);
             };
             Js.Promise.resolve();
           })
        |> Js.Promise.catch(_ => Js.Promise.resolve())
        |> ignore;
        Some(() => active.contents = false);
      };
    },
    (scene, prebuilt),
  );
  React.useLayoutEffect2(
    () => {
      switch (Js.Nullable.toOption(canvasHost.current), backdrop) {
      | (Some(element), Some(bitmap)) =>
        Terrain.mount(element, bitmap);
        setReady(_ => Some(scene));
        Some(() => Terrain.clear(element));
      | _ => None
      }
    },
    (scene, backdrop),
  );
  /* Native SVG image events do not bubble. Prefetch starts only after the exact
     current PNG loads; stale loads and failures cannot warm a replaced scene. */
  React.useLayoutEffect3(
    () => {
      switch (Js.Nullable.toOption(image.current), prebuilt, backdrop) {
      | (Some(element), Some(preview), None) =>
        let active = ref(true);
        let loaded = _ =>
          if (active.contents
              && currentScene.current === scene
              && Array.length(Terrain.changed(preview)) == 0) {
            setReady(_ => Some(scene));
          };
        let error = _ =>
          if (active.contents && currentScene.current === scene) {
            setReady(_ => None);
            setFailed(_ => Some(Terrain.key(scene)));
          };
        addListener(element, "load", loaded);
        addListener(element, "error", error);
        Some(
          () => {
            active.contents = false;
            removeListener(element, "load", loaded);
            removeListener(element, "error", error);
          },
        );
      | _ => None
      }
    },
    (scene, prebuilt, backdrop),
  );
  let handleKey =
    React.useCallback2(
      (event, tile: Domain.tile) => {
        let pressed = React.Event.Keyboard.key(event);
        if (pressed == "Enter" || pressed == " ") {
          React.Event.Keyboard.preventDefault(event);
          actions.choose(tile.id);
        } else {
          let destination =
            switch (pressed) {
            | "ArrowLeft" => Some((tile.col -. 1., tile.row))
            | "ArrowRight" => Some((tile.col +. 1., tile.row))
            | "ArrowUp" => Some((tile.col, tile.row -. 1.))
            | "ArrowDown" => Some((tile.col, tile.row +. 1.))
            | _ => None
            };
          Option.iter(
            ((col, row)) => {
              React.Event.Keyboard.preventDefault(event);
              let id = number(col) ++ "," ++ number(row);
              if (Native.tile(view.game, id) != None) {
                actions.focus(id);
                Js.Nullable.toOption(map.current)
                |> Option.iter(element =>
                     queryWithin(element, "[data-tile='" ++ id ++ "']")
                     |> Js.Nullable.toOption
                     |> Option.iter(focus)
                   );
              };
            },
            destination,
          );
        };
      },
      (view.game, actions),
    );
  let clips =
    React.useMemo1(
      () =>
        each(overlays, ((_, tile: Terrain_plan.tile)) =>
          <clipPath key={tile.id} id={tile.clip_id}> <polygon points={tile.points} /> </clipPath>
        ),
      [|overlays|],
    );
  let terrainNodes =
    React.useMemo5(
      () =>
        each(
          tiles,
          ((display, tile: Terrain_plan.tile)) => {
            let props =
              ReactDOM.domProps(
                ~className=display.Presentation.classes,
                ~role="button",
                ~tabIndex=tile.id == view.focused ? 0 : (-1),
                ~ariaLabel=display.label,
                ~onClick=_ => actions.choose(tile.id),
                ~onKeyDown=event => handleKey(event, display.tile),
                ~onPointerEnter=
                  event =>
                    if (React.Event.Pointer.pointerType(event) == "mouse") {
                      setHovered(_ => Some(tile.id));
                    },
                ~onPointerLeave=
                  _ => setHovered(previous => previous == Some(tile.id) ? None : previous),
                (),
              );
            dataTile(props, tile.id);
            dataKey(props, tile.id);
            let covered =
              backdrop != None
              || Option.fold(
                   ~none=false,
                   ~some=preview => !Terrain.changed_contains(preview, tile.id),
                   prebuilt,
                 );
            ReactDOM.createElement(
              "g",
              ~props,
              [|
                covered
                  ? <polygon points={tile.points} fill="transparent" pointerEvents="all" />
                  : <>
                      <polygon className="tile-ground" points={tile.points} fill={tile.fill} />
                      <g clipPath={"url(#" ++ tile.clip_id ++ ")"}>
                        {option(tile.frame, frame =>
                           <Frame
                             frame
                             size={tile.sprite_size}
                             x={tile.x -. tile.sprite_size /. 2.}
                             y={tile.y -. tile.sprite_size /. 2.}
                             className="terrain-art"
                           />
                         )}
                      </g>
                    </>,
                Js.String.includes(~search="objective", display.classes)
                || Js.String.includes(~search="recruitable", display.classes)
                  ? <polygon
                      className="tile-highlight-outline"
                      points={tile.points}
                      fill="none"
                    />
                  : React.null,
                <polygon className="tile-outline" points={tile.points} fill="none" />,
              |],
            );
          },
        ),
      (tiles, backdrop, prebuilt, view.focused, handleKey),
    );
  let unitNodes =
    React.useMemo2(
      () =>
        each(
          occupied,
          ((_, tile: Terrain_plan.tile, unit: Domain.public_unit)) => {
            let team = unit.side == Domain.side_player ? "#6fdec0" : "#ff9590";
            let props =
              ReactDOM.domProps(
                ~transform="translate(" ++ number(tile.x) ++ " " ++ number(tile.y) ++ ")",
                (),
              );
            dataKey(props, unit.id);
            dataUnit(props, tile.id);
            dataUnitId(props, unit.id);
            ReactDOM.createElement(
              "g",
              ~props,
              [|
                <ellipse
                  className="team-base"
                  cx="0"
                  cy="19"
                  rx="19"
                  ry="7"
                  fill=team
                  stroke=team
                />,
                option(Art.unit_frame(view.artwork, ~side=side(unit.side), unit.type_), frame =>
                  <Frame
                    frame
                    size=Geometry.sprite_size
                    x={-. Geometry.sprite_size /. 2.}
                    y={20. -. Geometry.sprite_size}
                    faction={side(unit.side)}
                    className={
                      "unit-art "
                      ++ (unit.side == Domain.side_enemy ? "enemy-unit " : "")
                      ++ (unit.attacked ? "spent-unit" : "")
                    }
                  />
                ),
              |],
            );
          },
        ),
      (occupied, view.artwork),
    );
  let healthNodes =
    React.useMemo1(
      () =>
        each(
          occupied,
          ((display, tile: Terrain_plan.tile, unit: Domain.public_unit)) => {
            let props =
              ReactDOM.domProps(
                ~transform="translate(" ++ number(tile.x) ++ " " ++ number(tile.y) ++ ")",
                (),
              );
            dataKey(props, unit.id);
            dataHealthUnit(props, unit.id);
            ReactDOM.createElement(
              "g",
              ~props,
              [|
                <rect className="hp-bg" x="-19" y="27" width="38" height="4" rx="2" />,
                <rect
                  className="hp"
                  x="-19"
                  y="27"
                  width={number(display.Presentation.health_width)}
                  height="4"
                  rx="2"
                />,
                unit.attacked
                  ? <text className="acted" x="21" y="-17"> {text({js|✓|js})} </text>
                  : React.null,
                Option.value(~default=false, unit.retreating)
                  ? <text className="retreat-marker" x="-28" y="-14"> {text({js|↶|js})} </text>
                  : React.null,
              |],
            );
          },
        ),
      [|occupied|],
    );
  let ownershipNodes =
    React.useMemo1(
      () =>
        each(
          tiles,
          ((display, tile: Terrain_plan.tile)) => {
            let props = ReactDOM.domProps();
            dataKey(props, tile.id);
            dataIndicator(props, tile.id);
            ReactDOM.createElement(
              "g",
              ~props,
              [|
                nullable(display.Presentation.tile.owner, _ =>
                  <circle
                    className="territory-badge"
                    cx={number(tile.x +. 23.)}
                    cy={number(tile.y +. 17.)}
                    r="5"
                    fill={display.owner_color}
                  />
                ),
              |],
            );
          },
        ),
      [|tiles|],
    );
  let style = ReactDOM.Style.make();
  mapScale(style, number(state.cols /. 11.));
  <section className="map-section" ariaLabel="Battlefield">
    <div className="map-scroll">
      <svg
        id="map"
        style
        ref={ReactDOM.Ref.domRef(map)}
        viewBox={"0 0 " ++ number(size.width) ++ " " ++ number(size.height)}
        role="group"
        ariaLabel={Catalog.map(view.catalog, state.mapId).name ++ " battlefield"}>
        <defs>
          clips
          {hasBackdrop
             ? option(hover, tile =>
                 <clipPath id="terrain-hover-clip">
                   <polygon points={tile.Terrain_plan.points} />
                 </clipPath>
               )
             : React.null}
        </defs>
        <g className="terrain-layer">
          {switch (backdrop) {
           | Some(bitmap) =>
             let dimensions = Terrain.bitmap_size(bitmap);
             <g className="pixel-art" pointerEvents="none" ariaHidden=true>
               <foreignObject
                 x="0"
                 y="0"
                 width={number(dimensions.display_width)}
                 height={number(dimensions.display_height)}>
                 <div className="terrain-bitmap" ref={ReactDOM.Ref.domRef(canvasHost)} />
               </foreignObject>
             </g>;
           | None =>
             option(
               prebuilt,
               preview => {
                 let sprite = Terrain.sprite(preview);
                 <g className="pixel-art" pointerEvents="none" ariaHidden=true>
                   <image
                     key={Terrain.key(scene)}
                     className="map-terrain"
                     ref={ReactDOM.Ref.domRef(image)}
                     href={sprite.url}
                     x="0"
                     y="0"
                     width={number(sprite.display_width)}
                     height={number(sprite.display_height)}
                     preserveAspectRatio="none"
                     pointerEvents="none"
                     ariaHidden=true
                   />
                 </g>;
               },
             )
           }}
          {hasBackdrop
             ? option(hover, tile =>
                 <g
                   className="terrain-hover"
                   clipPath="url(#terrain-hover-clip)"
                   pointerEvents="none"
                   ariaHidden=true>
                   <polygon
                     className="tile-ground"
                     points={tile.Terrain_plan.points}
                     fill={tile.fill}
                   />
                   {option(tile.frame, frame =>
                      <Frame
                        frame
                        size={tile.sprite_size}
                        x={tile.x -. tile.sprite_size /. 2.}
                        y={tile.y -. tile.sprite_size /. 2.}
                        className="terrain-art"
                      />
                    )}
                 </g>
               )
             : React.null}
          terrainNodes
        </g>
        <g className="unit-layer" pointerEvents="none" ariaHidden=true> unitNodes </g>
        <g className="indicator-layer" pointerEvents="none" ariaHidden=true>
          healthNodes
          ownershipNodes
        </g>
      </svg>
    </div>
  </section>;
};
