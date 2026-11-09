let text = value => React.string(Utf8.to_js(value));
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
let percent = value => ReactDOM.Style.make(~width=number(value) ++ "%", ());
let side = (value: Domain.Side.t) => Enum.string_of_side(value);
let ignorePromise = work => {
  let _ = work |> Js.Promise.catch(_ => Js.Promise.resolve());
  ();
};

include Browser_dom;

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
[@mel.get] external checkedTarget: React.Event.Form.t => {. "checked": bool } = "currentTarget";
[@mel.get] external checked: {. "checked": bool } => bool = "checked";
let bindChecked = event => checked(checkedTarget(event));
[@mel.get]
external eventCurrentTarget: React.Event.synthetic('a) => Dom.element = "currentTarget";
[@mel.get] external eventTarget: React.Event.synthetic('a) => Dom.element = "target";
[@mel.get] external eventX: React.Event.synthetic('a) => float = "clientX";
[@mel.get] external eventY: React.Event.synthetic('a) => float = "clientY";
let outsideDialog = event => {
  let element = eventCurrentTarget(event);
  if (eventTarget(event) === element) {
    let b = bounds(element);
    let x = eventX(event);
    let y = eventY(event);
    x < b.left || x > b.right || y < b.top || y > b.bottom;
  } else {
    false;
  };
};
type backdropGesture =
  | Idle
  | Pressed(Dom.eventPointerId)
  | Released;

/* A modal-opening press never started on the new modal's backdrop. */
let useBackdrop = (dialog, dismiss) => {
  let gesture = React.useRef(Idle);
  let reset = () => gesture.current = Idle;
  React.useEffect0(() =>
    switch (Js.Nullable.toOption(dialog.React.current)) {
    | None => None
    | Some(node) =>
      let closed = _ => reset();
      addListener(node, "close", closed);
      addListener(node, "cancel", closed);
      Some(
        () => {
          removeListener(node, "close", closed);
          removeListener(node, "cancel", closed);
        },
      );
    }
  );
  let down = event => {
    gesture.current =
      React.Event.Pointer.isPrimary(event)
      && React.Event.Pointer.button(event) == 0
      && outsideDialog(event)
        ? Pressed(React.Event.Pointer.pointerId(event)) : Idle;
  };
  let up = event => {
    gesture.current = (
      switch (gesture.current) {
      | Pressed(id) when id == React.Event.Pointer.pointerId(event) && outsideDialog(event) =>
        Released
      | _ => Idle
      }
    );
  };
  let click = event => {
    let completed = gesture.current == Released && outsideDialog(event);
    reset();
    if (completed) {
      dismiss();
    };
  };
  (down, up, _ => reset(), click);
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
      <svg
        className="ui-icon"
        viewBox="0 0 24 24"
        ariaHidden=true
        focusable="false"
        dangerouslySetInnerHTML={ "__html": Art.icon_path(name) }
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
