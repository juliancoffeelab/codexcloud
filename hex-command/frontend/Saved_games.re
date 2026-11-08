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
        <dd className={name == "gold" ? "gold" : ""}> {text(value)} </dd>
      </div>;
    let ratio = (v, a, b) => number(jn(v, a)) ++ " / " ++ number(jn(v, b));
    let goals = jf(stats, "objectives");
    <dl className="summary-stats">
      {stat("gold", "Gold", number(jn(stats, "gold")))}
      {stat("villages", "Villages", ratio(jf(stats, "villages"), "owned", "total"))}
      {stat(
         "objectives",
         "Objectives",
         goals == Js.Json.null ? "—" : ratio(goals, "completed", "total"),
       )}
      {each(Save_json.array(jf(stats, "units")), unit =>
         stat("units", "Units lvl. " ++ number(jn(unit, "level")), number(jn(unit, "count")))
       )}
    </dl>;
  };
};
[@react.component]
let make = (~session, ~view: Session.model, ~onLoad, ~refreshKey=0) => {
  let (entries, setEntries) = React.useState(() => [||]);
  let (flash, setFlash) = React.useState(() => None);
  let (failure, setFailure) = React.useState(() => "");
  let (busy, setBusy) = React.useState(() => false);
  let (expanded, setExpanded) = React.useState(() => []);
  let (limits, setLimits) = React.useState(() => []);
  let (stats, setStats) = React.useState(() => []);
  let (errors, setErrors) = React.useState(() => []);
  let (pending, setPending) = React.useState(() => None);
  let confirmation = React.useRef(Js.Nullable.null);
  let (armed, setArmed) = React.useState(() => false);
  let timer = React.useRef(None);
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
      close(confirmation);
      setPending(_ => None);
      setArmed(_ => false);
      Option.iter(Js.Global.clearTimeout, timer.current);
      timer.current = None;
      generation.current = generation.current + 1;
      setEntries(_ => [||]);
      setFlash(_ => None);
      setExpanded(_ => []);
      setLimits(_ => []);
      setStats(_ => []);
      setErrors(_ => []);
      inflight.current = [];
      setFailure(_ => "");
      let _ =
        refresh()
        |> Js.Promise.catch(_ => {
             if (alive.current) {
               setFailure(_ =>
                 "Saved games could not be read. Download a debug report from Settings so we can investigate."
               );
             };
             Js.Promise.resolve();
           });
      None;
    },
    [|refreshKey|],
  );
  let inspect = entry => {
    let id = jt(entry, "id");
    let opening = !List.mem(id, expanded);
    setExpanded(old => opening ? [id, ...old] : List.filter(x => x != id, old));
    if (opening && !List.mem_assoc(id, stats) && !List.mem(id, inflight.current)) {
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
               setErrors(old => [(id, "This saved battle could not be read."), ...old]);
             };
             inflight.current = List.filter(x => x != id, inflight.current);
             Js.Promise.resolve();
           });
      ();
    };
  };
  let ask = (kind, entry) => {
    setPending(_ => Some((kind, entry)));
    setFailure(_ => "");
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
    show(confirmation);
    Js.Nullable.toOption(confirmation.current)
    |> Option.iter(node =>
         Js.Nullable.toOption(queryWithin(node, ".cancel")) |> Option.iter(focus)
       );
  };
  let confirm = () =>
    switch (pending) {
    | Some((kind, entry)) when !busy && armed =>
      setBusy(_ => true);
      let operation =
        switch (kind) {
        | "load" => Session.restore_progress(session, jt(entry, "id"))
        | "turn" => Session.delete_saved_progress(session, jt(entry, "id"))
        | _ => Session.delete_saved_run(session, jt(entry, "runId"))
        };
      let _ =
        operation
        |> Js.Promise.then_(() => refresh())
        |> Js.Promise.then_(() => {
             close(confirmation);
             if (kind == "load") {
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
                 "This operation could not be completed. Download a debug report if the problem continues."
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
  <div className="saved-progress-style">
    <section className="saved-games-browser" ariaLabel="Saved games" ariaBusy=busy>
      {failure != "" && pending == None
         ? <p role="alert" className="save-alert"> {text(failure)} </p> : React.null}
      {Option.fold(
         ~none=false,
         ~some=(f: Save_error.t) => f.kind == Save_error.Full,
         view.save_failure,
       )
         ? <p className="space-warning" role="status">
             <strong> {text("Latest progress hasn’t been saved")} </strong>
             {text("Delete campaign saves or individual turns to free space.")}
           </p>
         : React.null}
      {each(
         games,
         (game: Save_browser.game) => {
           let history = filter(entry => jt(entry, "runId") == game.id, entries);
           let limit = Option.value(~default=30, List.assoc_opt(game.id, limits));
           let props = ReactDOM.domProps(~className="campaign", ~open_=true, ());
           dataKey(props, Utf8.to_js(game.id));
           dataRun(props, Utf8.to_js(game.id));
           ReactDOM.createElement(
             "details",
             ~props,
             [|
               <summary className="campaign-toggle">
                 <span className="campaign-copy">
                   <span className="campaign-name"> {text(runName(game.latest))} </span>
                   <span className="campaign-time">
                     {text(
                        Save_browser.first_saved(game.earliest)
                        ++ " · Last saved "
                        ++ Save_browser.save_time(jn(game.latest, "savedAt")),
                      )}
                   </span>
                 </span>
                 <svg className="chevron" viewBox="0 0 16 16" ariaHidden=true>
                   <path d="m6 3 5 5-5 5" />
                 </svg>
               </summary>,
               <div className="history">
                 {Array.length(history) == 0
                    ? <p className="empty-history"> {text("No earlier saves.")} </p>
                    : each(
                        Array.sub(history, 0, min(limit, Array.length(history))),
                        entry => {
                          let id = jt(entry, "id");
                          let open_ = List.mem(id, expanded);
                          let props =
                            ReactDOM.domProps(
                              ~className="save" ++ (open_ ? " expanded" : ""),
                              (),
                            );
                          dataKey(props, Utf8.to_js(id));
                          dataSave(props, Utf8.to_js(id));
                          ReactDOM.createElement(
                            "div",
                            ~props,
                            [|
                              <div className="save-row">
                                <div className="save-copy">
                                  <span className="turn"> {text(turnName(entry))} </span>
                                  <span className="save-time">
                                    {text(
                                       Save_browser.save_time(jn(entry, "savedAt"))
                                       ++ (jb(entry, "manual") ? " · Manual" : ""),
                                     )}
                                  </span>
                                </div>
                                <button
                                  className="row-icon load"
                                  disabled=busy
                                  ariaLabel={Utf8.to_js("Load " ++ turnName(entry))}
                                  title="Load saved turn"
                                  onClick={_ => ask("load", entry)}>
                                  <Save_icon name="load" />
                                </button>
                                <button
                                  className="row-icon inspect"
                                  disabled=busy
                                  ariaLabel={Utf8.to_js("Details for " ++ turnName(entry))}
                                  ariaExpanded=open_
                                  ariaControls={Utf8.to_js("summary-" ++ id)}
                                  title="Saved battle details"
                                  onClick={_ => inspect(entry)}>
                                  <Save_icon name="details" />
                                </button>
                              </div>,
                              open_
                                ? <div
                                    className="save-summary"
                                    id={Utf8.to_js("summary-" ++ id)}
                                    ariaLabel={Utf8.to_js(
                                      "Battle summary for " ++ turnName(entry),
                                    )}>
                                    {switch (List.assoc_opt(id, stats)) {
                                     | Some(stats) => <Summary stats />
                                     | None =>
                                       switch (List.assoc_opt(id, errors)) {
                                       | Some(error) =>
                                         <p className="summary-error" role="alert">
                                           {text(error)}
                                         </p>
                                       | None =>
                                         <p className="reading" role="status">
                                           {text("Reading saved battle…")}
                                         </p>
                                       }
                                     }}
                                    <div className="summary-footer">
                                      <button
                                        className="delete-save"
                                        disabled=busy
                                        onClick={_ => ask("turn", entry)}>
                                        <Save_icon name="trash" />
                                        {text("Delete save")}
                                      </button>
                                    </div>
                                  </div>
                                : React.null,
                            |],
                          );
                        },
                      )}
                 {Array.length(history) > limit
                    ? <button
                        className="show-more"
                        onClick={_ =>
                          setLimits(old =>
                            [(game.id, limit + 30), ...List.remove_assoc(game.id, old)]
                          )
                        }>
                        {text("Show more")}
                      </button>
                    : React.null}
                 <div className="campaign-management">
                   <button
                     className="delete-campaign"
                     disabled=busy
                     onClick={_ => {
                       let entry = Save_json.copy(game.latest);
                       Save_json.set(
                         entry,
                         "count",
                         Save_json.num(
                           float_of_int(
                             Array.length(history)
                             + (
                               Option.fold(
                                 ~none=false,
                                 ~some=e => jt(e, "runId") == game.id,
                                 flash,
                               )
                                 ? 1 : 0
                             ),
                           ),
                         ),
                       );
                       Save_json.set(
                         entry,
                         "started",
                         Save_json.str(Save_browser.first_saved(game.earliest)),
                       );
                       ask("campaign", entry);
                     }}>
                     <Save_icon name="trash" />
                     {text("Delete all campaign saves")}
                   </button>
                 </div>
               </div>,
             |],
           );
         },
       )}
      {Array.length(games) == 0 && failure == ""
         ? <p className="empty-history"> {text("No saved games.")} </p> : React.null}
    </section>
    <Dialog_window
      title={
        switch (pending) {
        | Some(("load", _)) => "Load saved turn?"
        | Some(("turn", _)) => "Delete save?"
        | _ => "Delete all campaign saves?"
        }
      }
      dialog=confirmation
      width="380px"
      canDismiss={!busy}>
      {option(pending, ((kind, entry)) =>
         <>
           <p className="confirmation-name"> {text(runName(entry))} </p>
           {switch (kind) {
            | "campaign" =>
              <>
                <p className="confirmation-time"> {text(jt(entry, "started"))} </p>
                {view.active_run == Some(jt(entry, "runId"))
                   ? <div className="current-warning">
                       <strong>
                         <svg viewBox="0 0 24 24" ariaHidden=true>
                           <path d="m12 3 10 18H2L12 3zM12 9v5M12 17v1" />
                         </svg>
                         <span> {text("This is the game you’re currently playing.")} </span>
                       </strong>
                       <p> {text("Deleting its saves will end this game and open New game.")} </p>
                     </div>
                   : React.null}
                <p className="confirmation-copy">
                  {text(
                     "All "
                     ++ number(jn(entry, "count"))
                     ++ " saves will be deleted, including chapter starts. This cannot be undone.",
                   )}
                </p>
              </>
            | _ =>
              <>
                <p className="confirmation-turn"> {text(turnName(entry))} </p>
                <p className="confirmation-time">
                  {text(Save_browser.save_time(jn(entry, "savedAt")))}
                </p>
                <p className="confirmation-copy">
                  {text(
                     kind == "load"
                       ? "Your current position will be replaced."
                       : "This save will be deleted. This cannot be undone.",
                   )}
                </p>
              </>
            }}
           {failure != ""
              ? <p className="save-alert" role="alert"> {text(failure)} </p> : React.null}
           <button
             className={"confirm-action" ++ (kind == "load" ? "" : " destructive")}
             disabled={busy || !armed}
             onClick={_ => confirm()}>
             {text(
                kind == "load"
                  ? "Load turn" : kind == "turn" ? "Delete save" : "Delete all campaign saves",
              )}
           </button>
           <button
             className="cancel"
             disabled=busy
             onClick={_ => {
               close(confirmation);
               setPending(_ => None);
             }}>
             {text("Cancel")}
           </button>
         </>
       )}
    </Dialog_window>
  </div>;
};
