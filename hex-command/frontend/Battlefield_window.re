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
  let text = value => React.string(value);
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
      ? {js|Trying to pick words…|js}
      : ready ? {js|Battlefield ready|js} : Loading_progress.label(loading.step);
  let lastCaption = React.useRef(caption);
  let previousCaption = React.useRef(None);
  if (lastCaption.current != caption) {
    previousCaption.current = Some(lastCaption.current);
    lastCaption.current = caption;
  };
  let hexClass = index => {
    let js_text_part_1 = ready || index < loading.step - 1 ? {js| is-filled|js} : {js||js};
    let js_text_part_2 = !ready && index == loading.step - 1 ? {js| is-current|js} : {js||js};
    {j|loading-hex$(js_text_part_1)$(js_text_part_2)|j};
  };
  let props =
    ReactDOM.domProps(
      ~id={js|map|js},
      ~className={
        let js_text_part_1 = ready ? {js| map-ready|js} : {js||js};
        {j|map-scroll phaser-battlefield battlefield-window$(js_text_part_1)|j};
      },
      ~ariaBusy=!ready,
      (),
    );
  markPhaser(props, {js||js});
  <section className={js|map-section|js} ariaLabel={js|Battlefield|js}>
    {ReactDOM.createElement(
       {js|div|js},
       ~props,
       [|
         <div className={js|battlefield-clip|js} ref=?{Option.map(ReactDOM.Ref.domRef, host)}>
           navigation
         </div>,
         !visible && ready
           ? React.null
           : <div
               className={
                           let js_text_part_1 = ready ? {js| is-complete|js} : {js||js};
                           let js_text_part_2 =
                             voiceScreen.current ? {js| is-voice-preparing|js} : {js||js};
                           {j|map-loading$(js_text_part_1)$(js_text_part_2)|j};
                         }
               role={js|status|js}>
               <div className={js|loading-card|js}>
                 <div className={js|loading-hexes|js} ariaHidden=true>
                   {React.array(
                      Array.map(
                        index => <span key={string_of_int(index)} className={hexClass(index)} />,
                        [|0, 1, 2, 3, 4, 5, 6|],
                      ),
                    )}
                 </div>
                 <div className={js|loading-action|js} ariaLive={js|polite|js}>
                   {Option.fold(
                      ~none=React.null,
                      ~some=
                        previous =>
                          <strong
                            key=previous
                            className={js|loading-caption is-leaving|js}
                            ariaHidden=true>
                            {text(previous)}
                          </strong>,
                      previousCaption.current,
                    )}
                   <strong
                     key=caption
                     className={
                                 let js_text_part_1 =
                                   previousCaption.current == None
                                     ? {js||js} : {js| is-entering|js};
                                 {j|loading-caption$(js_text_part_1)|j};
                               }>
                     {text(caption)}
                   </strong>
                 </div>
                 <div className={js|loading-track|js} ariaHidden=true>
                   <span
                     className={js|loading-fill|js}
                     ref={ReactDOM.Ref.domRef(fill)}
                     style=?{
                       preparation == None
                         ? None
                         : Some(
                             ReactDOM.Style.make(
                               ~transform={
                                 let js_text_part_1 = string_of_float(fraction);
                                 {j|scaleX($(js_text_part_1))|j};
                               },
                               ~animation={js|none|js},
                               (),
                             ),
                           )
                     }
                   />
                 </div>
                 <progress
                   className={js|loading-accessible-progress|js}
                   ariaLabel={
                     preparation == None
                       ? {js|Battlefield preparation|js} : {js|Dialogue voice preparation|js}
                   }
                   ariaValuetext={
                     preparation != None
                       ? {js|Trying to pick words…|js}
                       : ready
                           ? {js|Battlefield ready|js}
                           : {
                             let js_text_part_1 = string_of_int(loading.step);
                             let js_text_part_3 = Loading_progress.label(loading.step);
                             let js_text_part_4 =
                               loading.detail == {js||js}
                                 ? {js||js}
                                 : {
                                   let js_text_part_1 = loading.detail;
                                   {j| $(js_text_part_1)|j};
                                 };
                             {j|Step $(js_text_part_1) of 5: $(js_text_part_3)$(js_text_part_4)|j};
                           }
                   }
                   max={preparation == None ? {js|5|js} : {js|1|js}}
                   value={string_of_float(preparation == None ? loading.value : fraction)}
                 />
                 <div className={js|loading-meta|js}>
                   <span className={js|loading-step|js}>
                     {text(
                        preparation == None
                          ? {
                            let js_text_part_1 = string_of_int(loading.step);
                            {j|Step $(js_text_part_1) / 5|j};
                          }
                          : {js|Finding a voice|js},
                      )}
                   </span>
                   <span className={js|loading-detail|js}>
                     {text(
                        Option.fold(
                          ~none=loading.detail,
                          ~some=
                            (value: preparation) => {
                              let js_text_part_0 = string_of_int(value.completed);
                              let js_text_part_2 = string_of_int(value.total);
                              {j|$(js_text_part_0) / $(js_text_part_2) voices ready|j};
                            },
                          preparation,
                        ),
                      )}
                   </span>
                 </div>
                 {Option.fold(
                    ~none=React.null,
                    ~some=
                      reason =>
                        <div className={js|startup-failure|js} role={js|alert|js}>
                          <p> {text({js|The opening voices could not be prepared.|js})} </p>
                          <button className={js|quiet|js} onClick={_ => onRetryPreparation()}>
                            {text({js|Retry voices|js})}
                          </button>
                          <button className={js|quiet|js} onClick={_ => onSkipPreparation()}>
                            {text({js|Continue without voices|js})}
                          </button>
                          <details>
                            <summary> {text({js|Error details|js})} </summary>
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
