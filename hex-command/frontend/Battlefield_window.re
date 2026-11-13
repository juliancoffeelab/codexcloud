type phase =
  | Loading
  | Preparing
  | Ready;
type preparation = {
  completed: int,
  total: int,
  error: option(string),
};
[@mel.set] external markPhaser: (ReactDOM.domProps, string) => unit = "data-phaser-battlefield";

/* The startup shell and live renderer share this lightweight frame. It owns
   geometry and loading UI; mounting Phaser belongs to Battlefield alone. */
[@react.component]
let make =
    (
      ~host: option(React.ref(Js.Nullable.t(Dom.element)))=?,
      ~phase=Loading,
      ~preparation=None,
      ~onRetryPreparation=() => (),
      ~onSkipPreparation=() => (),
      ~navigation=React.null,
      ~children=React.null,
    ) => {
  let text = value => React.string(Utf8.to_js(value));
  let ready = phase == Ready && preparation == None;
  let voiceScreen = React.useRef(false);
  if (preparation != None) {
    voiceScreen.current = true;
  } else if (!ready) {
    voiceScreen.current = false;
  };
  let fraction =
    Option.fold(
      ~none=0.,
      ~some=
        (value: preparation) =>
          float_of_int(value.completed) /. float_of_int(max(1, value.total)),
      preparation,
    );
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
  React.useLayoutEffect3(
    () =>
      preparation == None
        ? Option.map(Loading_motion.animate, Js.Nullable.toOption(fill.current)) : None,
    (loading, visible, preparation),
  );
  let caption =
    preparation != None
      ? "Trying to pick words…"
      : ready ? "Battlefield ready" : Loading_progress.label(loading.step);
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
           : <div
               className={
                 "map-loading"
                 ++ (ready ? " is-complete" : "")
                 ++ (voiceScreen.current ? " is-voice-preparing" : "")
               }
               role="status">
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
                   <span
                     className="loading-fill"
                     ref={ReactDOM.Ref.domRef(fill)}
                     style=?{
                       preparation == None
                         ? None
                         : Some(
                             ReactDOM.Style.make(
                               ~transform="scaleX(" ++ string_of_float(fraction) ++ ")",
                               ~animation="none",
                               (),
                             ),
                           )
                     }
                   />
                 </div>
                 <progress
                   className="loading-accessible-progress"
                   ariaLabel={
                     preparation == None ? "Battlefield preparation" : "Dialogue voice preparation"
                   }
                   ariaValuetext={Utf8.to_js(
                     preparation != None
                       ? "Trying to pick words…"
                       : ready
                           ? "Battlefield ready"
                           : "Step "
                             ++ string_of_int(loading.step)
                             ++ " of 5: "
                             ++ Loading_progress.label(loading.step)
                             ++ (loading.detail == "" ? "" : " " ++ loading.detail),
                   )}
                   max={preparation == None ? "5" : "1"}
                   value={string_of_float(preparation == None ? loading.value : fraction)}
                 />
                 <div className="loading-meta">
                   <span className="loading-step">
                     {text(
                        preparation == None
                          ? "Step " ++ string_of_int(loading.step) ++ " / 5" : "Finding a voice",
                      )}
                   </span>
                   <span className="loading-detail">
                     {text(
                        Option.fold(
                          ~none=loading.detail,
                          ~some=
                            (value: preparation) =>
                              string_of_int(value.completed)
                              ++ " / "
                              ++ string_of_int(value.total)
                              ++ " voices ready",
                          preparation,
                        ),
                      )}
                   </span>
                 </div>
                 {Option.fold(
                    ~none=React.null,
                    ~some=
                      reason =>
                        <div className="startup-failure" role="alert">
                          <p> {text("The opening voices could not be prepared.")} </p>
                          <button className="quiet" onClick={_ => onRetryPreparation()}>
                            {text("Retry voices")}
                          </button>
                          <button className="quiet" onClick={_ => onSkipPreparation()}>
                            {text("Continue without voices")}
                          </button>
                          <details>
                            <summary> {text("Error details")} </summary>
                            <pre> {text(reason)} </pre>
                          </details>
                        </div>,
                    Option.bind(preparation, value => value.error),
                  )}
               </div>
             </div>,
         children,
       |],
     )}
  </section>;
};
