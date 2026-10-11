let text = React.string;
let number = Js.Float.toString;
let num = value => text(number(value));
let filter = (predicate, items) =>
  items |> Array.to_list |> List.filter(predicate) |> Array.of_list;
let each = (items, render) => React.array(Array.map(render, items));
let option = (value, render) =>
  switch (value) {
  | None => React.null
  | Some(value) => render(value)
  };
let nullable = (value, render) => option(Js.Nullable.toOption(value), render);
let percent = value => ReactDOM.Style.make(~width=number(value) ++ "%", ());
let side = (value: Domain.side) => (value :> string);
let ignorePromise = work => work |> Js.Promise.catch(_ => Js.Promise.resolve()) |> ignore;

/* Native DOM operations own no application policy or mutable game state. */
[@mel.send] external showModal: Dom.element => unit = "showModal";
[@mel.send] external closeModal: Dom.element => unit = "close";
[@mel.get] external isOpen: Dom.element => bool = "open";
[@mel.send] external focus: Dom.element => unit = "focus";
[@mel.scope "document"] external byId: string => Js.Nullable.t(Dom.element) = "getElementById";
[@mel.scope "document"] external query: string => Js.Nullable.t(Dom.element) = "querySelector";
[@mel.send]
external queryWithin: (Dom.element, string) => Js.Nullable.t(Dom.element) = "querySelector";
[@mel.get] external currentTarget: 'a => Dom.element = "currentTarget";
[@mel.get] external target: 'a => Dom.element = "target";
[@mel.get] external nodeType: Dom.element => int = "nodeType";
[@mel.send] external closest: (Dom.element, string) => Js.Nullable.t(Dom.element) = "closest";
[@mel.get] external inputValue: Dom.element => string = "value";
[@mel.get] external inputChecked: Dom.element => bool = "checked";
[@mel.get] external eventKey: 'a => string = "key";
[@mel.get] external eventRepeat: 'a => bool = "repeat";
[@mel.send] external preventDefault: 'a => unit = "preventDefault";
[@mel.send] external addListener: (Dom.element, string, 'a => unit) => unit = "addEventListener";
[@mel.send]
external removeListener: (Dom.element, string, 'a => unit) => unit = "removeEventListener";
external document: Dom.element = "document";
[@mel.send] external scroll: (Dom.element, {. "block": string }) => unit = "scrollIntoView";
let show = reference =>
  Js.Nullable.toOption(reference.React.current)
  |> Option.iter(element =>
       if (!isOpen(element)) {
         showModal(element);
       }
     );
let close = reference =>
  Js.Nullable.toOption(reference.React.current)
  |> Option.iter(element =>
       if (isOpen(element)) {
         closeModal(element);
       }
     );
let bindValue = event => inputValue(currentTarget(event));
let bindChecked = event => inputChecked(currentTarget(event));
type bounds = {
  left: float,
  right: float,
  top: float,
  bottom: float,
};
[@mel.send] external bounds: Dom.element => bounds = "getBoundingClientRect";
let backdrop = event => {
  let element = currentTarget(event);
  if (target(event) === element) {
    let b = bounds(element);
    let x = float_of_int(React.Event.Mouse.clientX(event));
    let y = float_of_int(React.Event.Mouse.clientY(event));
    if (x < b.left || x > b.right || y < b.top || y > b.bottom) {
      closeModal(element);
    };
  };
};
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
[@mel.set] external onCancel: (ReactDOM.domProps, React.Event.Form.t => unit) => unit = "onCancel";

module Icon = {
  [@react.component]
  let make = (~name) =>
    if (name == "blade"
        || name == "pierce"
        || name == "impact"
        || name == "move"
        || name == "search") {
      <span
        className="icon-container"
        dangerouslySetInnerHTML={ "__html": Art.icon_markup(name) }
      />;
    } else {
      <svg viewBox="0 0 24 24" ariaHidden=true>
        <path
          d={
            switch (name) {
            | "left" => "m15 5-7 7 7 7"
            | "right" => "m9 5 7 7-7 7"
            | _ => ""
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
        ~kind="units",
        ~name,
        ~side="player",
        ~floor=None,
        ~width,
        ~height,
        ~className="",
      ) => {
    let frame =
      switch (kind) {
      | "terrain" =>
        switch (floor) {
        | None => Art.terrain_frame(artwork, name)
        | Some(floor) => Art.terrain_frame(artwork, ~floor, name)
        }
      | "portraits" => Art.portrait_frame(artwork, name)
      | _ => Art.unit_frame(artwork, ~side, name)
      };
    option(frame, frame =>
      <svg
        className={"art-sprite pixel-art " ++ className}
        width={number(width)}
        height={number(height)}
        viewBox={"0 0 " ++ number(frame.width) ++ " " ++ number(frame.height)}
        ariaHidden=true
        focusable="false">
        <image
          href={frame.url}
          width={number(frame.width)}
          height={number(frame.height)}
          preserveAspectRatio="none"
        />
      </svg>
    );
  };
};
