open Ui;
[@react.component]
let make = (~view: Session.model, ~actions: Session.actions, ~onManage) => {
  let (opened, setOpened) = React.useState(() => false);
  let shown = React.useRef(None);
  let failure = view.save_failure;
  React.useEffect1(
    () => {
      switch (failure) {
      | Some(failure) when Some(failure.kind) != shown.current =>
        shown.current = Some(failure.kind);
        setOpened(_ => true);
      | None =>
        setOpened(_ => false);
        shown.current = None;
      | _ => ()
      };
      None;
    },
    [|failure|],
  );
  let kind = Option.fold(~none=Save_error.Internal, ~some=(f: Save_error.t) => f.kind, failure);
  <Modal
    onCloseAutoFocus={event => {
      Modal.preventDefault(event);
      Option.iter(focus, Js.Nullable.toOption(byId({js|settings-button|js})));
    }}
    opened
    onOpenChange={value => setOpened(_ => value)}
    alert=true
    title={
      kind == Save_error.Full
        ? {js|Storage is full|js}
        : kind == Save_error.Blocked
            ? {js|Saving is unavailable|js} : {js|Progress couldn’t be saved|js}
    }
    className={js|save-failure-style save-failure-dialog|js}>
    <h2 id={js|save-failure-title|js}>
      {text(
         kind == Save_error.Full
           ? {js|Storage is full|js}
           : kind == Save_error.Blocked
               ? {js|Saving is unavailable|js} : {js|Progress couldn’t be saved|js},
       )}
    </h2>
    <p>
      {text(
         kind == Save_error.Full
           ? {js|Delete some older saves to make room for your progress.|js}
           : kind == Save_error.Blocked
               ? {js|This browser is blocking access to saved games. Allow site storage in your browser settings.|js}
               : {js|The game encountered an error while saving. Download a debug report so we can investigate.|js},
       )}
    </p>
    <p className={js|unsaved|js}>
      {text({js|Your latest progress hasn’t been fully saved.|js})}
    </p>
    {switch (kind) {
     | Save_error.Full =>
       <>
         <button
           className={js|primary|js}
           onClick={_ => {
             setOpened(_ => false);
             onManage();
           }}>
           {text({js|Manage saves|js})}
         </button>
         <Radix.AlertCancel asChild=true>
           <button className={js|keep-playing|js} onClick={_ => setOpened(_ => false)}>
             {text({js|Keep playing|js})}
           </button>
         </Radix.AlertCancel>
       </>
     | Save_error.Blocked =>
       <Radix.AlertCancel asChild=true>
         <button className={js|primary|js} onClick={_ => setOpened(_ => false)}>
           {text({js|Close|js})}
         </button>
       </Radix.AlertCancel>
     | Save_error.Internal =>
       <>
         <button className={js|primary|js} onClick={_ => actions.download_debug_report()}>
           {text({js|Download debug report|js})}
         </button>
         <Radix.AlertCancel asChild=true>
           <button className={js|quiet-close|js} onClick={_ => setOpened(_ => false)}>
             {text({js|Close|js})}
           </button>
         </Radix.AlertCancel>
       </>
     }}
  </Modal>;
};
