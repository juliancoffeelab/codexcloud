type phase =
  | Loading
  | Preparing
  | Ready;
[@mel.set] external markPhaser: (ReactDOM.domProps, string) => unit = "data-phaser-battlefield";

/* The startup shell and live renderer share this lightweight frame. It owns
   geometry and loading UI; mounting Phaser belongs to Battlefield alone. */
[@react.component]
let make =
    (
      ~host: option(React.ref(Js.Nullable.t(Dom.element)))=?,
      ~phase=Loading,
      ~loaded=0,
      ~total=0,
      ~navigation=React.null,
      ~children=React.null,
    ) => {
  let text = value => React.string(Utf8.to_js(value));
  let ready = phase == Ready;
  let props =
    ReactDOM.domProps(
      ~id="map",
      ~className="map-scroll phaser-battlefield battlefield-window" ++ (ready ? " map-ready" : ""),
      ~ariaBusy=!ready,
      (),
    );
  markPhaser(props, "");
  <section className="map-section" ariaLabel="Battlefield">
    {ReactDOM.createElement(
       "div",
       ~props,
       [|
         <div className="battlefield-clip" ref=?{Option.map(ReactDOM.Ref.domRef, host)}>
           navigation
         </div>,
         ready
           ? React.null
           : <div className="map-loading" role="status">
               <div className="loading-card">
                 <strong ariaLive="polite">
                   {text(
                      phase == Preparing ? "Preparing battlefield…" : "Loading battlefield…",
                    )}
                 </strong>
                 <progress
                   ariaLabel="Current battlefield resources"
                   max={string_of_int(max(1, total))}
                   value=?{phase == Loading && total > 0 ? Some(string_of_int(loaded)) : None}
                 />
                 <span ariaHidden=true>
                   {text(
                      total > 0 && phase == Loading
                        ? string_of_int(loaded) ++ " / " ++ string_of_int(total) ++ " ready"
                        : "Preparing game resources…",
                    )}
                 </span>
               </div>
             </div>,
         children,
       |],
     )}
  </section>;
};
