open Ui;
[@react.component]
let make = (~session: Session.t, ~onStart) => {
  let (trace, setTrace) = React.useState(Performance_trace.state);
  let (error, setError) = React.useState(() => {js||js});
  React.useEffect0(() => Some(Performance_trace.subscribe(value => setTrace(_ => value))));
  let context = () => Session.trace_context(session);
  let start = () => {
    setError(_ => {js||js});
    try(
      {
        Performance_trace.start(context);
        onStart();
      }
    ) {
    | cause => setError(_ => Foreign_error.exception_(cause))
    };
  };
  let download = () => {
    setError(_ => {js||js});
    let _ =
      Performance_trace.download()
      |> Js.Promise.catch(cause => {
           setError(_ => Foreign_error.describe(cause));
           Js.Promise.resolve();
         });
    ();
  };
  <>
    <p className={js|trace-controls-copy|js}>
      {text(
         {js|Record timings while you play, then download the JSON trace to attach here. Recording stops after ten minutes or 250,000 events. Recordings stay in memory until reload. No recording is uploaded automatically.|js},
       )}
    </p>
    <div className={js|trace-buttons|js}>
      <button
        onClick={_ => start()}
        disabled={
          trace.status == Performance_trace.Recording || trace.status == Performance_trace.Stopping
        }>
        {text(
           trace.status == Performance_trace.Ready
             ? {js|Start new trace|js} : {js|Start recording trace|js},
         )}
      </button>
      <button
        onClick={_ => {
          let _ = Performance_trace.stop();
          ();
        }}
        disabled={trace.status != Performance_trace.Recording}>
        {text({js|Stop recording|js})}
      </button>
      <button
        onClick={_ => download()}
        disabled={
          trace.status == Performance_trace.Empty || trace.status == Performance_trace.Stopping
        }>
        {text(
           trace.status == Performance_trace.Recording
             ? {js|Stop and download trace|js} : {js|Download trace|js},
         )}
      </button>
    </div>
    <p className={js|trace-controls-copy|js} role={js|status|js}>
      {text(
         {
           let js_text_part_0 =
             switch (trace.status) {
             | Recording => {js|Recording|js}
             | Stopping => {js|Finishing trace…|js}
             | Ready => {js|Trace ready|js}
             | Empty => {js|Not recording|js}
             };
           let js_text_part_2 = string_of_int(trace.events);
           let js_text_part_4 =
             Option.fold(~none={js||js}, ~some=reason => {j| · $(reason)|j}, trace.reason);
           {j|$(js_text_part_0) · $(js_text_part_2) events$(js_text_part_4)|j};
         },
       )}
    </p>
    {error != {js||js}
       ? <p className={js|trace-controls-copy|js} role={js|alert|js}> {text(error)} </p>
       : React.null}
    <p className={js|trace-controls-copy|js}>
      {text(
         {js|Includes action/engine timings, browser frame intervals, pointer movement, camera/boundary samples, gesture cancellation, inertia, and CPU render-submission timings. GPU completion and physical display FPS are not measured.|js},
       )}
    </p>
  </>;
};
