let text = value => React.string(value);
let number = Js.Float.toString;
let num = value => text(number(value));
let filter = (predicate, items) =>
  items |> Array.to_list |> List.filter(predicate) |> Array.of_list;
let each = (items, render) => React.array(Array.map(render, items));
let eachList = (items, render) => React.array(Array.of_list(List.map(render, items)));
let filterList = (predicate, items) => List.filter(predicate, items);
let option = (value, render) =>
  switch (value) {
  | None => React.null
  | Some(value) => render(value)
  };
let percent = value =>
  ReactDOM.Style.make(
    ~width={
      let js_text_part_0 = number(value);
      {j|$(js_text_part_0)%|j};
    },
    (),
  );
let side = (value: Domain.Turn_phase.t) =>
  switch (value) {
  | Player => {js|player|js}
  | Ai => {js|enemy|js}
  };
let ignorePromise = work => {
  let _ = work |> Js.Promise.catch(_ => Js.Promise.resolve());
  ();
};
include Browser_dom;
[@mel.get] external checkedTarget: React.Event.Form.t => {. "checked": bool } = "currentTarget";
[@mel.get] external checked: {. "checked": bool } => bool = "checked";
let bindChecked = event => checked(checkedTarget(event));
[@mel.set] external dataKey: (ReactDOM.domProps, string) => unit = "key";
[@mel.set] external dataTile: (ReactDOM.domProps, string) => unit = "data-tile";
[@mel.set] external dataUnit: (ReactDOM.domProps, string) => unit = "data-unit-tile";
[@mel.set] external dataUnitId: (ReactDOM.domProps, string) => unit = "data-unit";
[@mel.set] external dataHealthUnit: (ReactDOM.domProps, string) => unit = "data-health-unit";
[@mel.set] external dataFaction: (ReactDOM.domProps, string) => unit = "data-faction";
[@mel.set] external dataIndicator: (ReactDOM.domProps, string) => unit = "data-indicator-tile";
[@mel.set] external dataAttack: (ReactDOM.domProps, string) => unit = "data-attack";
[@mel.set] external dataInspect: (ReactDOM.domProps, string) => unit = "data-inspect";
[@mel.set] external dataRecruit: (ReactDOM.domProps, string) => unit = "data-recruit";
[@mel.set] external dataForecast: (ReactDOM.domProps, string) => unit = "data-forecast";
[@mel.set] external mapScale: (ReactDOM.Style.t, string) => unit = "--map-scale";
module Icon = {
  [@react.component]
  let make = (~name) =>
    if (name == {js|blade|js}
        || name == {js|pierce|js}
        || name == {js|impact|js}
        || name == {js|move|js}
        || name == {js|search|js}) {
      <svg
        className={js|ui-icon|js}
        viewBox={js|0 0 24 24|js}
        ariaHidden=true
        focusable={js|false|js}
        dangerouslySetInnerHTML={ "__html": Art.icon_path(name) }
      />;
    } else {
      <svg viewBox={js|0 0 24 24|js} ariaHidden=true>
        <path
          d={
            switch (name) {
            | matched_text_0 when matched_text_0 == {js|left|js} => {js|m15 5-7 7 7 7|js}
            | matched_text_1 when matched_text_1 == {js|right|js} => {js|m9 5 7 7-7 7|js}
            | _ => {js||js}
            }
          }
        />
      </svg>;
    };
};
module Sprite = {
  [@react.component]
  let make =
      (
        ~artwork,
        ~kind={js|units|js},
        ~name,
        ~side={js|player|js},
        ~floor=None,
        ~width,
        ~height,
        ~className={js||js},
      ) => {
    let frame =
      switch (kind) {
      | matched_text_2 when matched_text_2 == {js|terrain|js} =>
        switch (floor) {
        | None => Art.terrain_frame(artwork, name)
        | Some(floor) => Art.terrain_frame(artwork, ~floor, name)
        }
      | matched_text_3 when matched_text_3 == {js|portraits|js} =>
        Art.portrait_frame(artwork, name)
      | _ => Art.unit_frame(artwork, ~side, name)
      };
    option(frame, frame =>
      <svg
        className={j|art-sprite pixel-art $(className)|j}
        width={number(width)}
        height={number(height)}
        viewBox={
                  let js_text_part_1 = number(frame.width);
                  let js_text_part_3 = number(frame.height);
                  {j|0 0 $(js_text_part_1) $(js_text_part_3)|j};
                }
        ariaHidden=true
        focusable={js|false|js}>
        <image
          href={frame.url}
          width={number(frame.width)}
          height={number(frame.height)}
          preserveAspectRatio={js|none|js}
        />
      </svg>
    );
  };
};
