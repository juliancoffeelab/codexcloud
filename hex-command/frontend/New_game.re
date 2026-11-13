open Ui;
[@mel.scope "Math"] external random: unit => float = "random";

module MapThumbnail = {
  [@react.component]
  let make = (~map: Domain.map_definition, ~backdrop=false) => {
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
    // The browsing artwork is decorative; the setup preview keeps the authored layout.
    let symmetric = backdrop && map.id == "mirror";
    let axis = Geometry.center(~col=Int32.to_int(map.cols) / 2, ~row=0).x;
    <svg
      className="setup-map-preview"
      viewBox={
        "0 0 " ++ number(symmetric ? axis *. 2. : size.width) ++ " " ++ number(size.height)
      }
      preserveAspectRatio={backdrop ? "xMidYMid slice" : "xMidYMid meet"}
      ariaHidden=true>
      {symmetric
         ? <>
             <defs>
               <clipPath id="new-game-mirror-half">
                 <rect width={number(axis)} height={number(size.height)} />
               </clipPath>
             </defs>
             <g clipPath="url(#new-game-mirror-half)"> {eachList(tiles, tile => tile)} </g>
             <g
               clipPath="url(#new-game-mirror-half)"
               transform={"translate(" ++ number(axis *. 2.) ++ " 0) scale(-1 1)"}>
               {eachList(tiles, tile => tile)}
             </g>
           </>
         : eachList(tiles, tile => tile)}
    </svg>;
  };
};

type faction = {
  id: string,
  name: string,
  leader: string,
};
let factions = [
  {
    id: "human",
    name: "Humans",
    leader: "commander",
  },
  {
    id: "woodland",
    name: "Woodland",
    leader: "wildkeeper",
  },
  {
    id: "skeletons",
    name: "Skeletons",
    leader: "necromancer",
  },
  {
    id: "coastal",
    name: "Merfolk",
    leader: "tidekeeper",
  },
];
let title = value =>
  value == "aggressive-predictor" ? "Aggressive Predictor" : String.capitalize_ascii(value);
let explanation = difficulty =>
  switch (difficulty) {
  | Setup.Preset(Setup.Relaxed) => "Forgiving, straightforward plans."
  | Setup.Preset(Setup.Standard) => "Balanced; responds to your army."
  | Setup.Preset(Setup.Veteran) => "Anticipates your recruits."
  | Setup.Custom => "Custom planning and recruitment."
  };
let plannerHelp = value =>
  switch (value) {
  | Domain.Planner_kind.Classic => "Straightforward tactical plans."
  | Domain.Planner_kind.Queued => "Plans actions as a sequence."
  | Domain.Planner_kind.Full => "Considers a wider set of actions."
  };
let recruiterHelp = value =>
  switch (value) {
  | Domain.Recruiter_kind.Classic => "Straightforward recruitment."
  | Domain.Recruiter_kind.Balanced => "Mixes troop roles."
  | Domain.Recruiter_kind.Frontline => "Prioritises front-line troops."
  | Domain.Recruiter_kind.Rotation => "Rotates recruitment choices."
  | Domain.Recruiter_kind.Situational => "Responds to the current battle."
  | Domain.Recruiter_kind.Predictor => "Anticipates opposing recruits."
  | Domain.Recruiter_kind.Aggressive_predictor => "Predictive recruitment with more aggression."
  };
module RaceStrip = {
  [@react.component]
  let make = (~label, ~value, ~onChange, ~side, ~artwork, ~session) => {
    let strip = React.useRef(Js.Nullable.null);
    React.useLayoutEffect1(
      () => {
        Option.iter(
          node =>
            Option.iter(
              selected => scroll(selected, { "block": "nearest" }),
              Js.Nullable.toOption(queryWithin(node, "[aria-checked='true']")),
            ),
          Js.Nullable.toOption(strip.current),
        );
        None;
      },
      [|value|],
    );
    let navigate = (event, index) => {
      let key = React.Event.Keyboard.key(event);
      if (List.mem(key, ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"])) {
        React.Event.Keyboard.preventDefault(event);
        let length = List.length(factions);
        let next =
          key == "Home"
            ? 0
            : key == "End"
                ? length - 1
                : (index + (key == "ArrowRight" || key == "ArrowDown" ? 1 : length - 1)) mod length;
        Option.iter(
          node => focus(descendants(node, "button")[next]),
          Js.Nullable.toOption(strip.current),
        );
        let faction = List.nth(factions, next).id;
        onChange(faction);
        if (next != index) {
          let _ = Session.menu_cue(~cue=Phaser_sound.faction_hud_cue(faction), session);
          ();
        };
      };
    };
    <section className="ng-races">
      <div className="ng-label"> {text(label)} </div>
      <div
        className="ng-strip"
        role="radiogroup"
        ariaLabel={Utf8.to_js(label)}
        ref={ReactDOM.Ref.domRef(strip)}>
        {eachList(
           List.mapi((index, army) => (index, army), factions),
           ((index, army)) => {
             let props =
               ReactDOM.domProps(
                 ~role="radio",
                 ~ariaChecked=string_of_bool(army.id == value),
                 ~tabIndex=army.id == value ? 0 : (-1),
                 ~onKeyDown=event => navigate(event, index),
                 ~onClick=_ => onChange(army.id),
                 (),
               );
             dataKey(props, army.id);
             dataFaction(props, army.id);
             ReactDOM.createElement(
               "button",
               ~props,
               [|
                 <Sprite artwork name={army.leader} width=64. height=64. side />,
                 <span> {text(army.name)} </span>,
               |],
             );
           },
         )}
      </div>
    </section>;
  };
};
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
  let recruitment = React.useRef(Js.Nullable.null);
  let body = React.useRef(Js.Nullable.null);
  let (opened, setOpened) = React.useState(() => false);
  let (browsing, setBrowsing) = React.useState(() => true);
  let (mode, setMode) = React.useState(() => Setup.Campaign);
  let (mapId, setMap) = React.useState(() => "mirror");
  let (campaignId, setCampaign) = React.useState(() => "lantern-road");
  let (player, setPlayer) = React.useState(() => "human");
  let (opponent, setOpponent) = React.useState(() => "skeletons");
  let (campaignPlanner, setCampaignPlanner) = React.useState(() => Domain.Planner_kind.Queued);
  let (campaignRecruiter, setCampaignRecruiter) =
    React.useState(() => Domain.Recruiter_kind.Situational);
  let (scenarioPlanner, setScenarioPlanner) = React.useState(() => Domain.Planner_kind.Queued);
  let (scenarioRecruiter, setScenarioRecruiter) =
    React.useState(() => Domain.Recruiter_kind.Situational);
  let planner = mode == Setup.Campaign ? campaignPlanner : scenarioPlanner;
  let recruiter = mode == Setup.Campaign ? campaignRecruiter : scenarioRecruiter;
  let setPlanner = mode == Setup.Campaign ? setCampaignPlanner : setScenarioPlanner;
  let setRecruiter = mode == Setup.Campaign ? setCampaignRecruiter : setScenarioRecruiter;
  let (first, setFirst) = React.useState(() => Setup.First(Domain.Side.Player));
  let (error, setError) = React.useState(() => "");
  let (starting, setStarting) = React.useState(() => false);
  let startGuard = React.useRef(false);
  let difficulty = Setup.difficulty(planner, recruiter);
  let maps =
    List.filter((map: Domain.map_definition) => map.human, Catalog.scenario_maps(view.catalog));
  let campaigns = List.filter((c: Domain.campaign) => c.human, Catalog.campaigns(view.catalog));
  let campaign = Catalog.campaign(view.catalog, campaignId);
  let selectedMap =
    mode == Setup.Campaign
      ? Catalog.mission_map(view.catalog, campaign.missions[0])
      : Catalog.map(view.catalog, mapId);
  let mapName = (map: Domain.map_definition) =>
    map.id == "river-valley" ? "River Valley" : map.name;
  let setDifficulty = policy => {
    let preset = Setup.preset(policy);
    setPlanner(_ => preset.planner);
    setRecruiter(_ => preset.recruiter);
  };
  let preloadCampaign = (c: Domain.campaign) =>
    Terrain_preload.map(view.catalog, c.missions[0].map.id);
  let dismiss = () =>
    if (!view.awaiting_new_game && !starting) {
      close(dialog);
      Option.iter(callback => callback(), onBack);
    };
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
  React.useLayoutEffect1(
    () => {
      Option.iter(
        ((_, tab)) => {
          setOpened(_ => true);
          setBrowsing(_ => true);
          setMode(_ => tab);
          setError(_ => "");
          show(dialog);
          if (tab == Setup.Campaign) {
            preloadCampaign(campaign);
          } else {
            Terrain_preload.map(view.catalog, mapId);
          };
        },
        request,
      );
      None;
    },
    [|request|],
  );
  React.useLayoutEffect3(
    () => {
      if (opened) {
        Option.iter(
          node =>
            Option.iter(
              focus,
              Js.Nullable.toOption(
                queryWithin(node, browsing ? ".ng-choice[aria-current='true']" : "header h2"),
              ),
            ),
          Js.Nullable.toOption(dialog.current),
        );
      };
      None;
    },
    (opened, browsing, mode),
  );
  let start = () =>
    if (!startGuard.current) {
      startGuard.current = true;
      setStarting(_ => true);
      close(confirmation);
      let opening =
        switch (first) {
        | Setup.Random => random() < 0.5 ? Domain.Side.Player : Domain.Side.Enemy
        | Setup.First(side) => side
        };
      let setup =
        mode == Setup.Campaign
          ? switch (difficulty) {
            | Setup.Preset(policy) => Setup.campaign(policy)
            | Setup.Custom => Setup.campaign(~overrides=(planner, recruiter), Setup.Veteran)
            }
          : Setup.skirmish(~first=opening, ~planner, ~recruiter);
      try(
        {
          if (mode == Setup.Campaign) {
            actions.start_campaign(~setup, campaignId);
          } else {
            actions.reset(~setup, ~armies=(player, opponent), mapId);
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
        setError(_ => Foreign_error.exception_(failure));
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
  let chevron =
    <svg className="chevron" viewBox="0 0 16 16" ariaHidden=true> <path d="m3 6 5 5 5-5" /> </svg>;
  let footer =
    <footer className={browsing ? "ng-footer ng-browse-footer" : "ng-footer ng-start-footer"}>
      {browsing
         ? <>
             <nav className="ng-modes" ariaLabel="Game mode">
               {eachList(
                  [(Setup.Campaign, "Campaigns"), (Setup.Skirmish, "Scenarios")],
                  ((tab, label)) =>
                  <button
                    ariaPressed={string_of_bool(mode == tab)}
                    disabled=starting
                    onClick={_ => {
                      setMode(_ => tab);
                      setError(_ => "");
                      if (tab == Setup.Campaign) {
                        preloadCampaign(campaign);
                      } else {
                        Terrain_preload.map(view.catalog, mapId);
                      };
                    }}>
                    {text(label)}
                  </button>
                )}
             </nav>
             {!view.awaiting_new_game
                ? <button onClick={_ => dismiss()} disabled=starting> {text("Cancel")} </button>
                : React.null}
           </>
         : <>
             <button disabled=starting onClick={_ => setBrowsing(_ => true)}>
               {text("Back")}
             </button>
             <button className="primary" disabled=starting onClick={_ => requestStart()}>
               {text(starting ? "Starting…" : "Start game")}
             </button>
           </>}
    </footer>;
  <>
    <Dialog_window
      title={
        browsing ? "New game" : mode == Setup.Campaign ? campaign.name : mapName(selectedMap)
      }
      className={"new-game-dialog settings-style " ++ (browsing ? "ng-browse" : "ng-setup")}
      dialog
      body
      width="420px"
      tall=true
      canDismiss={!view.awaiting_new_game && !starting}
      onBeforeClose={() => Option.iter(callback => callback(), onBack)}
      headerArtwork=?{!opened || browsing ? None : Some(<MapThumbnail map=selectedMap />)}
      footer>
      {opened
         ? <>
             {browsing
                ? <div className="ng-cards">
                    {mode == Setup.Campaign
                       ? eachList(campaigns, (c: Domain.campaign) =>
                           <button
                             key={c.id}
                             ariaCurrent={c.id == campaignId ? "true" : "false"}
                             className="ng-choice"
                             onFocus={_ => preloadCampaign(c)}
                             onPointerEnter={_ => preloadCampaign(c)}
                             onClick={_ => {
                               setCampaign(_ => c.id);
                               setBrowsing(_ => false);
                               setError(_ => "");
                               preloadCampaign(c);
                             }}>
                             <span className="ng-choice-map">
                               <MapThumbnail
                                 map={Catalog.mission_map(view.catalog, c.missions[0])}
                                 backdrop=true
                               />
                             </span>
                             <span className="ng-choice-art">
                               <Sprite
                                 artwork={view.artwork}
                                 name={c.leader}
                                 width=64.
                                 height=64.
                               />
                             </span>
                             <strong> {text(c.name)} </strong>
                             <p> {text(c.short_description)} </p>
                             <small>
                               {text(
                                  string_of_int(Array.length(c.missions))
                                  ++ (Array.length(c.missions) == 1 ? " mission" : " missions"),
                                )}
                             </small>
                           </button>
                         )
                       : eachList(maps, (map: Domain.map_definition) =>
                           <button
                             key={map.id}
                             ariaCurrent={map.id == mapId ? "true" : "false"}
                             className="ng-choice"
                             onFocus={_ => Terrain_preload.map(view.catalog, map.id)}
                             onPointerEnter={_ => Terrain_preload.map(view.catalog, map.id)}
                             onClick={_ => {
                               setMap(_ => map.id);
                               setBrowsing(_ => false);
                               setError(_ => "");
                               Terrain_preload.map(view.catalog, map.id);
                             }}>
                             <span className="ng-choice-map">
                               <MapThumbnail map backdrop=true />
                             </span>
                             <strong> {text(mapName(map))} </strong>
                             <p> {text(map.description)} </p>
                           </button>
                         )}
                  </div>
                : <>
                    {mode == Setup.Campaign
                       ? <div className="ng-campaign-info">
                           <Sprite
                             artwork={view.artwork}
                             name={campaign.leader}
                             width=64.
                             height=64.
                           />
                           <p> {text(campaign.short_description)} </p>
                         </div>
                       : <>
                           <RaceStrip
                             session
                             label="Your race"
                             value=player
                             onChange={value => setPlayer(_ => value)}
                             side="player"
                             artwork={view.artwork}
                           />
                           <RaceStrip
                             session
                             label="Opponent’s race"
                             value=opponent
                             onChange={value => setOpponent(_ => value)}
                             side="enemy"
                             artwork={view.artwork}
                           />
                         </>}
                    <section className="ng-difficulty">
                      <div className="ng-label"> {text("Difficulty")} </div>
                      <div
                        className={
                          "ng-segments " ++ (difficulty == Setup.Custom ? "ng-custom" : "")
                        }
                        role="group"
                        ariaLabel="Difficulty">
                        {eachList(Setup.difficulties, preset =>
                           <button
                             key={preset.Setup.id}
                             ariaPressed={string_of_bool(
                               difficulty == Setup.Preset(preset.policy),
                             )}
                             onClick={_ => setDifficulty(preset.policy)}>
                             {text(preset.name)}
                           </button>
                         )}
                        {difficulty == Setup.Custom
                           ? <button ariaPressed="true"> {text("Custom")} </button> : React.null}
                      </div>
                      <p className="help"> {text(explanation(difficulty))} </p>
                    </section>
                    <details className="advanced">
                      <summary className="setting-row">
                        <span className="label"> {text("Advanced")} </span>
                        chevron
                      </summary>
                      {mode == Setup.Skirmish
                         ? <div className="ng-advanced-group">
                             <span className="label"> {text("First turn")} </span>
                             <div className="ng-segments" role="group" ariaLabel="First turn">
                               {eachList(
                                  [
                                    (Setup.Random, "Random"),
                                    (Setup.First(Domain.Side.Player), "You"),
                                    (Setup.First(Domain.Side.Enemy), "Opponent"),
                                  ],
                                  ((value, label)) =>
                                  <button
                                    ariaPressed={string_of_bool(first == value)}
                                    onClick={_ => setFirst(_ => value)}>
                                    {text(label)}
                                  </button>
                                )}
                             </div>
                           </div>
                         : React.null}
                      <div className="ng-advanced-group">
                        <span className="label"> {text("Opponent planning")} </span>
                        <div className="ng-segments" role="group" ariaLabel="Opponent planning">
                          {eachList(Setup.planners, value =>
                             <button
                               ariaPressed={string_of_bool(planner == value)}
                               onClick={_ => setPlanner(_ => value)}>
                               {text(title(Enum.string_of_planner_kind(value)))}
                             </button>
                           )}
                        </div>
                        <p className="help"> {text(plannerHelp(planner))} </p>
                      </div>
                      <details className="ng-recruitment" ref={ReactDOM.Ref.domRef(recruitment)}>
                        <summary className="setting-row">
                          <span className="label"> {text("Recruitment")} </span>
                          <span className="ng-recruit-value">
                            {text(title(Enum.string_of_recruiter_kind(recruiter)))}
                          </span>
                          chevron
                        </summary>
                        <div role="radiogroup" ariaLabel="Recruitment">
                          {eachList(Setup.recruiters, value =>
                             <label className="ng-recruit-option">
                               <input
                                 type_="radio"
                                 name="new-game-recruiter"
                                 checked={recruiter == value}
                                 onChange={_ => {
                                   setRecruiter(_ => value);
                                   Option.iter(
                                     node => {
                                       open_(node, false);
                                       Option.iter(
                                         focus,
                                         Js.Nullable.toOption(queryWithin(node, "summary")),
                                       );
                                     },
                                     Js.Nullable.toOption(recruitment.current),
                                   );
                                 }}
                               />
                               <span>
                                 <strong>
                                   {text(title(Enum.string_of_recruiter_kind(value)))}
                                 </strong>
                                 <small> {text(recruiterHelp(value))} </small>
                               </span>
                             </label>
                           )}
                        </div>
                      </details>
                      <p className="help"> {text(recruiterHelp(recruiter))} </p>
                    </details>
                  </>}
             {error != "" ? <p role="alert" className="ng-error"> {text(error)} </p> : React.null}
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
        {text("Your current battle will remain available in Saved games.")}
      </p>
      <div className="ng-confirm-actions">
        <button className="quiet" onClick={_ => close(confirmation)}> {text("Cancel")} </button>
        <button className="primary" disabled=starting onClick={_ => start()}>
          {text("Start new game")}
        </button>
      </div>
    </dialog>
  </>;
};
