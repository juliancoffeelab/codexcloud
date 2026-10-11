open Ui;

module Markup = {
  [@react.component]
  let make = (~markup) => <g dangerouslySetInnerHTML={ "__html": markup } />;
};

/** Render detached native inspection. Keyboard navigation uses existing keyed
    tile nodes; it never queries or mutates an engine handle. */
[@react.component]
let make = (~view: Session.model, ~actions: Session.actions) => {
  let state = Native.state(view.game);
  let tiles =
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
  let size =
    Geometry.dimensions(~cols=int_of_float(state.cols), ~rows=int_of_float(state.rows));
  let map = React.useRef(Js.Nullable.null);
  let style = ReactDOM.Style.make();
  mapScale(style, number(state.cols /. 11.));
  let key = (event, tile: Domain.tile) => {
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
  };
  let center = (tile: Domain.tile) =>
    Geometry.center(~col=int_of_float(tile.col), ~row=int_of_float(tile.row));
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
          {each(
             tiles,
             display => {
               let tile = display.Presentation.tile;
               let p = center(tile);
               <clipPath key={tile.id} id={"hex-clip-" ++ tile.id}>
                 <polygon points={Geometry.hex_points(p.x, p.y)} />
               </clipPath>;
             },
           )}
        </defs>
        <g className="terrain-layer">
          {each(
             tiles,
             display => {
               let tile = display.Presentation.tile;
               let p = center(tile);
               let points = Geometry.hex_points(p.x, p.y);
               let props =
                 ReactDOM.domProps(
                   ~className=display.classes,
                   ~role="button",
                   ~tabIndex=tile.id == view.focused ? 0 : (-1),
                   ~ariaLabel=display.label,
                   ~onClick=_ => actions.choose(tile.id),
                   ~onKeyDown=event => key(event, tile),
                   (),
                 );
               dataTile(props, tile.id);
               dataKey(props, tile.id);
               ReactDOM.createElement(
                 "g",
                 ~props,
                 [|
                   <polygon
                     className="tile-ground"
                     points
                     fill={Art.color(view.artwork, tile.terrain)}
                   />,
                   <g clipPath={"url(#hex-clip-" ++ tile.id ++ ")"}>
                     <Markup
                       markup={Art.sprite(
                         view.artwork,
                         ~kind="terrain",
                         ~name=tile.terrain,
                         ~x=p.x -. 35.,
                         ~y=p.y -. 35.,
                         ~width=70.,
                         ~height=70.,
                         ~class_name="terrain-art",
                         (),
                       )}
                     />
                   </g>,
                   Js.String.includes(~search="objective", display.classes)
                   || Js.String.includes(~search="recruitable", display.classes)
                     ? <polygon className="tile-highlight-outline" points fill="none" />
                     : React.null,
                   <polygon className="tile-outline" points fill="none" />,
                 |],
               );
             },
           )}
        </g>
        <g className="unit-layer" pointerEvents="none" ariaHidden=true>
          {each(tiles, display =>
             option(
               display.Presentation.occupant,
               unit => {
                 let p = center(display.tile);
                 let definition = Catalog.unit(view.catalog, unit.type_);
                 let size = Option.value(~default=false, definition.leader) ? 62. : 56.;
                 let team = unit.side == Domain.side_player ? "#6fdec0" : "#ff9590";
                 let props = ReactDOM.domProps();
                 dataUnit(props, display.tile.id);
                 dataKey(props, unit.id);
                 ReactDOM.createElement(
                   "g",
                   ~props,
                   [|
                     <ellipse
                       className="team-base"
                       cx={number(p.x)}
                       cy={number(p.y +. 19.)}
                       rx="19"
                       ry="7"
                       fill=team
                       stroke=team
                     />,
                     <Markup
                       markup={Art.sprite(
                         view.artwork,
                         ~kind="units",
                         ~name=unit.type_,
                         ~side=side(unit.side),
                         ~x=p.x -. size /. 2.,
                         ~y=p.y +. 20. -. size,
                         ~width=size,
                         ~height=size,
                         ~class_name=
                           "unit-art "
                           ++ (unit.side == Domain.side_enemy ? "enemy-unit " : "")
                           ++ (unit.attacked ? "spent-unit" : ""),
                         (),
                       )}
                     />,
                   |],
                 );
               },
             )
           )}
        </g>
        <g className="indicator-layer" pointerEvents="none" ariaHidden=true>
          {each(
             tiles,
             display => {
               let p = center(display.Presentation.tile);
               let props = ReactDOM.domProps();
               dataIndicator(props, display.tile.id);
               dataKey(props, display.tile.id);
               ReactDOM.createElement(
                 "g",
                 ~props,
                 [|
                   option(display.occupant, unit =>
                     <>
                       <rect
                         className="hp-bg"
                         x={number(p.x -. 19.)}
                         y={number(p.y +. 27.)}
                         width="38"
                         height="4"
                         rx="2"
                       />
                       <rect
                         className="hp"
                         x={number(p.x -. 19.)}
                         y={number(p.y +. 27.)}
                         width={number(display.health_width)}
                         height="4"
                         rx="2"
                       />
                       {unit.attacked
                          ? <text
                              className="acted" x={number(p.x +. 21.)} y={number(p.y -. 17.)}>
                              {text({js|✓|js})}
                            </text>
                          : React.null}
                       {Option.value(~default=false, unit.retreating)
                          ? <text
                              className="retreat-marker"
                              x={number(p.x -. 28.)}
                              y={number(p.y -. 14.)}>
                              {text({js|↶|js})}
                            </text>
                          : React.null}
                     </>
                   ),
                   nullable(display.tile.owner, _ =>
                     <circle
                       className="territory-badge"
                       cx={number(p.x +. 23.)}
                       cy={number(p.y +. 17.)}
                       r="5"
                       fill={display.owner_color}
                     />
                   ),
                 |],
               );
             },
           )}
        </g>
      </svg>
    </div>
  </section>;
};
