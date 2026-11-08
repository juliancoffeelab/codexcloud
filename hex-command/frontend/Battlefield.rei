[@react.component]
let make:
  (
    ~view: Session.model,
    ~actions: Session.actions,
    ~renderer: Phaser_boot.t,
    ~previews: Terrain_preview.t,
    ~onRendererFailed: string => unit,
    ~prepareReload: string => Js.Promise.t(unit),
    ~showCommanderArrow: bool=?
  ) =>
  React.element;
