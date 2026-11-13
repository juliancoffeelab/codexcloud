open Ui;
[@mel.set] external setStyle: (ReactDOM.domProps, ReactDOM.Style.t) => unit = "style";
[@mel.set] external windowWidth: (ReactDOM.Style.t, string) => unit = "--window-width";
[@react.component]
let make =
    (
      ~title,
      ~dialog,
      ~id="",
      ~width="420px",
      ~tall=false,
      ~canDismiss=true,
      ~releaseControls=false,
      ~className="",
      ~onBeforeClose=() => (),
      ~onBack: option(unit => unit)=?,
      ~body: option(React.ref(Js.Nullable.t(Dom.element)))=?,
      ~headerArtwork: option(React.element)=?,
      ~footer: option(React.element)=?,
      ~children,
    ) => {
  let dismiss = () =>
    if (canDismiss) {
      onBeforeClose();
      close(dialog);
    };
  let (backdropDown, backdropUp, backdropCancel, backdropClick) = useBackdrop(dialog, dismiss);
  let props =
    ReactDOM.domProps(
      ~id,
      ~className="dialog-window " ++ className ++ (tall ? " tall" : ""),
      ~ref=ReactDOM.Ref.domRef(dialog),
      ~ariaLabel=Utf8.to_js(title),
      ~onPointerDown=backdropDown,
      ~onPointerUp=backdropUp,
      ~onPointerCancel=backdropCancel,
      ~onClick=backdropClick,
      (),
    );
  onCancel(props, event =>
    if (!canDismiss) {
      React.Event.Form.preventDefault(event);
    } else {
      onBeforeClose();
    }
  );
  let style = ReactDOM.Style.make();
  windowWidth(style, width);
  setStyle(props, style);
  ReactDOM.createElement(
    "dialog",
    ~props,
    [|
      <header className={onBack == None ? "" : "with-back"}>
        {option(onBack, back =>
           <button
             className={releaseControls ? "window-icon" : "window-icon press-control"}
             onPointerDown=?{releaseControls ? None : Some(Button_press.pointer_down(back))}
             onClick={releaseControls ? _ => back() : Button_press.click(back)}
             ariaLabel="Back to Menu">
             <svg viewBox="0 0 24 24" ariaHidden=true> <path d="m14 5-7 7 7 7" /> </svg>
           </button>
         )}
        {option(headerArtwork, artwork => artwork)}
        <h2 tabIndex=(-1)> {text(title)} </h2>
        {canDismiss
           ? <button
               className={releaseControls ? "window-icon" : "window-icon press-control"}
               onPointerDown=?{releaseControls ? None : Some(Button_press.pointer_down(dismiss))}
               onClick={releaseControls ? _ => dismiss() : Button_press.click(dismiss)}
               ariaLabel={Utf8.to_js("Close " ++ String.lowercase_ascii(title))}>
               <svg viewBox="0 0 24 24" ariaHidden=true> <path d="m6 6 12 12M18 6 6 18" /> </svg>
             </button>
           : React.null}
      </header>,
      switch (body) {
      | None => <div className="window-content"> children </div>
      | Some(body) =>
        <div className="window-content" ref={ReactDOM.Ref.domRef(body)}> children </div>
      },
      option(footer, node => node),
    |],
  );
};
