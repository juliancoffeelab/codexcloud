open Ui;
[@react.component]
let make = (~session: Session.t, ~onStart) => {
  let (trace, setTrace) = React.useState(Performance_trace.state);
  let (error, setError) = React.useState(() => "");
  React.useEffect0(() => Some(Performance_trace.subscribe(value => setTrace(_ => value))));
  let context = () => Session.trace_context(session);
  let start = () => {
    setError(_ => "");
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
    setError(_ => "");
    let _ =
      Performance_trace.download()
      |> Js.Promise.catch(cause => {
           setError(_ => Foreign_error.describe(cause));
           Js.Promise.resolve();
         });
    ();
  };
  <>
    <p className="trace-controls-copy">
      {text(
         "Record timings while you play, then download the JSON trace to attach here. Recording stops after ten minutes or 250,000 events. Recordings stay in memory until reload. No recording is uploaded automatically.",
       )}
    </p>
    <div className="trace-buttons">
      <button
        onClick={_ => start()}
        disabled={
          trace.status == Performance_trace.Recording || trace.status == Performance_trace.Stopping
        }>
        {text(
           trace.status == Performance_trace.Ready ? "Start new trace" : "Start recording trace",
         )}
      </button>
      <button
        onClick={_ => {
          let _ = Performance_trace.stop();
          ();
        }}
        disabled={trace.status != Performance_trace.Recording}>
        {text("Stop recording")}
      </button>
      <button
        onClick={_ => download()}
        disabled={
          trace.status == Performance_trace.Empty || trace.status == Performance_trace.Stopping
        }>
        {text(
           trace.status == Performance_trace.Recording
             ? "Stop and download trace" : "Download trace",
         )}
      </button>
    </div>
    <p className="trace-controls-copy" role="status">
      {text(
         (
           switch (trace.status) {
           | Recording => "Recording"
           | Stopping => "Finishing trace…"
           | Ready => "Trace ready"
           | Empty => "Not recording"
           }
         )
         ++ " · "
         ++ string_of_int(trace.events)
         ++ " events"
         ++ Option.fold(~none="", ~some=reason => " · " ++ reason, trace.reason),
       )}
    </p>
    {error != ""
       ? <p className="trace-controls-copy" role="alert"> {text(error)} </p> : React.null}
    <p className="trace-controls-copy">
      {text(
         "Includes action/engine timings, browser frame intervals, pointer movement, camera/boundary samples, gesture cancellation, inertia, and CPU render-submission timings. GPU completion and physical display FPS are not measured.",
       )}
    </p>
  </>;
};
