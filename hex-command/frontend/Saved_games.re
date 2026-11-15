open Ui;
let jt = Save_json.text;
let jn = Save_json.n;
let jb = Save_json.b;
let jf = Save_json.field;
[@mel.set] external dataRun: (ReactDOM.domProps, string) => unit = "data-run-id";
[@mel.set] external dataSave: (ReactDOM.domProps, string) => unit = "data-save-id";
module Summary = {
  [@react.component]
  let make = (~stats) => {
    let stat = (name, label, value) =>
      <div>
        <dt> <Save_icon name /> {text(label)} </dt>
        <dd className={name == {js|gold|js} ? {js|gold|js} : {js||js}}> {text(value)} </dd>
      </div>;
    let ratio = (v, a, b) => {
      let js_text_part_0 = number(jn(v, a));
      let js_text_part_2 = number(jn(v, b));
      {j|$(js_text_part_0) / $(js_text_part_2)|j};
    };
    let goals = jf(stats, {js|objectives|js});
    <dl className={js|summary-stats|js}>
      {stat({js|gold|js}, {js|Gold|js}, number(jn(stats, {js|gold|js})))}
      {stat(
         {js|villages|js},
         {js|Villages|js},
         ratio(jf(stats, {js|villages|js}), {js|owned|js}, {js|total|js}),
       )}
      {stat(
         {js|objectives|js},
         {js|Objectives|js},
         goals == Js.Json.null ? {js|—|js} : ratio(goals, {js|completed|js}, {js|total|js}),
       )}
      {each(Save_json.array(jf(stats, {js|units|js})), unit =>
         stat(
           {js|units|js},
           {
             let js_text_part_1 = number(jn(unit, {js|level|js}));
             {j|Units lvl. $(js_text_part_1)|j};
           },
           number(jn(unit, {js|count|js})),
         )
       )}
    </dl>;
  };
};
[@react.component]
let make = (~session, ~view: Session.model, ~onLoad, ~onDeleted, ~refreshKey=0) => {
  let (entries, setEntries) = React.useState(() => [||]);
  let (flash, setFlash) = React.useState(() => None);
  let (failure, setFailure) = React.useState(() => {js||js});
  let (busy, setBusy) = React.useState(() => false);
  let (limits, setLimits) = React.useState(() => []);
  let (stats, setStats) = React.useState(() => []);
  let (errors, setErrors) = React.useState(() => []);
  let (pending, setPending) = React.useState(() => None);
  let (confirmOpen, setConfirmOpen) = React.useState(() => false);
  let (armed, setArmed) = React.useState(() => false);
  let timer = React.useRef(None);
  let deleted = React.useRef(false);
  let alive = React.useRef(true);
  let generation = React.useRef(0);
  let inflight = React.useRef([]);
  let refresh = () => {
    let request = generation.current;
    Session.saved_progress(session)
    |> Js.Promise.then_(history =>
         Session.saved_flash(session)
         |> Js.Promise.then_(current => {
              if (alive.current && request == generation.current) {
                setEntries(_ => history);
                setFlash(_ => current);
              };
              Js.Promise.resolve();
            })
       );
  };
  React.useEffect1(
    () => {
      alive.current = true;
      Some(
        () => {
          Option.iter(Js.Global.clearTimeout, timer.current);
          alive.current = false;
          generation.current = generation.current + 1;
        },
      );
    },
    [|session|],
  );
  React.useEffect1(
    () => {
      setConfirmOpen(_ => false);
      setPending(_ => None);
      setArmed(_ => false);
      Option.iter(Js.Global.clearTimeout, timer.current);
      timer.current = None;
      generation.current = generation.current + 1;
      setEntries(_ => [||]);
      setFlash(_ => None);
      setLimits(_ => []);
      setStats(_ => []);
      setErrors(_ => []);
      inflight.current = [];
      setFailure(_ => {js||js});
      let _ =
        refresh()
        |> Js.Promise.catch(_ => {
             if (alive.current) {
               setFailure(_ =>
                 {js|Saved games could not be read. Download a debug report from Settings so we can investigate.|js}
               );
             };
             Js.Promise.resolve();
           });
      None;
    },
    [|refreshKey|],
  );
  let inspect = entry => {
    let id = jt(entry, {js|id|js});
    if (!List.mem_assoc(id, stats) && !List.mem(id, inflight.current)) {
      let request = generation.current;
      inflight.current = [id, ...inflight.current];
      let _ =
        Session.saved_summary(session, id)
        |> Js.Promise.then_(value => {
             if (alive.current && generation.current == request) {
               setStats(old => [(id, value), ...List.remove_assoc(id, old)]);
             };
             inflight.current = List.filter(x => x != id, inflight.current);
             Js.Promise.resolve();
           })
        |> Js.Promise.catch(_ => {
             if (alive.current && generation.current == request) {
               setErrors(old => [(id, {js|This saved battle could not be read.|js}), ...old]);
             };
             inflight.current = List.filter(x => x != id, inflight.current);
             Js.Promise.resolve();
           });
      ();
    };
  };
  let ask = (kind, entry) => {
    deleted.current = false;
    setPending(_ => Some((kind, entry)));
    setFailure(_ => {js||js});
    setArmed(_ => false);
    Option.iter(Js.Global.clearTimeout, timer.current);
    timer.current =
      Some(
        Js.Global.setTimeout(
          ~f=
            () =>
              if (alive.current) {
                setArmed(_ => true);
              },
          350,
        ),
      );
    setConfirmOpen(_ => true);
  };
  let confirm = () =>
    switch (pending) {
    | Some((kind, entry)) when !busy && armed =>
      setBusy(_ => true);
      let operation =
        switch (kind) {
        | matched_text_0 when matched_text_0 == {js|load|js} =>
          Session.restore_progress(session, jt(entry, {js|id|js}))
        | matched_text_1 when matched_text_1 == {js|turn|js} =>
          Session.delete_saved_progress(session, jt(entry, {js|id|js}))
        | _ => Session.delete_saved_run(session, jt(entry, {js|runId|js}))
        };
      let _ =
        operation
        |> Js.Promise.then_(() => {
             deleted.current = kind != {js|load|js};
             refresh();
           })
        |> Js.Promise.then_(() => {
             setConfirmOpen(_ => false);
             if (kind == {js|load|js}) {
               onLoad();
             };
             if (alive.current) {
               setBusy(_ => false);
             };
             Js.Promise.resolve();
           })
        |> Js.Promise.catch(_ => {
             if (alive.current) {
               setBusy(_ => false);
               setFailure(_ =>
                 {js|This operation could not be completed. Download a debug report if the problem continues.|js}
               );
             };
             Js.Promise.resolve();
           });
      ();
    | _ => ()
    };
  let games =
    Save_browser.games(
      Array.append(
        entries,
        switch (flash) {
        | None => [||]
        | Some(entry) => [|entry|]
        },
      ),
    );
  let turnName = Save_browser.turn_name(view.catalog);
  let runName = Save_browser.run_name(view.catalog);
  let confirmation = (kind, entry, trigger) =>
    <Dialog_window
      trigger
      onCloseAutoFocus={event =>
        if (deleted.current) {
          Modal.preventDefault(event);
          onDeleted();
        }
      }
      title={
        switch (pending) {
        | Some((matched_text_2, _)) when matched_text_2 == {js|load|js} => {js|Load saved turn?|js}
        | Some((matched_text_3, _)) when matched_text_3 == {js|turn|js} => {js|Delete save?|js}
        | _ => {js|Delete all campaign saves?|js}
        }
      }
      className={js|saved-progress-style|js}
      opened={
        confirmOpen
        && Option.fold(
             ~none=false,
             ~some=
               ((selectedKind, selectedEntry)) =>
                 selectedKind == kind && jt(selectedEntry, {js|id|js}) == jt(entry, {js|id|js}),
             pending,
           )
      }
      onOpenChange={value => setConfirmOpen(_ => value)}
      alert=true
      width={js|380px|js}
      canDismiss={!busy}>
      {option(pending, ((kind, entry)) =>
         <>
           <p className={js|confirmation-name|js}> {text(runName(entry))} </p>
           {switch (kind) {
            | matched_text_4 when matched_text_4 == {js|campaign|js} =>
              <>
                <p className={js|confirmation-time|js}> {text(jt(entry, {js|started|js}))} </p>
                {view.active_run == Some(jt(entry, {js|runId|js}))
                   ? <div className={js|current-warning|js}>
                       <strong>
                         <svg viewBox={js|0 0 24 24|js} ariaHidden=true>
                           <path d={js|m12 3 10 18H2L12 3zM12 9v5M12 17v1|js} />
                         </svg>
                         <span> {text({js|This is the game you’re currently playing.|js})} </span>
                       </strong>
                       <p>
                         {text({js|Deleting its saves will end this game and open New game.|js})}
                       </p>
                     </div>
                   : React.null}
                <p className={js|confirmation-copy|js}>
                  {text(
                     {
                       let js_text_part_1 = number(jn(entry, {js|count|js}));
                       {j|All $(js_text_part_1) saves will be deleted, including chapter starts. This cannot be undone.|j};
                     },
                   )}
                </p>
              </>
            | _ =>
              <>
                <p className={js|confirmation-turn|js}> {text(turnName(entry))} </p>
                <p className={js|confirmation-time|js}>
                  {text(Save_browser.save_time(jn(entry, {js|savedAt|js})))}
                </p>
                <p className={js|confirmation-copy|js}>
                  {text(
                     kind == {js|load|js}
                       ? {js|Your current position will be replaced.|js}
                       : {js|This save will be deleted. This cannot be undone.|js},
                   )}
                </p>
              </>
            }}
           {failure != {js||js}
              ? <p className={js|save-alert|js} role={js|alert|js}> {text(failure)} </p>
              : React.null}
           <button
             className={
                         let js_text_part_1 =
                           kind == {js|load|js} ? {js||js} : {js| destructive|js};
                         {j|confirm-action$(js_text_part_1)|j};
                       }
             disabled={busy || !armed}
             onClick={_ => confirm()}>
             {text(
                kind == {js|load|js}
                  ? {js|Load turn|js}
                  : kind == {js|turn|js} ? {js|Delete save|js} : {js|Delete all campaign saves|js},
              )}
           </button>
           <Radix.AlertCancel asChild=true>
             <button
               className={js|cancel|js}
               disabled=busy
               onClick={_ => {
                 setConfirmOpen(_ => false);
                 setPending(_ => None);
               }}>
               {text({js|Cancel|js})}
             </button>
           </Radix.AlertCancel>
         </>
       )}
    </Dialog_window>;
  <div className={js|saved-progress-style|js}>
    <section className={js|saved-games-browser|js} ariaLabel={js|Saved games|js} ariaBusy=busy>
      {failure != {js||js} && pending == None
         ? <p role={js|alert|js} className={js|save-alert|js}> {text(failure)} </p> : React.null}
      {Option.fold(
         ~none=false,
         ~some=(f: Save_error.t) => f.kind == Save_error.Full,
         view.save_failure,
       )
         ? <p className={js|space-warning|js} role={js|status|js}>
             <strong> {text({js|Latest progress hasn’t been saved|js})} </strong>
             {text({js|Delete campaign saves or individual turns to free space.|js})}
           </p>
         : React.null}
      {each(
         games,
         (game: Save_browser.game) => {
           let history = filter(entry => jt(entry, {js|runId|js}) == game.id, entries);
           let limit = Option.value(~default=30, List.assoc_opt(game.id, limits));
           let props = ReactDOM.domProps(~className={js|campaign|js}, ~open_=true, ());
           dataKey(props, game.id);
           dataRun(props, game.id);
           ReactDOM.createElement(
             {js|details|js},
             ~props,
             [|
               <summary className={js|campaign-toggle|js}>
                 <span className={js|campaign-copy|js}>
                   <span className={js|campaign-name|js}> {text(runName(game.latest))} </span>
                   <span className={js|campaign-time|js}>
                     {text(
                        {
                          let js_text_part_0 = Save_browser.first_saved(game.earliest);
                          let js_text_part_2 =
                            Save_browser.save_time(jn(game.latest, {js|savedAt|js}));
                          {j|$(js_text_part_0) · Last saved $(js_text_part_2)|j};
                        },
                      )}
                   </span>
                 </span>
                 <svg className={js|chevron|js} viewBox={js|0 0 16 16|js} ariaHidden=true>
                   <path d={js|m6 3 5 5-5 5|js} />
                 </svg>
               </summary>,
               <div className={js|history|js}>
                 {Array.length(history) == 0
                    ? <p className={js|empty-history|js}> {text({js|No earlier saves.|js})} </p>
                    : each(
                        Array.sub(history, 0, min(limit, Array.length(history))),
                        entry => {
                          let id = jt(entry, {js|id|js});
                          let props = ReactDOM.domProps(~className={js|save|js}, ());
                          dataKey(props, id);
                          dataSave(props, id);
                          <Radix.Collapsible.Root
                            key=id
                            asChild=true
                            onOpenChange={opened =>
                              if (opened) {
                                inspect(entry);
                              }
                            }>
                            {ReactDOM.createElement(
                               {js|div|js},
                               ~props,
                               [|
                                 <div className={js|save-row|js}>
                                   <div className={js|save-copy|js}>
                                     <span className={js|turn|js}>
                                       {text(turnName(entry))}
                                     </span>
                                     <span className={js|save-time|js}>
                                       {text(
                                          {
                                            let js_text_part_0 =
                                              Save_browser.save_time(jn(entry, {js|savedAt|js}));
                                            let js_text_part_1 =
                                              jb(entry, {js|manual|js})
                                                ? {js| · Manual|js} : {js||js};
                                            {j|$(js_text_part_0)$(js_text_part_1)|j};
                                          },
                                        )}
                                     </span>
                                   </div>
                                   {confirmation(
                                      {js|load|js},
                                      entry,
                                      <button
                                        className={js|row-icon load|js}
                                        disabled=busy
                                        ariaLabel={
                                                    let js_text_part_1 = turnName(entry);
                                                    {j|Load $(js_text_part_1)|j};
                                                  }
                                        title={js|Load saved turn|js}
                                        onClick={_ => ask({js|load|js}, entry)}>
                                        <Save_icon name={js|load|js} />
                                      </button>,
                                    )}
                                   <Radix.Collapsible.Trigger asChild=true>
                                     <button
                                       className={js|row-icon inspect|js}
                                       disabled=busy
                                       ariaLabel={
                                                   let js_text_part_1 = turnName(entry);
                                                   {j|Details for $(js_text_part_1)|j};
                                                 }
                                       title={js|Saved battle details|js}>
                                       <Save_icon name={js|details|js} />
                                     </button>
                                   </Radix.Collapsible.Trigger>
                                 </div>,
                                 <Radix.Collapsible.Content asChild=true>
                                   <div
                                     className={js|save-summary|js}
                                     ariaLabel={
                                                 let js_text_part_1 = turnName(entry);
                                                 {j|Battle summary for $(js_text_part_1)|j};
                                               }>
                                     {switch (List.assoc_opt(id, stats)) {
                                      | Some(stats) => <Summary stats />

                                      | None =>
                                        switch (List.assoc_opt(id, errors)) {
                                        | Some(error) =>
                                          <p className={js|summary-error|js} role={js|alert|js}>
                                            {text(error)}
                                          </p>

                                        | None =>
                                          <p className={js|reading|js} role={js|status|js}>
                                            {text({js|Reading saved battle…|js})}
                                          </p>
                                        }
                                      }}
                                     <div className={js|summary-footer|js}>
                                       {confirmation(
                                          {js|turn|js},
                                          entry,
                                          <button
                                            className={js|delete-save|js}
                                            disabled=busy
                                            onClick={_ => ask({js|turn|js}, entry)}>
                                            <Save_icon name={js|trash|js} />
                                            {text({js|Delete save|js})}
                                          </button>,
                                        )}
                                     </div>
                                   </div>
                                 </Radix.Collapsible.Content>,
                               |],
                             )}
                          </Radix.Collapsible.Root>;
                        },
                      )}
                 {Array.length(history) > limit
                    ? <button
                        className={js|show-more|js}
                        onClick={_ =>
                          setLimits(old =>
                            [(game.id, limit + 30), ...List.remove_assoc(game.id, old)]
                          )
                        }>
                        {text({js|Show more|js})}
                      </button>
                    : React.null}
                 <div className={js|campaign-management|js}>
                   {confirmation(
                      {js|campaign|js},
                      game.latest,
                      <button
                        className={js|delete-campaign|js}
                        disabled=busy
                        onClick={_ => {
                          let entry = Save_json.copy(game.latest);
                          Save_json.set(
                            entry,
                            {js|count|js},
                            Save_json.num(
                              float_of_int(
                                Array.length(history)
                                + (
                                  Option.fold(
                                    ~none=false,
                                    ~some=e => jt(e, {js|runId|js}) == game.id,
                                    flash,
                                  )
                                    ? 1 : 0
                                ),
                              ),
                            ),
                          );
                          Save_json.set(
                            entry,
                            {js|started|js},
                            Save_json.str(Save_browser.first_saved(game.earliest)),
                          );
                          ask({js|campaign|js}, entry);
                        }}>
                        <Save_icon name={js|trash|js} />
                        {text({js|Delete all campaign saves|js})}
                      </button>,
                    )}
                 </div>
               </div>,
             |],
           );
         },
       )}
      {Array.length(games) == 0 && failure == {js||js}
         ? <p className={js|empty-history|js}> {text({js|No saved games.|js})} </p> : React.null}
    </section>
  </div>;
};
