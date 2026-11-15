open Ui;
[@mel.set] external windowWidth: (ReactDOM.Style.t, string) => unit = "--window-width";
module Content = {
  [@react.component]
  let make =
      (
        ~title,
        ~onOpenChange,
        ~id={js||js},
        ~width={js|420px|js},
        ~tall=false,
        ~canDismiss=true,
        ~alert=false,
        ~className={js||js},
        ~onBeforeClose=() => (),
        ~onBack: option(unit => unit)=?,
        ~body: option(React.ref(Js.Nullable.t(Dom.element)))=?,
        ~onOpenAutoFocus: option(Modal.focus_event => unit)=?,
        ~onCloseAutoFocus: option(Modal.focus_event => unit)=?,
        ~contentRef: option(React.ref(Js.Nullable.t(Dom.element)))=?,
        ~headerArtwork: option(React.element)=?,
        ~footer: option(React.element)=?,
        ~children,
      ) => {
    let change = next =>
      if (next || canDismiss) {
        if (!next) {
          onBeforeClose();
        };
        onOpenChange(next);
      };
    let style = ReactDOM.Style.make();
    windowWidth(style, width);
    <Modal.Content
      title
      id
      ?contentRef
      canDismiss
      alert
      style
      className={
                  let js_text_part_2 = tall ? {js| tall|js} : {js||js};
                  {j|dialog-window $(className)$(js_text_part_2)|j};
                }
      ?onOpenAutoFocus
      ?onCloseAutoFocus>
      <header className={onBack == None ? {js||js} : {js|with-back|js}}>
        {option(onBack, back =>
           <Press_button onActivate=back>
             <button className={js|window-icon|js} ariaLabel={js|Back to Menu|js}>
               <svg viewBox={js|0 0 24 24|js} ariaHidden=true>
                 <path d={js|m14 5-7 7 7 7|js} />
               </svg>
             </button>
           </Press_button>
         )}
        {option(headerArtwork, artwork => artwork)}
        <h2 tabIndex=(-1)> {text(title)} </h2>
        {canDismiss
           ? <Press_button onActivate={() => change(false)}>
               <button
                 className={js|window-icon|js}
                 ariaLabel={
                             let js_text_part_1 = Js.String.toLowerCase(title);
                             {j|Close $(js_text_part_1)|j};
                           }>
                 <svg viewBox={js|0 0 24 24|js} ariaHidden=true>
                   <path d={js|m6 6 12 12M18 6 6 18|js} />
                 </svg>
               </button>
             </Press_button>
           : React.null}
      </header>
      {switch (body) {
       | None => <div className={js|window-content|js}> children </div>
       | Some(body) =>
         <div className={js|window-content|js} ref={ReactDOM.Ref.domRef(body)}> children </div>
       }}
      {option(footer, node => node)}
    </Modal.Content>;
  };
};
[@react.component]
let make =
    (
      ~title,
      ~opened,
      ~onOpenChange,
      ~id={js||js},
      ~width={js|420px|js},
      ~tall=false,
      ~canDismiss=true,
      ~alert=false,
      ~className={js||js},
      ~onBeforeClose=() => (),
      ~onBack=?,
      ~body=?,
      ~onOpenAutoFocus=?,
      ~onCloseAutoFocus=?,
      ~trigger=?,
      ~contentRef=?,
      ~headerArtwork=?,
      ~footer=?,
      ~children,
    ) => {
  let change = value => {
    if (!value) {
      onBeforeClose();
    };
    onOpenChange(value);
  };
  <Modal.Root opened onOpenChange=change canDismiss alert ?trigger>
    <Content
      title
      onOpenChange=change
      id
      width
      tall
      canDismiss
      alert
      className
      ?onBack
      ?body
      ?onOpenAutoFocus
      ?onCloseAutoFocus
      ?contentRef
      ?headerArtwork
      ?footer>
      children
    </Content>
  </Modal.Root>;
};
