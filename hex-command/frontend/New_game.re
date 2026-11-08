open Ui;

[@mel.scope "Math"] external random: unit => float = "random";
[@mel.set] external dataSide: (ReactDOM.domProps, string) => unit = "data-side";

type choice('value) = {
  value: 'value,
  label: string,
  description: option(string),
  sprite: option(string),
  map: option(Domain.map_definition),
};
let choice = (~value, ~label, ~description=None, ~sprite=None, ~map=None, ()) => {
  value,
  label,
  description,
  sprite,
  map,
};

module MapThumbnail = {
  [@react.component]
  let make = (~map: Domain.map_definition) => {
    let size =
      Geometry.preview_dimensions(~cols=Int32.to_int(map.cols), ~rows=Int32.to_int(map.rows));
    let tiles =
      React.useMemo1(
        () => {
          let terrain = Js.Dict.empty();
          Array.iter(
            (tile: Domain.tile) =>
              Js.Dict.set(
                terrain,
                Int32.to_string(tile.col) ++ "," ++ Int32.to_string(tile.row),
                tile.terrain,
              ),
            map.terrain,
          );
          List.init(
            Int32.to_int(map.rows) * Int32.to_int(map.cols),
            index => {
              let col = index mod Int32.to_int(map.cols);
              let row = index / Int32.to_int(map.cols);
              let point = Geometry.center(~col, ~row);
              let id = string_of_int(col) ++ "," ++ string_of_int(row);
              <polygon
                key=id
                points={Geometry.hex_points(point.x, point.y)}
                fill={Geometry.preview_color(
                  Option.value(~default="plains", Js.Dict.get(terrain, id)),
                )}
                stroke="#263322"
                strokeWidth="2"
              />;
            },
          );
        },
        [|map|],
      );
    <svg
      className="setup-map-preview"
      viewBox={"0 0 " ++ number(size.width) ++ " " ++ number(size.height)}
      ariaHidden=true>
      {eachList(tiles, tile => tile)}
    </svg>;
  };
};

module ChoicePicker = {
  [@react.component]
  let make =
      (
        ~label,
        ~choices: list(choice('value)),
        ~value,
        ~onChange,
        ~disabled=false,
        ~side="player",
      ) => {
    let id = React.useId();
    let dialog = React.useRef(Js.Nullable.null);
    let trigger = React.useRef(Js.Nullable.null);
    let (expanded, setExpanded) = React.useState(() => false);
    React.useEffect0(() =>
      switch (Js.Nullable.toOption(dialog.current)) {
      | None => None
      | Some(node) =>
        let closed = _ =>
          if (!isOpen(node)) {
            setExpanded(_ => false);
            Option.iter(focus, Js.Nullable.toOption(trigger.current));
          };
        addListener(node, "close", closed);
        Some(() => removeListener(node, "close", closed));
      }
    );
    let selected = List.find_opt(choice => choice.value == value, choices);
    let openPicker = () => {
      setExpanded(_ => true);
      show(dialog);
    };
    React.useLayoutEffect1(
      () => {
        if (expanded) {
          Option.iter(
            node =>
              Option.iter(
                focus,
                Js.Nullable.toOption(queryWithin(node, "[aria-selected='true']")),
              ),
            Js.Nullable.toOption(dialog.current),
          );
        };
        None;
      },
      [|expanded|],
    );
    let choose = choice => {
      onChange(choice.value);
      close(dialog);
    };
    let navigate = event => {
      let key = React.Event.Keyboard.key(event);
      switch (Js.Nullable.toOption(dialog.current)) {
      | Some(node) when List.mem(key, ["ArrowDown", "ArrowUp", "Home", "End"]) =>
        let rows = descendants(node, "[role='option']");
        let length = Array.length(rows);
        if (length > 0) {
          React.Event.Keyboard.preventDefault(event);
          let index = ref(-1);
          Array.iteri(
            (i, row) =>
              switch (Js.Nullable.toOption(activeElement)) {
              | Some(active) when row === active => index := i
              | _ => ()
              },
            rows,
          );
          let next =
            key == "Home"
              ? 0
              : key == "End"
                  ? length - 1 : (index^ + (key == "ArrowDown" ? 1 : (-1)) + length) mod length;
          focus(rows[next]);
        };
      | _ => ()
      };
    };
    <>
      <button
        className="choice-trigger"
        type_="button"
        ref={ReactDOM.Ref.domRef(trigger)}
        ariaLabel={Utf8.to_js(label)}
        ariaHaspopup="dialog"
        ariaExpanded=expanded
        disabled
        onClick={_ => openPicker()}>
        <span> {text(Option.fold(~none="", ~some=choice => choice.label, selected))} </span>
        <svg width="16" height="16" viewBox="0 0 16 16" ariaHidden=true>
          <path d="m4 6 4 4 4-4" fill="none" stroke="currentColor" strokeWidth="2" />
        </svg>
      </button>
      <dialog
        className="choice-dialog"
        ariaLabel={Utf8.to_js(label)}
        ref={ReactDOM.Ref.domRef(dialog)}
        onClick=backdrop
        onKeyDown=navigate>
        <header className="setup-heading">
          <h2> {text(label)} </h2>
          <button className="setup-close" ariaLabel="Close picker" onClick={_ => close(dialog)}>
            {text("×")}
          </button>
        </header>
        {<div className="choice-list" role="listbox" ariaLabel={Utf8.to_js(label)}>
           {expanded
              ? eachList(
                  List.mapi((index, choice) => (index, choice), choices), ((index, choice)) =>
                  <button
                    key={string_of_int(index)}
                    className="choice-row"
                    role="option"
                    ariaLabel={Utf8.to_js(choice.label)}
                    ariaDescribedby=?{
                      Option.map(_ => id ++ "-" ++ string_of_int(index), choice.description)
                    }
                    ariaSelected={choice.value == value}
                    onClick={_ => choose(choice)}>
                    {option(choice.sprite, name =>
                       <Sprite artwork={Art.current()} name width=64. height=64. side />
                     )}
                    {option(choice.map, map => <MapThumbnail map />)}
                    <span className="choice-copy">
                      <span> {text(choice.label)} </span>
                      {option(choice.description, description =>
                         <small id={id ++ "-" ++ string_of_int(index)}>
                           {text(description)}
                         </small>
                       )}
                    </span>
                    <span className="choice-tick" ariaHidden=true>
                      {text(choice.value == value ? "✓" : "")}
                    </span>
                  </button>
                )
              : React.null}
         </div>}
      </dialog>
    </>;
  };
};

let factions = [
  choice(~value="human", ~label="Humans", ~sprite=Some("commander"), ()),
  choice(~value="woodland", ~label="Woodland", ~sprite=Some("wildkeeper"), ()),
  choice(~value="skeletons", ~label="Skeletons", ~sprite=Some("necromancer"), ()),
  choice(~value="coastal", ~label="Merfolk", ~sprite=Some("tidekeeper"), ()),
];
let title = value =>
  value == "aggressive-predictor" ? "Aggressive Predictor" : String.capitalize_ascii(value);

/** Request identities open the persistent modal without resetting its choices. */
[@react.component]
let make =
    (
      ~session: Session.t,
      ~view: Session.model,
      ~actions: Session.actions,
      ~request: option((int, Setup.mode)),
      ~onBack: option(unit => unit)=?,
    ) => {
  let dialog = React.useRef(Js.Nullable.null);
  let confirmation = React.useRef(Js.Nullable.null);
  let (opened, setOpened) = React.useState(() => false);
  React.useEffect0(() =>
    switch (Js.Nullable.toOption(dialog.current)) {
    | None => None
    | Some(node) =>
      let closed = _ =>
        if (!isOpen(node)) {
          setOpened(_ => false);
        };
      addListener(node, "close", closed);
      Some(() => removeListener(node, "close", closed));
    }
  );
  let (mode, setMode) = React.useState(() => Setup.Skirmish);
  let (mapId, setMap) = React.useState(() => "mirror");
  let (player, setPlayer) = React.useState(() => "human");
  let (opponent, setOpponent) = React.useState(() => "skeletons");
  let (campaignId, setCampaign) = React.useState(() => "lantern-road");
  let (planner, setPlanner) = React.useState(() => Domain.Planner_kind.Queued);
  let (recruiter, setRecruiter) = React.useState(() => Domain.Recruiter_kind.Situational);
  let difficulty = Setup.difficulty(planner, recruiter);
  let (first, setFirst) = React.useState(() => Setup.Random);
  let (error, setError) = React.useState(() => "");
  let maps =
    List.filter(
      (map: Domain.map_definition) => map.id != "developer",
      Catalog.battle_maps(view.catalog),
    );
  let map = Catalog.map(view.catalog, mapId);
  let campaign = Catalog.campaign(view.catalog, campaignId);
  let mapChoices =
    React.useMemo1(
      () =>
        List.map(
          (map: Domain.map_definition) =>
            choice(
              ~value=map.id,
              ~label=map.id == "river-valley" ? "River Valley" : map.name,
              ~description=Some(map.description),
              ~map=Some(map),
              (),
            ),
          maps,
        ),
      [|view.catalog|],
    );
  let campaigns =
    React.useMemo1(
      () =>
        List.map(
          (campaign: Domain.campaign) =>
            choice(
              ~value=campaign.id,
              ~label=campaign.name,
              ~description=Some(campaign.short_description),
              ~sprite=Some(campaign.leader),
              (),
            ),
          Catalog.campaigns(view.catalog),
        ),
      [|view.catalog|],
    );
  let preloadMap = id => Terrain_preload.map(view.catalog, id);
  let preloadCampaign = id => {
    let missions = Catalog.campaign(view.catalog, id).missions;
    if (Array.length(missions) > 0) {
      preloadMap(missions[0].map.id);
    };
  };
  let setDifficulty = id => {
    let preset = Setup.preset(id);
    setPlanner(_ => preset.planner);
    setRecruiter(_ => preset.recruiter);
  };
  React.useLayoutEffect1(
    () => {
      Option.iter(
        ((_, tab)) => {
          setOpened(_ => true);
          setMode(_ => tab);
          if (tab == Setup.Campaign && difficulty == Setup.Custom) {
            setDifficulty(Setup.Standard);
          };
          setError(_ => "");
          let current = Native.state(view.game).map_id;
          let selectedMap =
            List.exists((map: Domain.map_definition) => map.id == current, maps)
              ? current : "mirror";
          setMap(_ => selectedMap);
          show(dialog);
          if (tab == Setup.Campaign) {
            preloadCampaign(campaignId);
          } else {
            preloadMap(selectedMap);
          };
        },
        request,
      );
      None;
    },
    [|request|],
  );
  React.useLayoutEffect1(
    () => {
      if (opened) {
        Option.iter(
          node => Option.iter(focus, Js.Nullable.toOption(queryWithin(node, ".setup-close"))),
          Js.Nullable.toOption(dialog.current),
        );
      };
      None;
    },
    [|opened|],
  );
  let (starting, setStarting) = React.useState(() => false);
  let startGuard = React.useRef(false);
  let start = () =>
    if (!startGuard.current) {
      startGuard.current = true;
      setStarting(_ => true);
      close(confirmation);
      let opening =
        mode == Setup.Campaign
          ? Domain.Side.Player
          : (
            switch (first) {
            | Setup.Random => random() < 0.5 ? Domain.Side.Player : Domain.Side.Enemy
            | Setup.First(side) => side
            }
          );
      let setup =
        switch (mode) {
        | Setup.Skirmish => Setup.skirmish(~first=opening, ~planner, ~recruiter)
        | Setup.Campaign =>
          Setup.campaign(
            switch (difficulty) {
            | Setup.Preset(policy) => policy
            | Setup.Custom => Setup.Standard
            },
          )
        };
      try(
        {
          if (mode == Setup.Campaign) {
            actions.start_campaign(~setup, campaignId);
          } else {
            preloadMap(mapId);
            actions.reset(
              ~setup,
              ~armies=?mapId == "developer" ? None : Some((player, opponent)),
              mapId,
            );
          };
          let _ =
            Session.saved(session)
            |> Js.Promise.then_(() => {
                 startGuard.current = false;
                 setStarting(_ => false);
                 close(dialog);
                 Js.Promise.resolve();
               })
            |> Js.Promise.catch(failure => {
                 startGuard.current = false;
                 setStarting(_ => false);
                 setError(_ => Save_json.detail(failure));
                 Js.Promise.resolve();
               });
          ();
        }
      ) {
      | failure =>
        startGuard.current = false;
        setStarting(_ => false);
        setError(_ => Fault.describe(failure));
      };
    };
  let requestStart = () => {
    let state = Native.state(view.game);
    if (!view.awaiting_new_game
        && state.map_id != "developer"
        && state.winner == None
        && view.completed_turns > 1.) {
      show(confirmation);
      Option.iter(
        node => Option.iter(focus, Js.Nullable.toOption(queryWithin(node, ".quiet"))),
        Js.Nullable.toOption(confirmation.current),
      );
    } else {
      start();
    };
  };
  <>
    <Dialog_window
      title="New game"
      className="new-game-dialog"
      tall=true
      dialog
      width="520px"
      canDismiss={!view.awaiting_new_game}
      onBack=?{
        view.awaiting_new_game
          ? None
          : Option.map(
              (callback, ()) => {
                close(dialog);
                callback();
              },
              onBack,
            )
      }>
      {opened
         ? <>
             <nav className="setup-tabs" ariaLabel="Game mode">
               <button
                 ariaPressed={string_of_bool(mode == Setup.Skirmish)}
                 onClick={_ => {
                   setMode(_ => Setup.Skirmish);
                   setError(_ => "");
                   preloadMap(mapId);
                 }}>
                 {text("Skirmish")}
               </button>
               <button
                 ariaPressed={string_of_bool(mode == Setup.Campaign)}
                 onClick={_ => {
                   setMode(_ => Setup.Campaign);
                   setError(_ => "");
                   preloadCampaign(campaignId);
                   if (difficulty == Setup.Custom) {
                     setDifficulty(Setup.Standard);
                   };
                 }}>
                 {text("Campaign")}
               </button>
             </nav>
             {mode == Setup.Skirmish
                ? <>
                    <div className="setup-map">
                      <MapThumbnail map />
                      <div>
                        <ChoicePicker
                          label="Battlefield"
                          choices=mapChoices
                          value=mapId
                          onChange={id => {
                            setMap(_ => id);
                            preloadMap(id);
                          }}
                        />
                        <p className="setup-meta"> {text(map.description)} </p>
                      </div>
                    </div>
                    {mapId != "developer"
                       ? <div className="setup-armies">
                           {eachList(
                              [
                                (player, "Your faction", "player"),
                                (opponent, "Opponent", "enemy"),
                              ],
                              ((id, label, side)) => {
                                let army = List.find(choice => choice.value == id, factions);
                                let props = ReactDOM.domProps(~className="setup-army", ());
                                dataSide(props, side);
                                ReactDOM.createElement(
                                  "div",
                                  ~props,
                                  [|
                                    <span className="setup-label"> {text(label)} </span>,
                                    <Sprite
                                      artwork={view.artwork}
                                      name={Option.get(army.sprite)}
                                      width=64.
                                      height=64.
                                      side
                                    />,
                                    <ChoicePicker
                                      label={
                                        side == "player" ? "Your faction" : "Opponent faction"
                                      }
                                      choices=factions
                                      value=id
                                      side
                                      onChange={value =>
                                        side == "player"
                                          ? setPlayer(_ => value) : setOpponent(_ => value)
                                      }
                                    />,
                                  |],
                                );
                              },
                            )}
                         </div>
                       : React.null}
                  </>
                : <>
                    <span className="setup-label"> {text("Your campaign")} </span>
                    <article className="setup-story">
                      <div className="setup-story-heading">
                        <Sprite
                          artwork={view.artwork}
                          name={campaign.leader}
                          width=64.
                          height=64.
                        />
                        <div>
                          <ChoicePicker
                            label="Campaign"
                            choices=campaigns
                            value=campaignId
                            onChange={value => {
                              setCampaign(_ => value);
                              preloadCampaign(value);
                            }}
                          />
                          <em> {text(campaign.tagline)} </em>
                        </div>
                      </div>
                      <p> {text(campaign.short_description)} </p>
                      <span className="setup-meta">
                        {text(
                           string_of_int(Array.length(campaign.missions))
                           ++ " missions · "
                           ++ (
                             campaign.race == Domain.Race.Human
                               ? "Human"
                               : campaign.race == Domain.Race.Skeletons
                                   ? "Skeleton"
                                   : campaign.race == Domain.Race.Merfolk ? "Merfolk" : "Woodland"
                           )
                           ++ " armies",
                         )}
                      </span>
                    </article>
                  </>}
             <div className="setup-difficulty">
               <span className="setup-label">
                 {text(
                    (mode == Setup.Skirmish ? "Opponent difficulty" : "Campaign difficulty")
                    ++ (difficulty == Setup.Custom ? " · Custom" : ""),
                  )}
               </span>
               <div className="setup-segments">
                 {eachList(Setup.difficulties, preset =>
                    <button
                      key={preset.Setup.id}
                      ariaPressed={string_of_bool(difficulty == Setup.Preset(preset.policy))}
                      onClick={_ => setDifficulty(preset.policy)}>
                      {text(preset.name)}
                    </button>
                  )}
               </div>
               <p className="setup-meta">
                 {text(
                    difficulty == Setup.Preset(Setup.Relaxed)
                      ? "A forgiving opponent with straightforward plans."
                      : difficulty == Setup.Preset(Setup.Standard)
                          ? "A balanced opponent that responds to your army."
                          : difficulty == Setup.Preset(Setup.Veteran)
                              ? "Plans ahead and anticipates the troops you can recruit."
                              : "Custom planner and recruiter.",
                  )}
               </p>
             </div>
             {mode == Setup.Skirmish
                ? <>
                    <div className="setup-field">
                      <span className="setup-label"> {text("First turn")} </span>
                      <ChoicePicker
                        label="First turn"
                        choices=[
                          choice(~value=Setup.Random, ~label="Random", ()),
                          choice(~value=Setup.First(Domain.Side.Player), ~label="You", ()),
                          choice(~value=Setup.First(Domain.Side.Enemy), ~label="Opponent", ()),
                        ]
                        value=first
                        onChange={value => setFirst(_ => value)}
                      />
                    </div>
                    <details className="setup-advanced">
                      <summary> {text("Advanced options")} </summary>
                      <div className="setup-field">
                        <span> {text("Opponent planner")} </span>
                        <ChoicePicker
                          label="Opponent planner"
                          choices={List.map(
                            value =>
                              choice(
                                ~value,
                                ~label=title(Enum.string_of_planner_kind(value)),
                                (),
                              ),
                            Setup.planners,
                          )}
                          value=planner
                          onChange={value => setPlanner(_ => value)}
                        />
                      </div>
                      <div className="setup-field">
                        <span> {text("Opponent recruiter")} </span>
                        <ChoicePicker
                          label="Opponent recruiter"
                          choices={List.map(
                            value =>
                              choice(
                                ~value,
                                ~label=title(Enum.string_of_recruiter_kind(value)),
                                (),
                              ),
                            Setup.recruiters,
                          )}
                          value=recruiter
                          onChange={value => setRecruiter(_ => value)}
                        />
                      </div>
                    </details>
                  </>
                : React.null}
             {error != "" ? <p role="alert"> {text(error)} </p> : React.null}
             <div className="setup-actions">
               <button className="quiet" onClick={_ => close(dialog)}>
                 {text("Keep playing")}
               </button>
               <button className="primary" disabled=starting onClick={_ => requestStart()}>
                 {text("Start " ++ (mode == Setup.Skirmish ? "skirmish" : "campaign"))}
               </button>
             </div>
           </>
         : React.null}
    </Dialog_window>
    <dialog
      className="replace-game-dialog"
      role="alertdialog"
      ariaLabelledby="replace-title"
      ariaDescribedby="replace-description"
      ref={ReactDOM.Ref.domRef(confirmation)}>
      <h2 id="replace-title"> {text("Leave your current battle?")} </h2>
      <p id="replace-description">
        {text("Your current battle will remain available in Saved progress.")}
      </p>
      <div className="setup-actions">
        <button className="quiet" onClick={_ => {close(confirmation)}}>
          {text("Cancel")}
        </button>
        <button className="primary" disabled=starting onClick={_ => start()}>
          {text("Start new game")}
        </button>
      </div>
    </dialog>
  </>;
};
