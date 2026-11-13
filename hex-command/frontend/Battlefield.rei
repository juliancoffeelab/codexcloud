[@react.component]
let make:
  (
    ~view: Session.model,
    ~actions: Session.actions,
    ~renderer: Phaser_boot.t,
    ~previews: Terrain_preview.t,
    ~progress: option(Battlefield_view.progress),
    ~onProgress: Battlefield_view.progress => unit,
    ~onRendererFailed: string => unit,
    ~prepareReload: string => Js.Promise.t(unit),
    ~showCommanderArrow: bool=?,
    ~voicePreparation: option(Battlefield_window.preparation)=?,
    ~onRetryVoice: unit => unit=?,
    ~onSkipVoice: unit => unit=?
  ) =>
  React.element;
