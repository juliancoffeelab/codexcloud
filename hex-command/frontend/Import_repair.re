open Ui;
let jt = Save_json.text;
let jf = Save_json.field;
let ja = Save_json.array;
let jb = Save_json.b;
[@mel.set] external dataRepairKey: (ReactDOM.domProps, string) => unit = "data-repair-key";
[@mel.set] external dataRepairRun: (ReactDOM.domProps, string) => unit = "data-repair-run";
[@react.component]
let make = (~session, ~view: Session.model, ~onNewGame, ~onManage, ~openKey=0) => {
  let (opened, setOpened) = React.useState(() => false);
  let (confirmOpen, setConfirmOpen) = React.useState(() => false);
  let shown = React.useRef(None);
  let (reviewing, setReviewing) = React.useState(() => false);
  let (busy, setBusy) = React.useState(() => false);
  let (error, setError) = React.useState(() => {js||js});
  let (inventory, setInventory) =
    React.useState(() =>
      Save_json.obj([
        ({js|records|js}, Save_json.arr([])),
        ({js|errors|js}, Save_json.arr([])),
        ({js|authoritative|js}, Save_json.bool(false)),
        ({js|completedElsewhere|js}, Save_json.bool(false)),
      ])
    );
  let (pending, setPending) = React.useState(() => None);
  let (armed, setArmed) = React.useState(() => false);
  let timer = React.useRef(None);
  let confirmationId = React.useRef({js||js});
  let repaired = React.useRef(false);
  let content = React.useRef(Js.Nullable.null);
  let mounted = React.useRef(true);
  let generation = React.useRef(0);
  let dismissConfirmation = () => {
    setConfirmOpen(_ => false);
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
    setOpened(_ => true);
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
        setOpened(_ => false);
        shown.current = None;
      | Some(problem) when view.load_problem != shown.current =>
        if (jt(problem, {js|kind|js}) == {js|ready|js}
            && !
                 Option.fold(
                   ~none=false,
                   ~some=p => jt(p, {js|kind|js}) == {js|ready|js},
                   shown.current,
                 )) {
          dismissConfirmation();
        };
        shown.current = view.load_problem;
        if (!opened) {
          let review =
            jt(problem, {js|kind|js}) == {js|data|js}
            || jt(problem, {js|kind|js}) == {js|ready|js};
          setReviewing(_ => review);
          setOpened(_ => true);
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
  let kind = Option.fold(~none={js||js}, ~some=p => jt(p, {js|kind|js}), view.load_problem);
  let title =
    reviewing
      ? model.ready ? {js|Saved games are ready|js} : {js|Review saved data|js}
      : kind == {js|read|js}
          ? {js|Couldn’t read saved progress|js}
          : kind == {js|full|js}
              ? {js|Not enough space|js}
              : kind == {js|blocked|js}
                  ? {js|Storage access is blocked|js} : {js|Review saved data|js};
  let ask = (target, message) =>
    if (!busy) {
      repaired.current = false;
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
      setConfirmOpen(_ => true);
    };
  let target = (kind, key, value) =>
    Save_json.obj([({js|kind|js}, Save_json.str(kind)), (key, Save_json.str(value))]);
  let resume = () => {
    let flash = Option.bind(model.flash, row => Save_json.get(row, {js|entry|js}));
    let message =
      Repair_view.confirmation(
        flash == None ? {js|Finish and choose new game?|js} : {js|Restore saved game?|js},
        Option.fold(
          ~none={js||js},
          ~some=
            e => {
              let js_text_part_0 = Save_browser.run_name(view.catalog, e);
              let js_text_part_2 = Save_browser.save_time(Save_json.n(e, {js|savedAt|js}));
              {j|$(js_text_part_0) · $(js_text_part_2)|j};
            },
          flash,
        ),
        flash == None
          ? {js|There is no flash save to restore. This finishes the update and opens New game, replacing any current unsaved play.|js}
          : {js|Your current unsaved play will be replaced by this flash save.|js},
        flash == None ? {js|Finish update|js} : {js|Restore saved game|js},
        false,
        {js||js},
        false,
      );
    ask(
      Save_json.obj([
        ({js|kind|js}, Save_json.str({js|resume|js})),
        ({js|expected|js}, Option.value(~default=Js.Json.null, flash)),
      ]),
      message,
    );
  };
  let readAgain = () =>
    ask(
      Save_json.obj([({js|kind|js}, Save_json.str({js|resume|js}))]),
      Repair_view.confirmation(
        {js|Try reading saved progress again?|js},
        {js||js},
        {js|Stored progress hasn’t been identified. If it can be read, its flash save will replace your current unsaved play. If there is no flash save, New game will open.|js},
        {js|Try reading again|js},
        false,
        {js||js},
        false,
      ),
    );
  let reset = () =>
    ask(
      Save_json.obj([({js|kind|js}, Save_json.str({js|all|js}))]),
      Repair_view.confirmation(
        {js|Delete all retained saves?|js},
        {js||js},
        {js|All retained saves will be deleted, including flash saves, unidentified saves and save-list entries.|js},
        {js|Delete all retained saves|js},
        true,
        {js||js},
        false,
      ),
    );
  let retryRead = () => {
    setBusy(_ => true);
    setError(_ => {js||js});
    let _ =
      Session.retry_saved_progress(session, None)
      |> Js.Promise.then_(success => {
           if (mounted.current) {
             setBusy(_ => false);
             if (!success) {
               setError(_ =>
                 {js|Saved progress still couldn’t be read. Stored data has not been replaced.|js}
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
      setError(_ => {js||js});
      let operation =
        if (jt(target, {js|kind|js}) == {js|resume|js}) {
          let expected =
            switch (Save_json.get(target, {js|expected|js})) {
            | None => None
            | Some(value) => Some(value == Js.Json.null ? None : Some(value))
            };
          Session.retry_saved_progress(session, expected)
          |> Js.Promise.then_(success =>
               if (success) {
                 setOpened(_ => false);
                 setConfirmOpen(_ => false);
                 Js.Promise.resolve();
               } else {
                 refresh();
               }
             );
        } else {
          Session.repair_saved_data(session, target)
          |> Js.Promise.then_(() => {
               repaired.current = true;
               refresh();
             });
        };
      let _ =
        operation
        |> Js.Promise.then_(() => {
             if (mounted.current) {
               setBusy(_ => false);
               setConfirmOpen(_ => false);
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
  let confirmation = (id, trigger) =>
    <Dialog_window
      trigger
      onCloseAutoFocus={event =>
        if (repaired.current) {
          Modal.preventDefault(event);
          Option.iter(
            node => Option.iter(focus, Js.Nullable.toOption(queryWithin(node, {js|h2|js}))),
            Js.Nullable.toOption(content.current),
          );
        }
      }
      title={Option.fold(
        ~none={js|Confirm saved-game action|js},
        ~some=((_, message)) => jt(message, {js|title|js}),
        pending,
      )}
      className={js|repair-style|js}
      opened={confirmOpen && confirmationId.current == id}
      onOpenChange={value => {
        if (value) {
          confirmationId.current = id;
        };
        setConfirmOpen(_ => value);
      }}
      alert=true
      width={js|400px|js}
      canDismiss={!busy}>
      {option(pending, ((_, message)) =>
         <>
           {jt(message, {js|subject|js}) != {js||js}
              ? <p className={js|subject|js}> {text(jt(message, {js|subject|js}))} </p>
              : React.null}
           <p> {text(jt(message, {js|copy|js}))} </p>
           {jt(message, {js|notice|js}) != {js||js}
              ? <p
                  className={
                              let js_text_part_1 =
                                jb(message, {js|warning|js}) ? {js| warning|js} : {js||js};
                              {j|notice$(js_text_part_1)|j};
                            }>
                  {text(jt(message, {js|notice|js}))}
                </p>
              : React.null}
           {jb(message, {js|destructive|js})
              ? <p className={js|irreversible|js}> {text({js|This cannot be undone.|js})} </p>
              : React.null}
           {error != {js||js}
              ? <p className={js|damage|js} role={js|alert|js}> {text(error)} </p> : React.null}
           <button
             className={
                         let js_text_part_1 =
                           jb(message, {js|destructive|js}) ? {js| destructive|js} : {js||js};
                         {j|confirm$(js_text_part_1)|j};
                       }
             disabled={busy || !armed}
             onClick={_ => confirmAction()}>
             {text(jt(message, {js|action|js}))}
           </button>
           <Radix.AlertCancel asChild=true>
             <button
               className={js|cancel|js} disabled=busy onClick={_ => setConfirmOpen(_ => false)}>
               {text({js|Cancel|js})}
             </button>
           </Radix.AlertCancel>
         </>
       )}
    </Dialog_window>;
  let row = (record, attention) => {
    let entry = jf(record, {js|entry|js});
    let name = Repair_view.record_name(view.catalog, record);
    let unknown =
      jt(record, {js|kind|js}) == {js|flash|js} && jt(record, {js|ownership|js}) != {js|known|js};
    let props =
      ReactDOM.domProps(
        ~className={
          let js_text_part_1 = attention ? {js| issue|js} : {js||js};
          let js_text_part_2 = attention && unknown ? {js| warning|js} : {js||js};
          {j|record$(js_text_part_1)$(js_text_part_2)|j};
        },
        (),
      );
    dataRepairKey(props, jt(record, {js|key|js}));
    let remove = () =>
      ask(
        target({js|record|js}, {js|key|js}, jt(record, {js|key|js})),
        Repair_view.record_confirmation(view.catalog, record),
      );
    ReactDOM.createElement(
      {js|div|js},
      ~props,
      [|
        <div className={js|record-top|js}>
          <div className={js|copy|js}>
            {attention && entry != Js.Json.null
               ? <p className={js|campaign-context|js}>
                   {text(Save_browser.run_name(view.catalog, entry))}
                 </p>
               : React.null}
            {let entry = jf(record, {js|entry|js});
             let (number, chapter) = Save_browser.chapter(view.catalog, entry);
             jt(entry, {js|kind|js}) == {js|turn|js} && number > 0
               ? <span className={js|turn-name|js}>
                   <span className={js|numeral|js}> {text(Save_browser.roman(number))} </span>
                   <strong>
                     {text(
                        {
                          let js_text_part_2 = Ui.number(Save_json.n(entry, {js|round|js}));
                          {j|$(chapter), $(js_text_part_2)|j};
                        },
                      )}
                   </strong>
                 </span>
               : <strong> {text(name)} </strong>}
            {entry != Js.Json.null
               ? <p className={js|time|js}>
                   {text(Save_browser.save_time(Save_json.n(entry, {js|savedAt|js})))}
                 </p>
               : React.null}
            {Save_json.get(record, {js|issue|js}) != None
               ? <p className={js|damage|js}> {text(Repair_view.issue(record))} </p> : React.null}
          </div>
          {jt(record, {js|kind|js}) != {js|index|js}
           && !unknown
           && jt(record, {js|ownership|js}) != {js|conflicting|js}
           && jt(record, {js|issue|js}) != {js|missing_snapshot|js}
             ? {
               let key = jt(record, {js|key|js});
               confirmation(
                 {j|record-top:$(key)|j},
                 <button
                   className={js|trash|js}
                   disabled=busy
                   ariaLabel={j|Delete $(name)|j}
                   onClick={_ => remove()}>
                   <Save_icon name={js|trash|js} />
                 </button>,
               );
             }
             : React.null}
        </div>,
        unknown
          ? <>
              <p>
                {text(
                   {js|Deleting a campaign will leave this flash save untouched. We strongly recommend deleting it separately.|js},
                 )}
              </p>
              {let key = jt(record, {js|key|js});
               confirmation(
                 {j|record-flash:$(key)|j},
                 <button className={js|delete-link|js} disabled=busy onClick={_ => remove()}>
                   <Save_icon name={js|trash|js} />
                   {text({js|Delete flash save|js})}
                 </button>,
               )}
            </>
          : jt(record, {js|issue|js}) == {js|missing_snapshot|js}
              ? {
                let key = jt(record, {js|key|js});
                confirmation(
                  {j|record-missing:$(key)|j},
                  <button className={js|delete-link|js} disabled=busy onClick={_ => remove()}>
                    <Save_icon name={js|trash|js} />
                    {text({js|Remove missing turn from list|js})}
                  </button>,
                );
              }
              : jt(record, {js|ownership|js}) == {js|conflicting|js}
                  ? <>
                      <p>
                        {text(
                           {
                             let js_text_part_1 =
                               string_of_int(
                                 List.length(
                                   List.sort_uniq(
                                     Stdlib.compare,
                                     Array.to_list(ja(jf(record, {js|claims|js})))
                                     |> List.map(e => jt(e, {js|runId|js})),
                                   ),
                                 ),
                               );
                             let js_text_part_3 =
                               Js.Array.join(
                                 ~sep={js| and |js},
                                 Array.of_list(
                                   List.sort_uniq(
                                     Stdlib.compare,
                                     Array.to_list(ja(jf(record, {js|claims|js})))
                                     |> List.map(Save_browser.run_name(view.catalog)),
                                   ),
                                 ),
                               );
                             {j|This turn is linked to $(js_text_part_1) different playthroughs: $(js_text_part_3). Deleting a campaign will leave this turn untouched.|j};
                           },
                         )}
                      </p>
                      {let key = jt(record, {js|key|js});
                       confirmation(
                         {j|record-unidentified:$(key)|j},
                         <button
                           className={js|delete-link|js} disabled=busy onClick={_ => remove()}>
                           <Save_icon name={js|trash|js} />
                           {text({js|Delete unidentified turn save|js})}
                         </button>,
                       )}
                    </>
                  : jt(record, {js|kind|js}) == {js|index|js}
                      ? <>
                          <p> {text(Repair_view.index_message(record))} </p>
                          {jt(record, {js|issue|js}) != {js|index_missing|js}
                             ? {
                               let key = jt(record, {js|key|js});
                               confirmation(
                                 {j|record-index:$(key)|j},
                                 <button
                                   className={js|delete-link|js}
                                   disabled=busy
                                   onClick={_ => remove()}>
                                   <Save_icon name={js|trash|js} />
                                   {text({js|Delete save list|js})}
                                 </button>,
                               );
                             }
                             : React.null}
                        </>
                      : React.null,
        <details className={js|technical|js}>
          <summary> {text({js|Technical details|js})} </summary>
          <code> {text(jt(record, {js|key|js}))} </code>
          {jt(record, {js|error|js}) != {js||js}
             ? <pre> {text(jt(record, {js|error|js}))} </pre> : React.null}
          {each(ja(jf(record, {js|claims|js})), claim =>
             <pre>
               {text(
                  {
                    let js_text_part_0 = Save_browser.run_name(view.catalog, claim);
                    let js_text_part_2 = jt(claim, {js|runId|js});
                    let js_text_part_4 = jt(claim, {js|mission|js});
                    let js_text_part_6 =
                      Save_browser.save_time(Save_json.n(claim, {js|savedAt|js}));
                    {j|$(js_text_part_0) · $(js_text_part_2) · $(js_text_part_4) · $(js_text_part_6)|j};
                  },
                )}
             </pre>
           )}
        </details>,
      |],
    );
  };
  <div className={js|repair-style|js}>
    <Dialog_window
      onCloseAutoFocus={event => {
        Modal.preventDefault(event);
        Option.iter(focus, Js.Nullable.toOption(byId({js|settings-button|js})));
      }}
      title
      contentRef=content
      opened
      onOpenChange={value => setOpened(_ => value)}
      className={js|repair-style repair-window|js}
      width={js|540px|js}
      tall=reviewing
      canDismiss={!busy}>
      <div className={js|repair-scroll|js}>
        {reviewing
           ? <>
               <p className={js|intro|js} role={js|status|js}>
                 {text(
                    busy && Array.length(ja(jf(inventory, {js|records|js}))) == 0
                      ? {js|Checking retained saves…|js} : model.summary,
                  )}
               </p>
               {Array.length(ja(jf(inventory, {js|errors|js}))) > 0
                  ? {
                    confirmation(
                      {js|read|js},
                      <button className={js|review|js} disabled=busy onClick={_ => readAgain()}>
                        {text({js|Try reading again|js})}
                      </button>,
                    );
                  }
                  : React.null}
               {Array.length(model.attention) > 0
                  ? <>
                      <p className={js|section-label|js}> {text({js|Needs attention|js})} </p>
                      {each(model.attention, record => row(record, true))}
                    </>
                  : React.null}
               {Array.length(model.campaigns) > 0
                  ? <p className={js|section-label|js}>
                      {text(
                         Array.length(model.attention) > 0
                           ? {js|Also retained|js} : {js|Saved games|js},
                       )}
                    </p>
                  : React.null}
               {each(
                  model.campaigns,
                  (campaign: Repair_view.group) => {
                    let props = ReactDOM.domProps(~className={js|campaign|js}, ());
                    dataRepairRun(props, campaign.id);
                    ReactDOM.createElement(
                      {js|details|js},
                      ~props,
                      [|
                        <summary>
                          <span>
                            {text(campaign.name)}
                            <small>
                              {text(
                                 {
                                   let js_text_part_0 =
                                     string_of_int(
                                       Array.length(
                                         filter(
                                           r => jt(r, {js|kind|js}) == {js|turn|js},
                                           campaign.records,
                                         ),
                                       ),
                                     );
                                   let js_text_part_1 =
                                     Array.length(
                                       filter(
                                         r => jt(r, {js|kind|js}) == {js|turn|js},
                                         campaign.records,
                                       ),
                                     )
                                     == 1
                                       ? {js| turn save|js} : {js| turn saves|js};
                                   let js_text_part_2 =
                                     Array.exists(
                                       r => jt(r, {js|kind|js}) == {js|flash|js},
                                       campaign.records,
                                     )
                                       ? {js| · Flash save|js} : {js||js};
                                   {j|$(js_text_part_0)$(js_text_part_1)$(js_text_part_2)|j};
                                 },
                               )}
                            </small>
                          </span>
                        </summary>,
                        each(
                          filter(r => Save_json.get(r, {js|issue|js}) == None, campaign.records),
                          record =>
                          row(record, false)
                        ),
                        model.ready
                          ? React.null
                          : {
                            let id = campaign.id;
                            confirmation(
                              {j|campaign:$(id)|j},
                              <button
                                className={js|delete-link|js}
                                disabled=busy
                                onClick={_ =>
                                  ask(
                                    target({js|campaign|js}, {js|runId|js}, campaign.id),
                                    Repair_view.campaign_confirmation(
                                      view.catalog,
                                      inventory,
                                      campaign.id,
                                    ),
                                  )
                                }>
                                <Save_icon name={js|trash|js} />
                                {text({js|Delete campaign saves|js})}
                              </button>,
                            );
                          },
                      |],
                    );
                  },
                )}
             </>
           : <>
               <p>
                 {text(
                    kind == {js|read|js}
                      ? {js|The browser couldn’t read saved progress. We don’t know whether a flash save is available.|js}
                      : kind == {js|full|js}
                          ? {js|There isn’t enough space to finish updating saved games. Delete saves you no longer need, then finish the update.|js}
                          : {js|The browser refused access to saved data. Allow storage in your browser’s site settings, or play without saving.|js},
                  )}
               </p>
               {kind == {js|read|js}
                  ? <>
                      <p className={js|status-note|js}>
                        {text(
                           {js|Saved data has been left untouched. Automatic saving is paused.|js},
                         )}
                      </p>
                      <button className={js|primary|js} disabled=busy onClick={_ => retryRead()}>
                        {text({js|Try reading again|js})}
                      </button>
                    </>
                  : React.null}
               <button className={js|review|js} disabled=busy onClick={_ => openReview()}>
                 {text({js|Review saved games|js})}
               </button>
               <button
                 className={js|play|js}
                 disabled=busy
                 onClick={_ => {
                   setOpened(_ => false);
                   onNewGame();
                 }}>
                 {text({js|New game without saving|js})}
               </button>
             </>}
        {error != {js||js}
           ? <p className={js|damage|js} role={js|alert|js}> {text(error)} </p> : React.null}
        <details className={js|technical failure|js}>
          <summary> {text({js|Technical details|js})} </summary>
          <pre>
            {text(
               Option.fold(~none={js||js}, ~some=p => jt(p, {js|detail|js}), view.load_problem),
             )}
          </pre>
          {each(ja(jf(inventory, {js|errors|js})), issue =>
             <pre> {text(Save_json.string(issue))} </pre>
           )}
        </details>
      </div>
      {reviewing
         ? <footer className={js|repair-actions|js}>
             {model.hint != {js||js}
                ? <p className={js|hint|js}> {text(model.hint)} </p> : React.null}
             {confirmation(
                {js|resume|js},
                <button
                  className={js|primary|js}
                  disabled={busy || !model.can_continue}
                  onClick={_ => resume()}>
                  {text(model.continuation)}
                </button>,
              )}
             {model.ready
                ? <button
                    className={js|review|js}
                    disabled=busy
                    onClick={_ => {
                      setOpened(_ => false);
                      onManage();
                    }}>
                    {text({js|Saved games|js})}
                  </button>
                : React.null}
             <button
               className={js|play|js}
               disabled=busy
               onClick={_ => {
                 setOpened(_ => false);
                 onNewGame();
               }}>
               {text(model.ready ? {js|New game|js} : {js|New game without saving|js})}
             </button>
             <div className={js|support|js}>
               <button onClick={_ => Session.commands(session).download_debug_report()}>
                 {text({js|Download debug report|js})}
               </button>
               {confirmation(
                  {js|all|js},
                  <button
                    className={js|delete-link|js}
                    disabled={busy || model.ready}
                    onClick={_ => reset()}>
                    <Save_icon name={js|trash|js} />
                    {text({js|Delete all retained saves|js})}
                  </button>,
                )}
             </div>
           </footer>
         : React.null}
    </Dialog_window>
  </div>;
};
