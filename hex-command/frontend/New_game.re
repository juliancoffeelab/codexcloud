open Ui;
[@mel.scope {js|Math|js}] external random: unit => float = "random";
module MapThumbnail = {
  [@react.component]
  let make = (~map: Domain.browser_map, ~backdrop=false) => {
    let size =
      Geometry.preview_dimensions(
        ~cols=Int32.to_int(map.state.cols),
        ~rows=Int32.to_int(map.state.rows),
      );
    let tiles =
      React.useMemo1(
        () => {
          let terrain = Js.Dict.empty();
          Array.iter(
            (tile: Domain.tile) =>
              Js.Dict.set(
                terrain,
                {
                  let js_text_part_0 = Int32.to_string(tile.col);
                  let js_text_part_2 = Int32.to_string(tile.row);
                  {j|$(js_text_part_0),$(js_text_part_2)|j};
                },
                tile.terrain,
              ),
            map.state.tiles,
          );
          List.init(
            Int32.to_int(map.state.rows) * Int32.to_int(map.state.cols),
            index => {
              let col = index mod Int32.to_int(map.state.cols);
              let row = index / Int32.to_int(map.state.cols);
              let point = Geometry.center(~col, ~row);
              let id = {
                let js_text_part_0 = string_of_int(col);
                let js_text_part_2 = string_of_int(row);
                {j|$(js_text_part_0),$(js_text_part_2)|j};
              };
              <polygon
                key=id
                points={Geometry.hex_points(point.x, point.y)}
                fill={Geometry.preview_color(
                  Option.value(~default={js|plains|js}, Js.Dict.get(terrain, id)),
                )}
                stroke={js|#263322|js}
                strokeWidth={js|2|js}
              />;
            },
          );
        },
        [|map|],
      );
    let symmetric = backdrop && map.id == {js|mirror|js};
    let axis = Geometry.center(~col=Int32.to_int(map.state.cols) / 2, ~row=0).x;
    <svg
      className={js|setup-map-preview|js}
      viewBox={
                let js_text_part_1 = number(symmetric ? axis *. 2. : size.width);
                let js_text_part_3 = number(size.height);
                {j|0 0 $(js_text_part_1) $(js_text_part_3)|j};
              }
      preserveAspectRatio={backdrop ? {js|xMidYMid slice|js} : {js|xMidYMid meet|js}}
      ariaHidden=true>
      {symmetric
         ? <>
             <defs>
               <clipPath id={js|new-game-mirror-half|js}>
                 <rect width={number(axis)} height={number(size.height)} />
               </clipPath>
             </defs>
             <g clipPath={js|url(#new-game-mirror-half)|js}> {eachList(tiles, tile => tile)} </g>
             <g
               clipPath={js|url(#new-game-mirror-half)|js}
               transform={
                           let js_text_part_1 = number(axis *. 2.);
                           {j|translate($(js_text_part_1) 0) scale(-1 1)|j};
                         }>
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
    id: {js|human|js},
    name: {js|Humans|js},
    leader: {js|commander|js},
  },
  {
    id: {js|woodland|js},
    name: {js|Woodland|js},
    leader: {js|wildkeeper|js},
  },
  {
    id: {js|skeletons|js},
    name: {js|Skeletons|js},
    leader: {js|necromancer|js},
  },
  {
    id: {js|coastal|js},
    name: {js|Merfolk|js},
    leader: {js|tidekeeper|js},
  },
];
let title = value =>
  value == {js|aggressive-predictor|js}
    ? {js|Aggressive Predictor|js}
    : (
        text => {
          let js_text_part_0 = Js.String.toUpperCase(Js.String.slice(~start=0, ~end_=1, text));
          let js_text_part_1 = Js.String.slice(~start=1, text);
          {j|$(js_text_part_0)$(js_text_part_1)|j};
        }
      )(
        value,
      );
let explanation = difficulty =>
  switch (difficulty) {
  | Setup.Preset(Setup.Relaxed) => {js|Forgiving, straightforward plans.|js}
  | Setup.Preset(Setup.Standard) => {js|Balanced; responds to your army.|js}
  | Setup.Preset(Setup.Veteran) => {js|Anticipates your recruits.|js}
  | Setup.Custom => {js|Custom planning and recruitment.|js}
  };
let plannerHelp = value =>
  switch (value) {
  | Domain.Planner_kind.Classic => {js|Straightforward tactical plans.|js}
  | Domain.Planner_kind.Queued => {js|Plans actions as a sequence.|js}
  | Domain.Planner_kind.Full => {js|Considers a wider set of actions.|js}
  };
let recruiterHelp = value =>
  switch (value) {
  | Domain.Recruiter_kind.Classic => {js|Straightforward recruitment.|js}
  | Domain.Recruiter_kind.Balanced => {js|Mixes troop roles.|js}
  | Domain.Recruiter_kind.Frontline => {js|Prioritises front-line troops.|js}
  | Domain.Recruiter_kind.Rotation => {js|Rotates recruitment choices.|js}
  | Domain.Recruiter_kind.Situational => {js|Responds to the current battle.|js}
  | Domain.Recruiter_kind.Predictor => {js|Anticipates opposing recruits.|js}
  | Domain.Recruiter_kind.Aggressive_predictor => {js|Predictive recruitment with more aggression.|js}
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
              selected => scroll(selected, { "block": {js|nearest|js} }),
              Js.Nullable.toOption(queryWithin(node, {js|[aria-checked='true']|js})),
            ),
          Js.Nullable.toOption(strip.current),
        );
        None;
      },
      [|value|],
    );
    let change = faction => {
      onChange(faction);
      let _ = Session.menu_cue(~cue=Phaser_sound.faction_hud_cue(faction), session);
      ();
    };
    <section className={js|ng-races|js}>
      <div className={js|ng-label|js}> {text(label)} </div>
      <Radix.Radio.Root value onValueChange=change asChild=true>
        <div
          className={js|ng-strip|js}
          role={js|radiogroup|js}
          ariaLabel=label
          ref={ReactDOM.Ref.domRef(strip)}>
          {eachList(
             factions,
             army => {
               let props = ReactDOM.domProps(~type_={js|button|js}, ());
               dataFaction(props, army.id);
               <Radix.Radio.Item key={army.id} value={army.id} asChild=true>
                 {ReactDOM.createElement(
                    {js|button|js},
                    ~props,
                    [|
                      <Sprite artwork name={army.leader} width=64. height=64. side />,
                      <span> {text(army.name)} </span>,
                    |],
                  )}
               </Radix.Radio.Item>;
             },
           )}
        </div>
      </Radix.Radio.Root>
    </section>;
  };
};
[@react.component]
let make =
    (
      ~session: Session.t,
      ~allowNonHumanMaps,
      ~campaignTags,
      ~scenarioTags,
      ~onMapTagsChange,
      ~view: Session.model,
      ~actions: Session.actions,
      ~request: option((int, Setup.mode)),
      ~onBack: option(unit => unit)=?,
    ) => {
  let modal = React.useRef(Js.Nullable.null);
  let (confirmOpen, setConfirmOpen) = React.useState(() => false);
  let recruitmentTrigger = React.useRef(Js.Nullable.null);
  let (recruitmentOpen, setRecruitmentOpen) = React.useState(() => false);
  let body = React.useRef(Js.Nullable.null);
  let (opened, setOpened) = React.useState(() => false);
  let (browsing, setBrowsing) = React.useState(() => true);
  let (mode, setMode) = React.useState(() => Setup.Campaign);
  let (mapId, setMap) = React.useState(() => {js|mirror|js});
  let (campaignId, setCampaign) = React.useState(() => {js|lantern-road|js});
  let (playerAi, setPlayerAi) = React.useState(() => None);
  let (player, setPlayer) = React.useState(() => {js|human|js});
  let (opponent, setOpponent) = React.useState(() => {js|skeletons|js});
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
  let (first, setFirst) = React.useState(() => Setup.First(Domain.Turn_phase.Player));
  let (error, setError) = React.useState(() => {js||js});
  let (starting, setStarting) = React.useState(() => false);
  let startGuard = React.useRef(false);
  let difficulty = Setup.difficulty(planner, recruiter);
  let selectedTags = mode == Setup.Campaign ? campaignTags : scenarioTags;
  let changeSelectedTags = tags => onMapTagsChange(mode, tags);
  let permitted = tags => allowNonHumanMaps || Array.exists(tag => tag == {js|human|js}, tags);
  let matches = (selection, tags) =>
    selection == [] || Array.exists(tag => List.mem(tag, selection), tags);
  let allMaps =
    List.filter(
      (map: Domain.browser_map) => permitted(map.tags),
      Catalog.scenario_maps(view.catalog),
    );
  let allCampaigns =
    List.filter(
      (c: Domain.browser_campaign) => permitted(c.tags),
      Catalog.campaigns(view.catalog),
    );
  let maps =
    List.filter(
      (map: Domain.browser_map) => !allowNonHumanMaps || matches(selectedTags, map.tags),
      allMaps,
    );
  let campaigns =
    List.filter(
      (c: Domain.browser_campaign) => !allowNonHumanMaps || matches(selectedTags, c.tags),
      allCampaigns,
    );
  let entries =
    mode == Setup.Campaign
      ? List.map((c: Domain.browser_campaign) => c.tags, allCampaigns)
      : List.map((map: Domain.browser_map) => map.tags, allMaps);
  let tagOptions =
    selectedTags
    @ List.concat_map(Array.to_list, entries)
    |> List.sort_uniq((a, b) =>
         a == b ? 0 : a == {js|human|js} ? (-1) : b == {js|human|js} ? 1 : Stdlib.compare(a, b)
       )
    |> List.map(tag =>
         (
           tag,
           List.length(List.filter(tags => Array.exists(value => value == tag, tags), entries)),
         )
       );
  let countTags = selection =>
    List.length(List.filter(tags => matches(selection, tags), entries));
  let tagsView = tags =>
    allowNonHumanMaps
      ? <span className={js|ng-map-tags|js}>
          {eachList(Array.to_list(tags), tag =>
             <span key=tag> {text(Map_tag_filter.label(tag))} </span>
           )}
        </span>
      : React.null;
  let campaign = Catalog.campaign(view.catalog, campaignId);
  let selectedMap =
    mode == Setup.Campaign
      ? Catalog.mission_map(view.catalog, campaign.missions[0])
      : Catalog.map(view.catalog, mapId);
  let fixedArmies = selectedMap.fixed_armies;
  let mapName = (map: Domain.browser_map) =>
    map.id == {js|river-valley|js} ? {js|River Valley|js} : map.name;
  let setDifficulty = policy => {
    let preset = Setup.preset(policy);
    setPlanner(_ => preset.planner);
    setRecruiter(_ => preset.recruiter);
  };
  let preloadCampaign = (c: Domain.browser_campaign) =>
    Terrain_preload.map(view.catalog, c.missions[0].map.id);
  let dismiss = () =>
    if (!view.awaiting_new_game && !starting) {
      setOpened(_ => false);
      Option.iter(callback => callback(), onBack);
    };
  React.useLayoutEffect1(
    () => {
      Option.iter(
        ((_, tab)) => {
          setOpened(_ => true);
          setPlayerAi(_ => None);
          setBrowsing(_ => true);
          setMode(_ => tab);
          setError(_ => {js||js});
          setOpened(_ => true);
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
  React.useLayoutEffect2(
    () => {
      if (opened) {
        Option.iter(
          node =>
            Option.iter(
              focus,
              Js.Nullable.toOption(
                queryWithin(
                  node,
                  browsing ? {js|.ng-choice[aria-current='true']|js} : {js|header h2|js},
                ),
              ),
            ),
          Js.Nullable.toOption(modal.current),
        );
      };
      None;
    },
    (opened, browsing),
  );
  React.useLayoutEffect3(
    () => {
      if (opened && browsing) {
        Option.iter(
          node =>
            Option.iter(
              selected =>
                if (Js.Nullable.toOption(queryWithin(node, {js|.ng-choice|js}))
                    == Some(selected)) {
                  Option.iter(
                    body => setScrollTop(body, 0.),
                    Js.Nullable.toOption(body.current),
                  );
                } else {
                  scroll(selected, { "block": {js|nearest|js} });
                },
              Js.Nullable.toOption(queryWithin(node, {js|.ng-choice[aria-current='true']|js})),
            ),
          Js.Nullable.toOption(modal.current),
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
      setConfirmOpen(_ => false);
      let opening =
        switch (first) {
        | Setup.Random => random() < 0.5 ? Domain.Turn_phase.Player : Domain.Turn_phase.Ai
        | Setup.First(side) => side
        };
      let setup =
        mode == Setup.Campaign
          ? switch (difficulty) {
            | Setup.Preset(policy) => Setup.campaign(policy)
            | Setup.Custom => Setup.campaign(~overrides=(planner, recruiter), Setup.Veteran)
            }
          : Setup.skirmish(~first=opening, ~planner, ~recruiter);
      let setup = Setup.with_player_ai(setup, playerAi);
      try(
        {
          if (mode == Setup.Campaign) {
            actions.start_campaign(~setup, campaignId);
          } else {
            actions.reset(~setup, ~armies=?fixedArmies ? None : Some((player, opponent)), mapId);
          };
          let _ =
            Session.saved(session)
            |> Js.Promise.then_(() => {
                 startGuard.current = false;
                 setStarting(_ => false);
                 setOpened(_ => false);
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
  let requestStart = event => {
    let state = Native.state(view.game);
    if (!view.awaiting_new_game
        && state.map_id != {js|developer|js}
        && state.winner == None
        && view.completed_turns > 1.) {
      ();
    } else {
      React.Event.Mouse.preventDefault(event);
      start();
    };
  };
  let chevron =
    <svg className={js|chevron|js} viewBox={js|0 0 16 16|js} ariaHidden=true>
      <path d={js|m3 6 5 5 5-5|js} />
    </svg>;
  let startControl =
    <Modal
      trigger={
        <button className={js|primary|js} disabled=starting onClick=requestStart>
          {text(starting ? {js|Starting…|js} : {js|Start game|js})}
        </button>
      }
      title={js|Leave your current battle?|js}
      opened=confirmOpen
      onOpenChange={value => setConfirmOpen(_ => value)}
      alert=true
      canDismiss={!starting}
      className={js|replace-game-dialog|js}>
      <h2 id={js|replace-title|js}> {text({js|Leave your current battle?|js})} </h2>
      <p id={js|replace-description|js}>
        {text({js|Your current battle will remain available in Saved games.|js})}
      </p>
      <div className={js|ng-confirm-actions|js}>
        <Radix.AlertCancel asChild=true>
          <button className={js|quiet|js} onClick={_ => setConfirmOpen(_ => false)}>
            {text({js|Cancel|js})}
          </button>
        </Radix.AlertCancel>
        <button className={js|primary|js} disabled=starting onClick={_ => start()}>
          {text({js|Start new game|js})}
        </button>
      </div>
    </Modal>;
  let footer =
    <footer
      className={browsing ? {js|ng-footer ng-browse-footer|js} : {js|ng-footer ng-start-footer|js}}>
      {browsing
         ? <>
             <div className={js|ng-browse-tools|js}>
               <Radix.Tabs.List asChild=true>
                 <nav className={js|ng-modes|js} ariaLabel={js|Game mode|js}>
                   {eachList(
                      [
                        (Setup.Campaign, {js|Campaigns|js}),
                        (Setup.Skirmish, {js|Scenarios|js}),
                      ],
                      ((tab, label)) =>
                      <Radix.Tabs.Trigger
                        key=label
                        value={tab == Setup.Campaign ? {js|campaign|js} : {js|scenario|js}}
                        disabled=starting
                        asChild=true>
                        <button> {text(label)} </button>
                      </Radix.Tabs.Trigger>
                    )}
                 </nav>
               </Radix.Tabs.List>
               {allowNonHumanMaps
                  ? <Map_tag_filter
                      options=tagOptions
                      selected=selectedTags
                      onChange=changeSelectedTags
                      count=countTags
                    />
                  : React.null}
             </div>
             {!view.awaiting_new_game
                ? <button onClick={_ => dismiss()} disabled=starting>
                    {text({js|Cancel|js})}
                  </button>
                : React.null}
           </>
         : <>
             <button disabled=starting onClick={_ => setBrowsing(_ => true)}>
               {text({js|Back|js})}
             </button>
             startControl
           </>}
    </footer>;
  <Radix.Tabs.Root
    value={mode == Setup.Campaign ? {js|campaign|js} : {js|scenario|js}}
    onValueChange={value => {
      let tab = value == {js|campaign|js} ? Setup.Campaign : Setup.Skirmish;
      setMode(_ => tab);
      setError(_ => {js||js});
      if (tab == Setup.Campaign) {
        preloadCampaign(campaign);
      } else {
        Terrain_preload.map(view.catalog, mapId);
      };
    }}
    asChild=true>
    <div>
      <Dialog_window
        onCloseAutoFocus={event =>
          if (onBack == None) {
            Modal.preventDefault(event);
            Option.iter(focus, Js.Nullable.toOption(byId({js|settings-button|js})));
          }
        }
        title={
          browsing
            ? {js|New game|js} : mode == Setup.Campaign ? campaign.name : mapName(selectedMap)
        }
        className={
                    let js_text_part_1 = browsing ? {js|ng-browse|js} : {js|ng-setup|js};
                    {j|new-game-dialog settings-style $(js_text_part_1)|j};
                  }
        opened
        onOpenChange={value =>
          if (!value) {
            dismiss();
          }
        }
        body
        width={js|420px|js}
        tall=true
        canDismiss={!view.awaiting_new_game && !starting}
        headerArtwork=?{!opened || browsing ? None : Some(<MapThumbnail map=selectedMap />)}
        footer
        contentRef=modal
        onOpenAutoFocus={event => {
          Modal.preventDefault(event);
          Option.iter(
            node =>
              Option.iter(
                focus,
                Js.Nullable.toOption(
                  queryWithin(
                    node,
                    browsing ? {js|.ng-choice[aria-current='true']|js} : {js|header h2|js},
                  ),
                ),
              ),
            Js.Nullable.toOption(modal.current),
          );
        }}>
        {opened
           ? <>
               {browsing
                  ? <>
                      {allowNonHumanMaps
                         ? <div className={js|ng-map-filters|js}>
                             <p className={js|help|js}>
                               {text(
                                  {
                                    let js_text_part_0 = string_of_int(countTags(selectedTags));
                                    let js_text_part_1 =
                                      mode == Setup.Campaign
                                        ? {js| campaigns|js} : {js| scenarios|js};
                                    {j|$(js_text_part_0)$(js_text_part_1)|j};
                                  },
                                )}
                             </p>
                           </div>
                         : React.null}
                      {countTags(allowNonHumanMaps ? selectedTags : []) == 0
                         ? <div className={js|ng-empty|js}>
                             <p> {text({js|No maps match these tags.|js})} </p>
                             <button onClick={_ => changeSelectedTags([])}>
                               {text({js|Clear filters|js})}
                             </button>
                           </div>
                         : React.null}
                      <Radix.Tabs.Content
                        value={mode == Setup.Campaign ? {js|campaign|js} : {js|scenario|js}}
                        asChild=true>
                        <div className={js|ng-cards|js}>
                          {mode == Setup.Campaign
                             ? eachList(campaigns, (c: Domain.browser_campaign) =>
                                 <button
                                   key={c.id}
                                   ariaCurrent={c.id == campaignId ? {js|true|js} : {js|false|js}}
                                   className={js|ng-choice|js}
                                   onFocus={_ => preloadCampaign(c)}
                                   onPointerEnter={_ => preloadCampaign(c)}
                                   onClick={_ => {
                                     setCampaign(_ => c.id);
                                     setBrowsing(_ => false);
                                     setError(_ => {js||js});
                                     preloadCampaign(c);
                                   }}>
                                   <span className={js|ng-choice-map|js}>
                                     <MapThumbnail
                                       map={Catalog.mission_map(view.catalog, c.missions[0])}
                                       backdrop=true
                                     />
                                   </span>
                                   <span className={js|ng-choice-art|js}>
                                     <Sprite
                                       artwork={view.artwork}
                                       name={c.leader}
                                       width=64.
                                       height=64.
                                     />
                                   </span>
                                   <strong> {text(c.name)} </strong>
                                   {tagsView(c.tags)}
                                   <p> {text(c.short_description)} </p>
                                   <small>
                                     {text(
                                        {
                                          let js_text_part_0 =
                                            string_of_int(Array.length(c.missions));
                                          let js_text_part_1 =
                                            Array.length(c.missions) == 1
                                              ? {js| mission|js} : {js| missions|js};
                                          {j|$(js_text_part_0)$(js_text_part_1)|j};
                                        },
                                      )}
                                   </small>
                                 </button>
                               )
                             : eachList(maps, (map: Domain.browser_map) =>
                                 <button
                                   key={map.id}
                                   ariaCurrent={map.id == mapId ? {js|true|js} : {js|false|js}}
                                   className={js|ng-choice|js}
                                   onFocus={_ => Terrain_preload.map(view.catalog, map.id)}
                                   onPointerEnter={_ => Terrain_preload.map(view.catalog, map.id)}
                                   onClick={_ => {
                                     setMap(_ => map.id);
                                     setBrowsing(_ => false);
                                     setError(_ => {js||js});
                                     Terrain_preload.map(view.catalog, map.id);
                                   }}>
                                   <span className={js|ng-choice-map|js}>
                                     <MapThumbnail map backdrop=true />
                                   </span>
                                   <strong> {text(mapName(map))} </strong>
                                   {tagsView(map.tags)}
                                   <p> {text(map.description)} </p>
                                 </button>
                               )}
                        </div>
                      </Radix.Tabs.Content>
                    </>
                  : <>
                      {mode == Setup.Campaign
                         ? <div className={js|ng-campaign-info|js}>
                             <Sprite
                               artwork={view.artwork}
                               name={campaign.leader}
                               width=64.
                               height=64.
                             />
                             <p> {text(campaign.short_description)} </p>
                           </div>
                         : !fixedArmies
                             ? <>
                                 <RaceStrip
                                   session
                                   label={js|Your race|js}
                                   value=player
                                   onChange={value => setPlayer(_ => value)}
                                   side={js|player|js}
                                   artwork={view.artwork}
                                 />
                                 <RaceStrip
                                   session
                                   label={js|Opponent’s race|js}
                                   value=opponent
                                   onChange={value => setOpponent(_ => value)}
                                   side={js|enemy|js}
                                   artwork={view.artwork}
                                 />
                               </>
                             : <p className={js|help|js}>
                                 {text({js|This map provides the starting armies.|js})}
                               </p>}
                      <section className={js|ng-difficulty|js}>
                        <div className={js|ng-label|js}> {text({js|Difficulty|js})} </div>
                        <Radix.Radio.Root
                          value={
                            difficulty == Setup.Custom
                              ? {js|custom|js}
                              : List.find(
                                  preset => difficulty == Setup.Preset(preset.Setup.policy),
                                  Setup.difficulties,
                                ).
                                  id
                          }
                          onValueChange={value =>
                            Option.iter(
                              preset => setDifficulty(preset.Setup.policy),
                              List.find_opt(
                                preset => preset.Setup.id == value,
                                Setup.difficulties,
                              ),
                            )
                          }
                          asChild=true>
                          <div
                            className={
                                        let js_text_part_1 =
                                          difficulty == Setup.Custom
                                            ? {js| ng-custom|js} : {js||js};
                                        {j|ng-segments$(js_text_part_1)|j};
                                      }
                            role={js|radiogroup|js}
                            ariaLabel={js|Difficulty|js}>
                            {eachList(Setup.difficulties, preset =>
                               <Radix.Radio.Item
                                 key={preset.Setup.id} value={preset.id} asChild=true>
                                 <button> {text(preset.name)} </button>
                               </Radix.Radio.Item>
                             )}
                            {difficulty == Setup.Custom
                               ? <Radix.Radio.Item value={js|custom|js} asChild=true>
                                   <button> {text({js|Custom|js})} </button>
                                 </Radix.Radio.Item>
                               : React.null}
                          </div>
                        </Radix.Radio.Root>
                        <p className={js|help|js}> {text(explanation(difficulty))} </p>
                      </section>
                      <details className={js|advanced|js}>
                        <summary className={js|setting-row|js}>
                          <span className={js|label|js}> {text({js|Advanced|js})} </span>
                          chevron
                        </summary>
                        {mode == Setup.Skirmish
                           ? <div className={js|ng-advanced-group|js}>
                               <span className={js|label|js}> {text({js|First turn|js})} </span>
                               <Radix.Radio.Root
                                 value={
                                   first == Setup.Random
                                     ? {js|random|js}
                                     : first == Setup.First(Domain.Turn_phase.Player)
                                         ? {js|player|js} : {js|ai|js}
                                 }
                                 onValueChange={value =>
                                   setFirst(_ =>
                                     value == {js|random|js}
                                       ? Setup.Random
                                       : Setup.First(
                                           value == {js|player|js}
                                             ? Domain.Turn_phase.Player : Domain.Turn_phase.Ai,
                                         )
                                   )
                                 }
                                 asChild=true>
                                 <div
                                   className={js|ng-segments|js}
                                   role={js|radiogroup|js}
                                   ariaLabel={js|First turn|js}>
                                   {eachList(
                                      [
                                        ({js|random|js}, {js|Random|js}),
                                        ({js|player|js}, {js|You|js}),
                                        ({js|ai|js}, {js|Opponent|js}),
                                      ],
                                      ((value, label)) =>
                                      <Radix.Radio.Item key=value value asChild=true>
                                        <button> {text(label)} </button>
                                      </Radix.Radio.Item>
                                    )}
                                 </div>
                               </Radix.Radio.Root>
                             </div>
                           : React.null}
                        <div className={js|ng-advanced-group|js}>
                          <span className={js|label|js}> {text({js|Opponent planning|js})} </span>
                          <Radix.Radio.Root
                            value={Enum.string_of_planner_kind(planner)}
                            onValueChange={value =>
                              Option.iter(
                                planner => setPlanner(_ => planner),
                                List.find_opt(
                                  planner => Enum.string_of_planner_kind(planner) == value,
                                  Setup.planners,
                                ),
                              )
                            }
                            asChild=true>
                            <div
                              className={js|ng-segments|js}
                              role={js|radiogroup|js}
                              ariaLabel={js|Opponent planning|js}>
                              {eachList(Setup.planners, value =>
                                 <Radix.Radio.Item
                                   key={Enum.string_of_planner_kind(value)}
                                   value={Enum.string_of_planner_kind(value)}
                                   asChild=true>
                                   <button>
                                     {text(title(Enum.string_of_planner_kind(value)))}
                                   </button>
                                 </Radix.Radio.Item>
                               )}
                            </div>
                          </Radix.Radio.Root>
                          <p className={js|help|js}> {text(plannerHelp(planner))} </p>
                        </div>
                        <Radix.Collapsible.Root
                          _open=recruitmentOpen
                          onOpenChange={value => setRecruitmentOpen(_ => value)}
                          asChild=true>
                          <div className={js|ng-recruitment|js}>
                            <Radix.Collapsible.Trigger asChild=true>
                              <button
                                className={js|setting-row recruitment-trigger|js}
                                ref={ReactDOM.Ref.domRef(recruitmentTrigger)}>
                                <span className={js|label|js}> {text({js|Recruitment|js})} </span>
                                <span className={js|ng-recruit-value|js}>
                                  {text(title(Enum.string_of_recruiter_kind(recruiter)))}
                                </span>
                                chevron
                              </button>
                            </Radix.Collapsible.Trigger>
                            <Radix.Collapsible.Content asChild=true>
                              <div role={js|radiogroup|js} ariaLabel={js|Recruitment|js}>
                                {eachList(Setup.recruiters, value =>
                                   <label className={js|ng-recruit-option|js}>
                                     <input
                                       type_={js|radio|js}
                                       name={js|new-game-recruiter|js}
                                       checked={recruiter == value}
                                       onChange={_ => {
                                         setRecruiter(_ => value);
                                         setRecruitmentOpen(_ => false);
                                         Option.iter(
                                           focus,
                                           Js.Nullable.toOption(recruitmentTrigger.current),
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
                            </Radix.Collapsible.Content>
                          </div>
                        </Radix.Collapsible.Root>
                        <p className={js|help|js}> {text(recruiterHelp(recruiter))} </p>
                        <div className={js|ng-advanced-group ng-control-options|js}>
                          <span className={js|label|js}> {text({js|Player control|js})} </span>
                          <Radix.Radio.Root
                            value={Option.fold(
                              ~none={js|you|js},
                              ~some=
                                policy =>
                                  List.find(
                                    preset => preset.Setup.policy == policy,
                                    Setup.difficulties,
                                  ).
                                    id,
                              playerAi,
                            )}
                            onValueChange={value =>
                              setPlayerAi(_ =>
                                Option.map(
                                  preset => preset.Setup.policy,
                                  List.find_opt(
                                    preset => preset.Setup.id == value,
                                    Setup.difficulties,
                                  ),
                                )
                              )
                            }
                            asChild=true>
                            <div
                              className={js|ng-segments ng-player-control|js}
                              role={js|radiogroup|js}
                              ariaLabel={js|Player control|js}>
                              <Radix.Radio.Item value={js|you|js} asChild=true>
                                <button> {text({js|You|js})} </button>
                              </Radix.Radio.Item>
                              {eachList(Setup.difficulties, preset =>
                                 <Radix.Radio.Item
                                   key={preset.Setup.id} value={preset.id} asChild=true>
                                   <button>
                                     {text(
                                        {
                                          let js_text_part_0 = preset.name;
                                          {j|$(js_text_part_0) AI|j};
                                        },
                                      )}
                                   </button>
                                 </Radix.Radio.Item>
                               )}
                            </div>
                          </Radix.Radio.Root>
                          <p className={js|help|js}>
                            {text(
                               {js|AI control is saved with this battle. Story dialogue still waits for you.|js},
                             )}
                          </p>
                        </div>
                      </details>
                    </>}
               {error != {js||js}
                  ? <p role={js|alert|js} className={js|ng-error|js}> {text(error)} </p>
                  : React.null}
             </>
           : React.null}
      </Dialog_window>
    </div>
  </Radix.Tabs.Root>;
};
