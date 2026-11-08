open Ui;
[@react.component]
let make = (~view: Session.model, ~actions: Session.actions, ~onManage) => {
  let dialog = React.useRef(Js.Nullable.null);
  let shown = React.useRef(None);
  let failure = view.save_failure;
  React.useEffect1(
    () => {
      switch (failure) {
      | Some(failure) when Some(failure.kind) != shown.current =>
        shown.current = Some(failure.kind);
        show(dialog);
      | None =>
        close(dialog);
        shown.current = None;
      | _ => ()
      };
      None;
    },
    [|failure|],
  );
  let kind = Option.fold(~none=Save_error.Internal, ~some=(f: Save_error.t) => f.kind, failure);
  <div className="save-failure-style">
    <dialog
      className="save-failure-dialog"
      ref={ReactDOM.Ref.domRef(dialog)}
      ariaLabelledby="save-failure-title">
      <h2 id="save-failure-title">
        {text(
           kind == Save_error.Full
             ? "Storage is full"
             : kind == Save_error.Blocked
                 ? "Saving is unavailable" : "Progress couldn’t be saved",
         )}
      </h2>
      <p>
        {text(
           kind == Save_error.Full
             ? "Delete some older saves to make room for your progress."
             : kind == Save_error.Blocked
                 ? "This browser is blocking access to saved games. Allow site storage in your browser settings."
                 : "The game encountered an error while saving. Download a debug report so we can investigate.",
         )}
      </p>
      <p className="unsaved"> {text("Your latest progress hasn’t been fully saved.")} </p>
      {switch (kind) {
       | Save_error.Full =>
         <>
           <button
             className="primary"
             onClick={_ => {
               close(dialog);
               onManage();
             }}>
             {text("Manage saves")}
           </button>
           <button className="keep-playing" onClick={_ => close(dialog)}>
             {text("Keep playing")}
           </button>
         </>
       | Save_error.Blocked =>
         <button className="primary" onClick={_ => close(dialog)}> {text("Close")} </button>
       | Save_error.Internal =>
         <>
           <button className="primary" onClick={_ => actions.download_debug_report()}>
             {text("Download debug report")}
           </button>
           <button className="quiet-close" onClick={_ => close(dialog)}>
             {text("Close")}
           </button>
         </>
       }}
    </dialog>
  </div>;
};
