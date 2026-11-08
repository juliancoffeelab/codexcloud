open Ui;
let jt = Save_json.text;
let jf = Save_json.field;
let ja = Save_json.array;
let jb = Save_json.b;
[@mel.set] external dataRepairKey: (ReactDOM.domProps, string) => unit = "data-repair-key";
[@mel.set] external dataRepairRun: (ReactDOM.domProps, string) => unit = "data-repair-run";
[@react.component]
let make = (~session, ~view: Session.model, ~onNewGame, ~onManage, ~openKey=0) => {
  let dialog = React.useRef(Js.Nullable.null);
  let confirmation = React.useRef(Js.Nullable.null);
  let shown = React.useRef(None);
  let (reviewing, setReviewing) = React.useState(() => false);
  let (busy, setBusy) = React.useState(() => false);
  let (error, setError) = React.useState(() => "");
  let (inventory, setInventory) =
    React.useState(() =>
      Save_json.obj([
        ("records", Save_json.arr([])),
        ("errors", Save_json.arr([])),
        ("authoritative", Save_json.bool(false)),
        ("completedElsewhere", Save_json.bool(false)),
      ])
    );
  let (pending, setPending) = React.useState(() => None);
  let (armed, setArmed) = React.useState(() => false);
  let timer = React.useRef(None);
  let mounted = React.useRef(true);
  let generation = React.useRef(0);
  let dismissConfirmation = () => {
    close(confirmation);
    setPending(_ => None);
    setArmed(_ => false);
    Option.iter(Js.Global.clearTimeout, timer.current);
    timer.current = None;
  };
  let refresh = () => {
    let request = generation.current;
    Session.repair_inventory(session)
    |> Js.Promise.then_(value => {
         if (mounted.current && request == generation.current) {
           setInventory(_ => value);
         };
         Js.Promise.resolve();
       });
  };
  let refreshSafely = () => {
    setBusy(_ => true);
    let _ =
      refresh()
      |> Js.Promise.then_(() => {
           if (mounted.current) {
             setBusy(_ => false);
           };
           Js.Promise.resolve();
         })
      |> Js.Promise.catch(e => {
           if (mounted.current) {
             setError(_ => Save_json.detail(e));
             setBusy(_ => false);
           };
           Js.Promise.resolve();
         });
    ();
  };
  let openReview = () => {
    dismissConfirmation();
    setReviewing(_ => true);
    show(dialog);
    refreshSafely();
  };
  React.useEffect1(
    () => {
      mounted.current = true;
      Some(
        () => {
          mounted.current = false;
          generation.current = generation.current + 1;
          Option.iter(Js.Global.clearTimeout, timer.current);
        },
      );
    },
    [|session|],
  );
  React.useEffect1(
    () => {
      switch (view.load_problem) {
      | None =>
        dismissConfirmation();
        close(dialog);
        shown.current = None;
      | Some(problem) when view.load_problem != shown.current =>
        if (jt(problem, "kind") == "ready"
            && !Option.fold(~none=false, ~some=p => jt(p, "kind") == "ready", shown.current)) {
          dismissConfirmation();
        };
        shown.current = view.load_problem;
        let opened =
          Option.fold(~none=false, ~some=isOpen, Js.Nullable.toOption(dialog.current));
        if (!opened) {
          let review = jt(problem, "kind") == "data" || jt(problem, "kind") == "ready";
          setReviewing(_ => review);
          show(dialog);
          if (review) {
            refreshSafely();
          };
        };
      | _ => ()
      };
      None;
    },
    [|view.load_problem|],
  );
  React.useEffect1(
    () => {
      if (openKey > 0) {
        openReview();
      };
      None;
    },
    [|openKey|],
  );
  let model = Repair_view.model(view.catalog, inventory, view.load_problem);
  let kind = Option.fold(~none="", ~some=p => jt(p, "kind"), view.load_problem);
  let title =
    reviewing
      ? model.ready ? "Saved games are ready" : "Review saved data"
      : kind == "read"
          ? "Couldn’t read saved progress"
          : kind == "full"
              ? "Not enough space"
              : kind == "blocked" ? "Storage access is blocked" : "Review saved data";
  let ask = (target, message) =>
    if (!busy) {
      setPending(_ => Some((target, message)));
      setArmed(_ => false);
      Option.iter(Js.Global.clearTimeout, timer.current);
      timer.current =
        Some(
          Js.Global.setTimeout(
            ~f=
              () =>
                if (mounted.current) {
                  setArmed(_ => true);
                },
            350,
          ),
        );
      show(confirmation);
      Js.Nullable.toOption(confirmation.current)
      |> Option.iter(node =>
           Js.Nullable.toOption(queryWithin(node, ".cancel")) |> Option.iter(focus)
         );
    };
  let target = (kind, key, value) =>
    Save_json.obj([("kind", Save_json.str(kind)), (key, Save_json.str(value))]);
  let resume = () => {
    let flash = Option.bind(model.flash, row => Save_json.get(row, "entry"));
    let message =
      Repair_view.confirmation(
        flash == None ? "Finish and choose new game?" : "Restore saved game?",
        Option.fold(
          ~none="",
          ~some=
            e =>
              Save_browser.run_name(view.catalog, e)
              ++ " · "
              ++ Save_browser.save_time(Save_json.n(e, "savedAt")),
          flash,
        ),
        flash == None
          ? "There is no flash save to restore. This finishes the update and opens New game, replacing any current unsaved play."
          : "Your current unsaved play will be replaced by this flash save.",
        flash == None ? "Finish update" : "Restore saved game",
        false,
        "",
        false,
      );
    ask(
      Save_json.obj([
        ("kind", Save_json.str("resume")),
        ("expected", Option.value(~default=Js.Json.null, flash)),
      ]),
      message,
    );
  };
  let readAgain = () =>
    ask(
      Save_json.obj([("kind", Save_json.str("resume"))]),
      Repair_view.confirmation(
        "Try reading saved progress again?",
        "",
        "Stored progress hasn’t been identified. If it can be read, its flash save will replace your current unsaved play. If there is no flash save, New game will open.",
        "Try reading again",
        false,
        "",
        false,
      ),
    );
  let reset = () =>
    ask(
      Save_json.obj([("kind", Save_json.str("all"))]),
      Repair_view.confirmation(
        "Delete all retained saves?",
        "",
        "All retained saves will be deleted, including flash saves, unidentified saves and save-list entries.",
        "Delete all retained saves",
        true,
        "",
        false,
      ),
    );
  let retryRead = () => {
    setBusy(_ => true);
    setError(_ => "");
    let _ =
      Session.retry_saved_progress(session, None)
      |> Js.Promise.then_(success => {
           if (mounted.current) {
             setBusy(_ => false);
             if (!success) {
               setError(_ =>
                 "Saved progress still couldn’t be read. Stored data has not been replaced."
               );
             };
           };
           Js.Promise.resolve();
         })
      |> Js.Promise.catch(e => {
           if (mounted.current) {
             setBusy(_ => false);
             setError(_ => Save_json.detail(e));
           };
           Js.Promise.resolve();
         });
    ();
  };
  let confirmAction = () =>
    switch (pending) {
    | Some((target, _)) when !busy && armed =>
      setBusy(_ => true);
      setError(_ => "");
      let operation =
        if (jt(target, "kind") == "resume") {
          let expected =
            switch (Save_json.get(target, "expected")) {
            | None => None
            | Some(value) => Some(value == Js.Json.null ? None : Some(value))
            };
          Session.retry_saved_progress(session, expected)
          |> Js.Promise.then_(success =>
               if (success) {
                 close(dialog);
                 close(confirmation);
                 Js.Promise.resolve();
               } else {
                 refresh();
               }
             );
        } else {
          Session.repair_saved_data(session, target) |> Js.Promise.then_(() => refresh());
        };
      let _ =
        operation
        |> Js.Promise.then_(() => {
             if (mounted.current) {
               setBusy(_ => false);
               close(confirmation);
             };
             Js.Promise.resolve();
           })
        |> Js.Promise.catch(e => {
             if (mounted.current) {
               setBusy(_ => false);
               setError(_ => Save_json.detail(e));
             };
             refresh() |> Js.Promise.catch(_ => Js.Promise.resolve());
           });
      ();
    | _ => ()
    };
  let row = (record, attention) => {
    let entry = jf(record, "entry");
    let name = Repair_view.record_name(view.catalog, record);
    let unknown = jt(record, "kind") == "flash" && jt(record, "ownership") != "known";
    let props =
      ReactDOM.domProps(
        ~className=
          "record" ++ (attention ? " issue" : "") ++ (attention && unknown ? " warning" : ""),
        (),
      );
    dataRepairKey(props, Utf8.to_js(jt(record, "key")));
    let remove = () =>
      ask(
        target("record", "key", jt(record, "key")),
        Repair_view.record_confirmation(view.catalog, record),
      );
    ReactDOM.createElement(
      "div",
      ~props,
      [|
        <div className="record-top">
          <div className="copy">
            {attention && entry != Js.Json.null
               ? <p className="campaign-context">
                   {text(Save_browser.run_name(view.catalog, entry))}
                 </p>
               : React.null}
            {let entry = jf(record, "entry");
             let (number, chapter) = Save_browser.chapter(view.catalog, entry);
             jt(entry, "kind") == "turn" && number > 0
               ? <span className="turn-name">
                   <span className="numeral"> {text(Save_browser.roman(number))} </span>
                   <strong>
                     {text(chapter ++ ", " ++ Ui.number(Save_json.n(entry, "round")))}
                   </strong>
                 </span>
               : <strong> {text(name)} </strong>}
            {entry != Js.Json.null
               ? <p className="time">
                   {text(Save_browser.save_time(Save_json.n(entry, "savedAt")))}
                 </p>
               : React.null}
            {Save_json.get(record, "issue") != None
               ? <p className="damage"> {text(Repair_view.issue(record))} </p> : React.null}
          </div>
          {jt(record, "kind") != "index"
           && !unknown
           && jt(record, "ownership") != "conflicting"
           && jt(record, "issue") != "missing_snapshot"
             ? <button
                 className="trash"
                 disabled=busy
                 ariaLabel={Utf8.to_js("Delete " ++ name)}
                 onClick={_ => remove()}>
                 <Save_icon name="trash" />
               </button>
             : React.null}
        </div>,
        unknown
          ? <>
              <p>
                {text(
                   "Deleting a campaign will leave this flash save untouched. We strongly recommend deleting it separately.",
                 )}
              </p>
              <button className="delete-link" disabled=busy onClick={_ => remove()}>
                <Save_icon name="trash" />
                {text("Delete flash save")}
              </button>
            </>
          : jt(record, "issue") == "missing_snapshot"
              ? <button className="delete-link" disabled=busy onClick={_ => remove()}>
                  <Save_icon name="trash" />
                  {text("Remove missing turn from list")}
                </button>
              : jt(record, "ownership") == "conflicting"
                  ? <>
                      <p>
                        {text(
                           "This turn is linked to "
                           ++ string_of_int(
                                List.length(
                                  List.sort_uniq(
                                    String.compare,
                                    Array.to_list(ja(jf(record, "claims")))
                                    |> List.map(e => jt(e, "runId")),
                                  ),
                                ),
                              )
                           ++ " different playthroughs: "
                           ++ String.concat(
                                " and ",
                                List.sort_uniq(
                                  String.compare,
                                  Array.to_list(ja(jf(record, "claims")))
                                  |> List.map(Save_browser.run_name(view.catalog)),
                                ),
                              )
                           ++ ". Deleting a campaign will leave this turn untouched.",
                         )}
                      </p>
                      <button className="delete-link" disabled=busy onClick={_ => remove()}>
                        <Save_icon name="trash" />
                        {text("Delete unidentified turn save")}
                      </button>
                    </>
                  : jt(record, "kind") == "index"
                      ? <>
                          <p> {text(Repair_view.index_message(record))} </p>
                          {jt(record, "issue") != "index_missing"
                             ? <button
                                 className="delete-link" disabled=busy onClick={_ => remove()}>
                                 <Save_icon name="trash" />
                                 {text("Delete save list")}
                               </button>
                             : React.null}
                        </>
                      : React.null,
        <details className="technical">
          <summary> {text("Technical details")} </summary>
          <code> {text(jt(record, "key"))} </code>
          {jt(record, "error") != "" ? <pre> {text(jt(record, "error"))} </pre> : React.null}
          {each(ja(jf(record, "claims")), claim =>
             <pre>
               {text(
                  Save_browser.run_name(view.catalog, claim)
                  ++ " · "
                  ++ jt(claim, "runId")
                  ++ " · "
                  ++ jt(claim, "mission")
                  ++ " · "
                  ++ Save_browser.save_time(Save_json.n(claim, "savedAt")),
                )}
             </pre>
           )}
        </details>,
      |],
    );
  };
  <div className="repair-style">
    <Dialog_window
      title dialog className="repair-window" width="540px" tall=reviewing canDismiss={!busy}>
      <div className="repair-scroll">
        {reviewing
           ? <>
               <p className="intro" role="status">
                 {text(
                    busy && Array.length(ja(jf(inventory, "records"))) == 0
                      ? "Checking retained saves…" : model.summary,
                  )}
               </p>
               {Array.length(ja(jf(inventory, "errors"))) > 0
                  ? {
                    <button className="review" disabled=busy onClick={_ => readAgain()}>
                      {text("Try reading again")}
                    </button>;
                  }
                  : React.null}
               {Array.length(model.attention) > 0
                  ? <>
                      <p className="section-label"> {text("Needs attention")} </p>
                      {each(model.attention, record => row(record, true))}
                    </>
                  : React.null}
               {Array.length(model.campaigns) > 0
                  ? <p className="section-label">
                      {text(Array.length(model.attention) > 0 ? "Also retained" : "Saved games")}
                    </p>
                  : React.null}
               {each(
                  model.campaigns,
                  (campaign: Repair_view.group) => {
                    let props = ReactDOM.domProps(~className="campaign", ());
                    dataRepairRun(props, Utf8.to_js(campaign.id));
                    ReactDOM.createElement(
                      "details",
                      ~props,
                      [|
                        <summary>
                          <span>
                            {text(campaign.name)}
                            <small>
                              {text(
                                 string_of_int(
                                   Array.length(
                                     filter(r => jt(r, "kind") == "turn", campaign.records),
                                   ),
                                 )
                                 ++ (
                                   Array.length(
                                     filter(r => jt(r, "kind") == "turn", campaign.records),
                                   )
                                   == 1
                                     ? " turn save" : " turn saves"
                                 )
                                 ++ (
                                   Array.exists(r => jt(r, "kind") == "flash", campaign.records)
                                     ? " · Flash save" : ""
                                 ),
                               )}
                            </small>
                          </span>
                        </summary>,
                        each(
                          filter(r => Save_json.get(r, "issue") == None, campaign.records),
                          record =>
                          row(record, false)
                        ),
                        model.ready
                          ? React.null
                          : <button
                              className="delete-link"
                              disabled=busy
                              onClick={_ =>
                                ask(
                                  target("campaign", "runId", campaign.id),
                                  Repair_view.campaign_confirmation(
                                    view.catalog,
                                    inventory,
                                    campaign.id,
                                  ),
                                )
                              }>
                              <Save_icon name="trash" />
                              {text("Delete campaign saves")}
                            </button>,
                      |],
                    );
                  },
                )}
             </>
           : <>
               <p>
                 {text(
                    kind == "read"
                      ? "The browser couldn’t read saved progress. We don’t know whether a flash save is available."
                      : kind == "full"
                          ? "There isn’t enough space to finish updating saved games. Delete saves you no longer need, then finish the update."
                          : "The browser refused access to saved data. Allow storage in your browser’s site settings, or play without saving.",
                  )}
               </p>
               {kind == "read"
                  ? <>
                      <p className="status-note">
                        {text("Saved data has been left untouched. Automatic saving is paused.")}
                      </p>
                      <button className="primary" disabled=busy onClick={_ => retryRead()}>
                        {text("Try reading again")}
                      </button>
                    </>
                  : React.null}
               <button className="review" disabled=busy onClick={_ => openReview()}>
                 {text("Review saved games")}
               </button>
               <button
                 className="play"
                 disabled=busy
                 onClick={_ => {
                   close(dialog);
                   onNewGame();
                 }}>
                 {text("New game without saving")}
               </button>
             </>}
        {error != "" ? <p className="damage" role="alert"> {text(error)} </p> : React.null}
        <details className="technical failure">
          <summary> {text("Technical details")} </summary>
          <pre>
            {text(Option.fold(~none="", ~some=p => jt(p, "detail"), view.load_problem))}
          </pre>
          {each(ja(jf(inventory, "errors")), issue =>
             <pre> {text(Save_json.string(issue))} </pre>
           )}
        </details>
      </div>
      {reviewing
         ? <footer className="repair-actions">
             {model.hint != "" ? <p className="hint"> {text(model.hint)} </p> : React.null}
             <button
               className="primary" disabled={busy || !model.can_continue} onClick={_ => resume()}>
               {text(model.continuation)}
             </button>
             {model.ready
                ? <button
                    className="review"
                    disabled=busy
                    onClick={_ => {
                      close(dialog);
                      onManage();
                    }}>
                    {text("Saved games")}
                  </button>
                : React.null}
             <button
               className="play"
               disabled=busy
               onClick={_ => {
                 close(dialog);
                 onNewGame();
               }}>
               {text(model.ready ? "New game" : "New game without saving")}
             </button>
             <div className="support">
               <button onClick={_ => Session.commands(session).download_debug_report()}>
                 {text("Download debug report")}
               </button>
               <button
                 className="delete-link" disabled={busy || model.ready} onClick={_ => reset()}>
                 <Save_icon name="trash" />
                 {text("Delete all retained saves")}
               </button>
             </div>
           </footer>
         : React.null}
    </Dialog_window>
    <Dialog_window
      title={Option.fold(
        ~none="Confirm saved-game action",
        ~some=((_, message)) => jt(message, "title"),
        pending,
      )}
      dialog=confirmation
      width="400px"
      canDismiss={!busy}>
      {option(pending, ((_, message)) =>
         <>
           {jt(message, "subject") != ""
              ? <p className="subject"> {text(jt(message, "subject"))} </p> : React.null}
           <p> {text(jt(message, "copy"))} </p>
           {jt(message, "notice") != ""
              ? <p className={"notice" ++ (jb(message, "warning") ? " warning" : "")}>
                  {text(jt(message, "notice"))}
                </p>
              : React.null}
           {jb(message, "destructive")
              ? <p className="irreversible"> {text("This cannot be undone.")} </p> : React.null}
           {error != "" ? <p className="damage" role="alert"> {text(error)} </p> : React.null}
           <button
             className={"confirm" ++ (jb(message, "destructive") ? " destructive" : "")}
             disabled={busy || !armed}
             onClick={_ => confirmAction()}>
             {text(jt(message, "action"))}
           </button>
           <button className="cancel" disabled=busy onClick={_ => close(confirmation)}>
             {text("Cancel")}
           </button>
         </>
       )}
    </Dialog_window>
  </div>;
};
