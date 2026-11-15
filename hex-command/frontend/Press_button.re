type installation;
[@mel.module {js|@presentation/press-button.mjs|js}]
external installGuard: (Dom.event => unit) => installation = "installPressGuard";
[@mel.send] external dispose: installation => unit = "dispose";
[@react.component] [@mel.module {js|@presentation/press-button.mjs|js}]
external make: (~onActivate: unit => unit, ~children: React.element) => React.element =
  "PressButton";
