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
      ~navigation=React.null,
      ~children=React.null,
    ) => {
  let text = value => React.string(Utf8.to_js(value));
  let ready = phase == Ready;
  let fill = React.useRef(Js.Nullable.null);
  let (visible, setVisible) = React.useState(() => !ready);
  let (loading, setLoading) = React.useState(Loading_progress.current);
  React.useLayoutEffect0(() => {
    let unsubscribe = Loading_progress.subscribe(value => setLoading(_ => value));
    setLoading(_ => Loading_progress.current());
    Some(unsubscribe);
  });
  React.useLayoutEffect1(
    () =>
      if (ready) {
        if (loading.value >= 5. && !Loading_motion.reduced_motion()) {
          let timer = Browser_time.timeout(() => setVisible(_ => false), 240);
          Some(() => Browser_time.clear_timeout(timer));
        } else {
          setVisible(_ => false);
          None;
        };
      } else {
        setVisible(_ => true);
        None;
      },
    [|ready|],
  );
  React.useLayoutEffect2(
    () => {Option.map(Loading_motion.animate, Js.Nullable.toOption(fill.current))},
    (loading, visible),
  );
  let caption = ready ? "Battlefield ready" : Loading_progress.label(loading.step);
  let lastCaption = React.useRef(caption);
  let previousCaption = React.useRef(None);
  if (lastCaption.current != caption) {
    previousCaption.current = Some(lastCaption.current);
    lastCaption.current = caption;
  };
  let hexClass = index =>
    "loading-hex"
    ++ (ready || index < loading.step - 1 ? " is-filled" : "")
    ++ (!ready && index == loading.step - 1 ? " is-current" : "");
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
         !visible && ready
           ? React.null
           : <div className={"map-loading" ++ (ready ? " is-complete" : "")} role="status">
               <div className="loading-card">
                 <div className="loading-hexes" ariaHidden=true>
                   {React.array(
                      Array.map(
                        index => <span key={string_of_int(index)} className={hexClass(index)} />,
                        [|0, 1, 2, 3, 4, 5, 6|],
                      ),
                    )}
                 </div>
                 <div className="loading-action" ariaLive="polite">
                   {Option.fold(
                      ~none=React.null,
                      ~some=
                        previous =>
                          <strong
                            key=previous className="loading-caption is-leaving" ariaHidden=true>
                            {text(previous)}
                          </strong>,
                      previousCaption.current,
                    )}
                   <strong
                     key=caption
                     className={
                       "loading-caption" ++ (previousCaption.current == None ? "" : " is-entering")
                     }>
                     {text(caption)}
                   </strong>
                 </div>
                 <div className="loading-track" ariaHidden=true>
                   <span className="loading-fill" ref={ReactDOM.Ref.domRef(fill)} />
                 </div>
                 <progress
                   className="loading-accessible-progress"
                   ariaLabel="Battlefield preparation"
                   ariaValuetext={Utf8.to_js(
                     ready
                       ? "Battlefield ready"
                       : "Step "
                         ++ string_of_int(loading.step)
                         ++ " of 5: "
                         ++ Loading_progress.label(loading.step)
                         ++ (loading.detail == "" ? "" : " " ++ loading.detail),
                   )}
                   max="5"
                   value={string_of_float(loading.value)}
                 />
                 <div className="loading-meta">
                   <span className="loading-step">
                     {text("Step " ++ string_of_int(loading.step) ++ " / 5")}
                   </span>
                   <span className="loading-detail"> {text(loading.detail)} </span>
                 </div>
               </div>
             </div>,
         children,
       |],
     )}
  </section>;
};
