open Ui;

module KeepTogether = {
  [@react.component]
  let make = (~value) => {
    let parts =
      Js.String.splitByRe(~regexp=Js.Re.fromStringWithFlags("(\\([^)]*\\))", ~flags="g"), value);
    each(
      parts,
      part => {
        let part = Option.value(~default="", part);
        String.length(part) > 0 && part.[0] == '('
          ? <span className="keep-together"> {text(part)} </span> : text(part);
      },
    );
  };
};

module UnitCard = {
  [@react.component]
  let make =
      (
        ~view: Hud_details.context,
        ~unit: Domain.public_unit,
        ~game: option(Native.view)=?,
        ~showName=false,
        ~showProgress=false,
        ~hp: option(float)=?,
      ) => {
    let definition = Catalog.unit(view.catalog, unit.type_);
    let faction = Native.faction(Option.value(~default=view.game, game), unit.id);
    let health = Option.value(~default=Int32.to_float(unit.hp), hp);
    let level = number(definition.level);
    let progress =
      "L"
      ++ level
      ++ " · "
      ++ (
        switch (definition.experience_needed) {
        | Some(needed) when needed > 0. =>
          number(Option.value(~default=0., unit.experience)) ++ "/" ++ number(needed) ++ " XP"
        | _ => "Max"
        }
      );
    let hostile =
      Option.fold(
        ~none=false,
        ~some=(f: Domain.unit_faction) => f.relation == Domain.Relation.Hostile,
        faction,
      );
    let controllable =
      Option.fold(~none=false, ~some=(f: Domain.unit_faction) => f.controllable, faction);
    <div
      className={"combatant " ++ (hostile ? "enemy" : "player")}
      role="group"
      ariaLabel={Utf8.to_js(
        Option.fold(
          ~none=false,
          ~some=(faction: Domain.unit_faction) => faction.relation == Domain.Relation.Neutral,
          faction,
        )
          ? "Neutral unit" : controllable ? "Your unit" : hostile ? "Enemy unit" : "Allied unit",
      )}>
      <Sprite
        artwork={view.artwork}
        name={unit.type_}
        side={Option.fold(
          ~none=side(unit.side),
          ~some=(faction: Domain.unit_faction) => faction.palette,
          faction,
        )}
        width=72.
        height=90.
        className={"portrait " ++ (hostile ? "enemy-unit" : "")}
      />
      <div className="unit-heading">
        <h2>
          {text(
             showName
               ? Option.value(~default=definition.name, Native.unit_name(view.game, unit.id))
               : definition.name,
           )}
        </h2>
        {showProgress
           ? <span className="unit-progress" title={Utf8.to_js("Level " ++ level)}>
               {text(progress)}
             </span>
           : React.null}
      </div>
      {Option.value(~default=false, unit.retreating)
         ? <span className="retreat-label"> {text("Retreating")} </span> : React.null}
      {showName && Native.unit_name(view.game, unit.id) != None
         ? <span className="named-unit-type"> {text(definition.name)} </span> : React.null}
      <div className="health-numbers">
        <b> {num(health)} </b>
        <span> {text(" / " ++ number(Int32.to_float(definition.hp)) ++ " HP")} </span>
      </div>
      <div
        className="health-track"
        role="img"
        ariaLabel={Utf8.to_js(
          number(health) ++ " of " ++ number(Int32.to_float(definition.hp)) ++ " current health",
        )}>
        <span
          className="health-left"
          style={percent(health /. Int32.to_float(definition.hp) *. 100.)}
        />
      </div>
    </div>;
  };
};
module ItemList = {
  [@react.component]
  let make = (~items: array(Domain.item), ~artwork: Art.t, ~onDrop: option(string => unit)=?) =>
    <ul className="item-list">
      {items
       |> Array.mapi((index, item: Domain.item) =>
            <li key={item.id ++ "-" ++ string_of_int(index)} className="item-row">
              <span className="item-icon" ariaHidden=true>
                {option(Art.find(artwork, "terrain_prop_items_" ++ item.id), frame =>
                   <img className="pixel-art" src={frame.url} alt="" width="64" height="64" />
                 )}
              </span>
              <div className="item-copy">
                <div className="item-heading">
                  <strong> {text(item.name)} </strong>
                  {option(onDrop, drop => {
                     <button
                       className="item-drop"
                       ariaLabel={"Drop " ++ item.name}
                       onClick={_ => drop(item.id)}>
                       {text("Drop")}
                     </button>
                   })}
                </div>
                <p> {text(item.description)} </p>
              </div>
            </li>
          )
       |> React.array}
    </ul>;
};
module GroundItems = {
  [@react.component]
  let make = (~artwork: Art.t, ~items: array(Domain.item)) => {
    <section
      id="ground-items-panel"
      className="panel"
      ariaLabel="Items on the ground"
      hidden={Array.length(items) == 0}>
      <h2> {text("On the ground")} </h2>
      <ItemList items artwork />
    </section>;
  };
};
module Arsenal = {
  [@react.component]
  let make =
      (~view: Hud_details.context, ~unit: Domain.public_unit, ~onDrop: option(string => unit)=?) => {
    <>
      {Array.length(unit.items) == 0
         ? React.null
         : <section className="equipment" ariaLabel="Equipment">
             <h3> {text("Equipment")} </h3>
             <ItemList items={unit.items} artwork={view.artwork} ?onDrop />
           </section>}
      <div className="arsenal">
        <h3> {text("Attacks")} </h3>
        <table className="weapon-table" ariaLabel="Unit attacks">
          <thead>
            <tr>
              <th scope="col"> {text("Attack")} </th>
              <th scope="col"> {text("Damage")} </th>
              <th scope="col"> {text("Range")} </th>
            </tr>
          </thead>
          <tbody>
            {eachList(Presentation.weapons(view.catalog, unit), weapon =>
               <tr key={weapon.Domain.id}>
                 <th scope="row">
                   <span className="weapon-name">
                     <Icon name={Enum.string_of_damage_type(weapon.damage_type)} />
                     <strong> {text(weapon.name)} </strong>
                   </span>
                   <span className="weapon-detail weapon-category">
                     {text(weapon.category == Domain.Category.Melee ? "Melee" : "Ranged")}
                   </span>
                 </th>
                 <td className="weapon-damage">
                   <b>
                     {text(
                        number(float_of_int(weapon.count))
                        ++ " × "
                        ++ Presentation.weapon_damage(weapon),
                      )}
                   </b>
                   <span className="weapon-detail">
                     {text(Presentation.damage_name(weapon.damage_type))}
                   </span>
                 </td>
                 <td> {text(Presentation.weapon_range(weapon))} </td>
               </tr>
             )}
          </tbody>
        </table>
      </div>
      <div className="resistances">
        <h3> {text("Damage received")} </h3>
        <div className="resistance-list">
          {eachList(Presentation.armor(view.catalog, unit), modifier =>
             <span
               key={modifier.Presentation.name} className={"resistance " ++ modifier.class_name}>
               <span> {text(modifier.name)} </span>
               <b> {text(modifier.label)} </b>
             </span>
           )}
        </div>
      </div>
    </>;
  };
};
module Details = {
  [@react.component]
  let make =
      (~view: Hud_details.context, ~actions: Session.actions, ~details: Presentation.unit_details) =>
    <div className="unit-detail">
      <UnitCard view unit={details.unit} showName=true showProgress=true />
      <div className="unit-info">
        <div className="unit-stats">
          <span className="unit-stat">
            <Icon name="move" />
            <span> {text("Move: ")} <b> {num(details.definition.move)} </b> </span>
          </span>
          <p className="unit-state">
            <span> {text(details.terrain_name)} </span>
            <span> {text(details.action_status)} </span>
          </p>
        </div>
        {switch (
           Array.find_opt(
             (a: Domain.unit_appearance) => a.unit == details.unit.id,
             Native.presentation(view.game).unit_appearances,
           )
         ) {
         | Some({ concealment: Domain.Unit_concealment_appearance.Concealed, _ }) =>
           <p className="concealment-status"> {text("Concealment active")} </p>
         | Some({ concealment: Domain.Unit_concealment_appearance.Exposed, _ }) =>
           <p className="concealment-status revealed"> {text("Exposed")} </p>
         | _ => React.null
         }}
        <p className="ability"> {text(details.ability.name)} </p>
        {eachList(details.ability.facts, fact =>
           <p key=fact className="ability-fact"> {text(fact)} </p>
         )}
        <p> {text(details.ability.description)} </p>
        {eachList(details.promotions, promotion =>
           <React.Fragment key={promotion.Domain.id}>
             <button className="primary" onClick={_ => actions.advance(promotion.id)}>
               {text(
                  "Advance to "
                  ++ promotion.name
                  ++ " · "
                  ++ number(Int32.to_float(promotion.hp))
                  ++ " HP",
                )}
             </button>
             <p> {text(promotion.description)} </p>
           </React.Fragment>
         )}
        <Arsenal
          view
          unit={details.unit}
          onDrop=?{
            Native.controllable(view.game, details.unit.id)
            && Native.state(view.game).turn == Domain.Turn_phase.Player
            && Native.state(view.game).winner == None
              ? Some(item => actions.drop(details.unit.id, item)) : None
          }
        />
        <div className="movement-costs">
          <h3> {text("Terrain effects")} </h3>
          <table className="weapon-table terrain-effects-table" ariaLabel="Unit terrain effects">
            <thead>
              <tr>
                <th scope="col"> {text("Terrain")} </th>
                <th scope="col"> {text("Cost")} </th>
                <th scope="col"> {text("Damage received")} </th>
              </tr>
            </thead>
            <tbody>
              {eachList(details.terrain_effects, terrain =>
                 <tr key={terrain.Presentation.terrain} className={terrain.class_name}>
                   <th scope="row"> {text(terrain.name)} </th>
                   <td
                     className="terrain-cost"
                     title=?{terrain.class_name == "blocked" ? Some("Impassable") : None}>
                     <b> {text(terrain.label)} </b>
                   </td>
                   <td>
                     {eachList(terrain.facts, fact =>
                        <span key={fact.Presentation.label} className="terrain-damage-fact">
                          <b
                            className={
                              fact.value < 0. ? "resistant" : fact.value > 0. ? "vulnerable" : ""
                            }>
                            {text(Presentation.signed_value(fact.value))}
                          </b>
                          {text(" ")}
                          <span className="terrain-damage-kind">
                            {text(fact.label == "" ? "Damage" : fact.label ++ " Damage")}
                          </span>
                        </span>
                      )}
                   </td>
                 </tr>
               )}
            </tbody>
          </table>
          <p>
            {text(
               "Cost is movement spent to enter. Damage is the change per hit, in addition to Damage received above.",
             )}
          </p>
        </div>
      </div>
    </div>;
};
module Forecast = {
  [@react.component]
  let make = (~forecast: Presentation.forecast) => {
    let props = ReactDOM.domProps(~className="attack-health " ++ side(forecast.side), ());
    dataForecast(props, side(forecast.side));
    ReactDOM.createElement(
      "span",
      ~props,
      [|
        <span className="forecast-label">
          <span>
            {text(number(forecast.current_hp) ++ " → ")}
            <b className={forecast.remaining_class}> {text(forecast.remaining_label)} </b>
            {text(" / " ++ number(forecast.maximum_hp) ++ " HP")}
          </span>
        </span>,
        <span className="health-track" role="img" ariaLabel={Utf8.to_js(forecast.health_label)}>
          <span
            className="forecast-loss"
            style={ReactDOM.Style.make(
              ~left=number(forecast.maximum_percent) ++ "%",
              ~width=
                number(max(0., forecast.current_percent -. forecast.maximum_percent)) ++ "%",
              (),
            )}
          />
          <span
            className="forecast-possible"
            style={ReactDOM.Style.make(
              ~left=number(min(forecast.minimum_percent, forecast.current_percent)) ++ "%",
              ~width=
                number(
                  min(forecast.maximum_percent, forecast.current_percent)
                  -. min(forecast.minimum_percent, forecast.current_percent),
                )
                ++ "%",
              (),
            )}
          />
          <span
            className="forecast-certain"
            style={percent(min(forecast.minimum_percent, forecast.current_percent))}
          />
          <span
            className="forecast-healing"
            style={ReactDOM.Style.make(
              ~left=number(forecast.current_percent) ++ "%",
              ~width=
                number(max(0., forecast.maximum_percent -. forecast.current_percent)) ++ "%",
              (),
            )}
          />
          <span
            className="forecast-current"
            style={ReactDOM.Style.make(~left=number(forecast.current_percent) ++ "%", ())}
            title={Utf8.to_js("Current HP: " ++ number(forecast.current_hp))}
          />
        </span>,
        <span className="forecast-impact"> <KeepTogether value={forecast.impact_label} /> </span>,
      |],
    );
  };
};
module Attack = {
  [@react.component]
  let make = (~choice: Presentation.attack, ~onAttack, ~busy=false) => {
    let (expanded, setExpanded) = React.useState(() => false);
    let id = "attack-details-" ++ choice.weapon.id;
    let props =
      ReactDOM.domProps(
        ~className="attack-action",
        ~ariaLabel=Utf8.to_js(choice.button_label),
        ~disabled=busy || choice.result == None,
        ~onClick=_ => onAttack(choice.weapon.id),
        (),
      );
    dataAttack(props, choice.weapon.id);
    <div className="attack-action-row">
      <div className="attack-box">
        {ReactDOM.createElement(
           "button",
           ~props,
           [|
             <span className={"attack-heading" ++ (choice.result != None ? " paired" : "")}>
               {eachList(
                  choice.result == None ? [choice.name] : [choice.name, choice.response_name],
                  name =>
                  <strong key=name> <KeepTogether value=name /> </strong>
                )}
               {choice.result == None
                  ? <span> {text("Out of reach or blocked")} </span> : React.null}
             </span>,
             option(choice.result, result =>
               <span className="attack-forecasts">
                 <Forecast forecast={result.Presentation.player} />
                 <Forecast forecast={result.enemy} />
               </span>
             ),
           |],
         )}
        {option(
           choice.result,
           _ => {
             let props =
               ReactDOM.domProps(
                 ~className="attack-inspect",
                 ~ariaLabel=
                   Utf8.to_js(
                     "Inspect "
                     ++ String.lowercase_ascii(choice.weapon.name)
                     ++ " damage calculation",
                   ),
                 ~ariaExpanded=expanded,
                 ~ariaControls=id,
                 ~dangerouslySetInnerHTML={ "__html": Art.icon_markup("search") },
                 ~onClick=_ => setExpanded(value => !value),
                 (),
               );
             dataInspect(props, choice.weapon.id);
             ReactDOM.createElement("button", ~props, [||]);
           },
         )}
      </div>
      {option(choice.result, result =>
         <div id className="combat-explanation" hidden={!expanded}>
           <div className="attack-calculation">
             <p> {text(result.Presentation.attack.description)} </p>
             <p className="damage-equation"> {text(result.attack.equation)} </p>
           </div>
           <div className="attack-calculation">
             <p> {text(result.retaliation.description)} </p>
             <p className="damage-equation"> {text(result.retaliation.equation)} </p>
           </div>
         </div>
       )}
    </div>;
  };
};
module CombatResult = {
  [@react.component]
  let make =
      (
        ~combat: Domain.combat_animation,
        ~entries: list(Presentation.exchange_entry),
        ~shown,
        ~complete,
      ) => {
    let (expanded, setExpanded) = React.useState(() => None);
    let units = combat.units;
    let player = units[0].side == Domain.Turn_phase.Player ? 0 : 1;
    let opponent = 1 - player;
    let finalHealth =
      Array.map(Int32.to_float, combat.strikes[Array.length(combat.strikes) - 1].Domain.health);
    let playerChange = finalHealth[player] -. Int32.to_float(units[player].hp);
    let opponentChange = finalHealth[opponent] -. Int32.to_float(units[opponent].hp);
    <ol className="exchange-log" role="log" ariaLabel="Combat exchange">
      {eachList(List.filter(entry => entry.Presentation.strike < shown, entries), entry =>
         <li key={entry.id} className={"exchange-event " ++ (entry.good ? "good" : "bad")}>
           <div className="exchange-line">
             <strong> {text(entry.text)} </strong>
             <button
               className="icon-button exchange-inspect"
               ariaLabel={Utf8.to_js("Inspect calculation: " ++ entry.text)}
               ariaExpanded={expanded == Some(entry.id)}
               ariaControls={"exchange-" ++ entry.id}
               dangerouslySetInnerHTML={ "__html": Art.icon_markup("search") }
               onClick={_ =>
                 setExpanded(previous => previous == Some(entry.id) ? None : Some(entry.id))
               }
             />
           </div>
           <div
             id={"exchange-" ++ entry.id}
             className="exchange-calculation"
             hidden={expanded != Some(entry.id)}>
             {eachList(entry.details, detail =>
                <p key=detail className="damage-equation"> {text(detail)} </p>
              )}
           </div>
         </li>
       )}
      {complete
         ? <>
             {finalHealth[opponent] == 0.
                ? <li className="exchange-outcome good"> {text("Your opponent died")} </li>
                : React.null}
             {finalHealth[player] == 0.
                ? <li className="exchange-outcome bad"> {text("You died")} </li> : React.null}
             {finalHealth[0] > 0. && finalHealth[1] > 0.
                ? <li className="exchange-outcome"> {text("Exchange ended")} </li> : React.null}
             <li className="exchange-totals">
               {playerChange > 0.
                  ? <span className="good">
                      {text("You healed +" ++ number(playerChange) ++ " HP")}
                    </span>
                  : <span className="bad">
                      {text("You took " ++ Presentation.loss(-. playerChange) ++ " HP")}
                    </span>}
               {opponentChange > 0.
                  ? <span className="bad">
                      {text("Opponent healed +" ++ number(opponentChange) ++ " HP")}
                    </span>
                  : <span className="good">
                      {text("You dealt " ++ Presentation.loss(-. opponentChange) ++ " HP")}
                    </span>}
             </li>
           </>
         : React.null}
    </ol>;
  };
};

module Unit = {
  [@react.component]
  let make =
      (~view: Hud_details.context, ~hud: Hud_details.t, ~actions: Session.actions, ~onNewBattle) => {
    let playback = Hud_details.report(hud);
    let busy =
      Option.fold(
        ~none=false,
        ~some=(report: Battle_details.report) => !report.complete,
        playback,
      );
    let game =
      Option.fold(
        ~none=view.game,
        ~some=(report: Battle_details.report) => report.game,
        playback,
      );
    let health =
      Option.bind(playback, report =>
        report.shown == 0
          ? None
          : Some(Array.map(Int32.to_float, report.combat.strikes[report.shown - 1].health))
      );
    let attack = weapon =>
      if (!busy) {
        actions.attack(weapon);
      };
    let panel = Hud_details.panel(hud);
    let element = React.useRef(Js.Nullable.null);
    React.useLayoutEffect1(
      () =>
        Js.Nullable.toOption(element.current)
        |> Option.map(panel => Viewport.reveal_attack(~panel)),
      [|Hud_details.reveal_key(hud)|],
    );
    <section
      ref={ReactDOM.Ref.domRef(element)}
      id="unit-panel"
      className={"panel" ++ (busy ? " resolving" : "")}
      ariaLabel="Selected unit"
      hidden={panel == Presentation.Hidden}>
      {switch (panel) {
       | Hidden => React.null
       | Outcome(title, description) =>
         <>
           <div className="unit-title"> <h2> {text(title)} </h2> </div>
           <p> {text(description)} </p>
           <button id="play-again" className="primary" onClick={_ => onNewBattle()}>
             {text("Play again")}
           </button>
         </>
       | Details(details) => <Details view actions details />
       | Combat(attacker, defender, reset) =>
         <>
           <div className="duel">
             <UnitCard view game unit=attacker hp=?{Option.map(health => health[0], health)} />
             <span className="duel-divider" ariaHidden=true> {text("⚔")} </span>
             <UnitCard view game unit=defender hp=?{Option.map(health => health[1], health)} />
           </div>
           {switch (playback) {
            | Some(playback) =>
              <CombatResult
                combat={playback.combat}
                entries={Presentation.exchange_entries(view.catalog, playback.combat)}
                shown={playback.shown}
                complete={playback.complete}
              />
            | None =>
              <div key=reset className="attack-actions" ariaLabel="Attack actions">
                {eachList(Presentation.attacks(view.catalog, game, attacker, defender), choice =>
                   <Attack key={choice.Presentation.weapon.id} choice onAttack=attack busy />
                 )}
              </div>
            }}
         </>
       }}
    </section>;
  };
};
module TerrainPreview = {
  [@react.component]
  let make = (~cache: Terrain_preview.t, ~frame: Art.frame) => {
    let host = React.useRef(Js.Nullable.null);
    let (loading, setLoading) = React.useState(() => false);
    let (error, setError) = React.useState(() => None);
    let (attempt, setAttempt) = React.useState(() => 0);
    React.useEffect2(
      () => {
        let active = ref(true);
        setError(_ => None);
        let show = image =>
          if (active.contents) {
            Option.iter(
              node => Terrain_preview.mount(node, image),
              Js.Nullable.toOption(host.current),
            );
            setLoading(_ => false);
          };
        switch (Terrain_preview.ready(cache, frame)) {
        | Some(image) => show(image)
        | None =>
          Option.iter(Browser_image.clear, Js.Nullable.toOption(host.current));
          setLoading(_ => true);
          let _ =
            Terrain_preview.load(cache, frame)
            |> Js.Promise.then_(image => {
                 show(image);
                 Js.Promise.resolve();
               })
            |> Js.Promise.catch(_ => {
                 if (active.contents) {
                   setLoading(_ => false);
                   setError(_ => Some("Terrain preview could not load: " ++ frame.url));
                 };
                 Js.Promise.resolve();
               });
          ();
        };
        Some(() => active.contents = false);
      },
      (frame, attempt),
    );
    <>
      <div className="preview-image" ref={ReactDOM.Ref.domRef(host)} ariaHidden=true />
      {loading
         ? <span className="preview-status" role="status"> {text("Loading preview…")} </span>
         : option(error, message =>
             <div className="preview-status" role="alert" title=message>
               {text("Preview could not load.")}
               <button
                 onClick={_ => {
                   let _ =
                     Terrain_preview.retry(cache, frame)
                     |> Js.Promise.then_(_ => Js.Promise.resolve())
                     |> Js.Promise.catch(_ => Js.Promise.resolve());
                   setAttempt(previous => previous + 1);
                 }}>
                 {text("Retry")}
               </button>
             </div>
           )}
    </>;
  };
};
module Terrain = {
  [@react.component]
  let make =
      (
        ~view: Hud_details.context,
        ~panel: option(Presentation.terrain_panel),
        ~actions: Session.actions,
        ~previews: Terrain_preview.t,
      ) => {
    let state = Native.state(view.game);
    <section
      id="terrain-panel" className="panel" ariaLabel="Selected terrain" hidden={panel == None}>
      {option(panel, panel =>
         <>
           {option(panel.Presentation.recruitment, recruitment =>
              <div className="terrain-recruitment">
                <h3> {text("Recruit here")} </h3>
                <div className="recruit-list">
                  {eachList(
                     recruitment.options,
                     recruit => {
                       let definition = recruit.Presentation.definition;
                       let props =
                         ReactDOM.domProps(
                           ~className="recruit-button",
                           ~title=Utf8.to_js(definition.description),
                           ~disabled=!recruit.enabled,
                           ~onClick=_ => actions.recruit(definition.id, panel.tile.id),
                           (),
                         );
                       dataRecruit(props, definition.id);
                       ReactDOM.createElement(
                         "button",
                         ~props,
                         [|
                           <span>
                             {text(definition.name)}
                             {state.campaign != None
                                ? <small className="recruit-preview">
                                    {text(
                                       number(Int32.to_float(definition.hp))
                                       ++ " HP   "
                                       ++ number(definition.move)
                                       ++ " move",
                                     )}
                                  </small>
                                : React.null}
                           </span>,
                           <span> {text(number(recruit.cost) ++ " gold")} </span>,
                         |],
                       );
                     },
                   )}
                </div>
                <p className="recruit-hint"> {text(recruitment.hint)} </p>
              </div>
            )}
           <div className="terrain-detail">
             <div className="terrain-portrait">
               {option(Art.terrain_portrait(view.artwork, panel.tile.terrain), frame =>
                  <TerrainPreview cache=previews frame />
                )}
               <span className="unit-side"> {text("Terrain")} </span>
             </div>
             <div className="terrain-info">
               <h2> {text(panel.name)} </h2>
               {option(panel.ownership, ((label, className)) =>
                  <p className={"terrain-owner " ++ className}> {text(label)} </p>
                )}
               <p> {text(panel.description)} </p>
               <ul className="terrain-facts" role="list">
                 {eachList(panel.facts, fact => <li key=fact> {text(fact)} </li>)}
               </ul>
             </div>
           </div>
         </>
       )}
    </section>;
  };
};

module Campaign = {
  [@react.component]
  let make =
      (
        ~view: Session.model,
        ~actions: Session.actions,
        ~showResult,
        ~expanded,
        ~onCollapse,
        ~onNewCampaign,
      ) => {
    let state = Native.state(view.game);
    let conversations = React.useRef(Js.Nullable.null);
    let (recall, setRecall) = React.useState(() => []);
    React.useEffect2(
      () => {
        setRecall(_ => []);
        None;
      },
      (state.map_id, state.winner),
    );
    <>
      {option(
         state.campaign,
         campaign => {
           let definition = Catalog.campaign(view.catalog, campaign.Domain.id);
           let chapter = Story.mission(view.catalog, view.game, campaign);
           let mission = chapter.Story.source;
           let layout = Catalog.mission_map(view.catalog, mission);
           let survivors =
             filterList(
               (unit: Domain.public_unit) => {
                 unit.side == Domain.Turn_phase.Player
                 && unit.role != Some(Domain.Role.Leader)
                 && unit.role != Some(Domain.Role.Courier)
                 && !
                      Array.exists(
                        name => Some(name) == unit.character,
                        mission.required_companions,
                      )
                 && !Story.returns_with_story(view.catalog, campaign, unit)
               },
               Array.to_list(state.units),
             );
           let toggle = id =>
             setRecall(previous =>
               List.exists(value => value == id, previous)
                 ? filterList(value => value != id, previous)
                 : List.length(previous) < 4 ? List.append(previous, [id]) : previous
             );
           <>
             <section
               id="mission-overview"
               className="campaign-panel mission-overview"
               hidden={!expanded}
               ariaLabel="Mission briefing">
               <h2> {text(layout.name)} </h2>
               <p className="mission-heading">
                 {text(
                    definition.name
                    ++ " "
                    ++ number(campaign.mission +. 1.)
                    ++ "/"
                    ++ string_of_int(Array.length(definition.missions)),
                  )}
               </p>
               <h3> {text("Briefing")} </h3>
               <p>
                 {text(chapter.briefing ++ " ")}
                 <button
                   className="conversation-icon"
                   ariaLabel="Campaign conversations"
                   title="Campaign conversations"
                   onClick={_ => show(conversations)}>
                   <svg viewBox="0 0 24 24" ariaHidden=true focusable="false">
                     <path d="M14 3H4a2 2 0 0 0-2 2v11l4-4h8a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2Z" />
                     <path d="M8 16v1a2 2 0 0 0 2 2h8l4 3V11a2 2 0 0 0-2-2" />
                   </svg>
                 </button>
               </p>
               {chapter.hints == []
                  ? React.null
                  : <>
                      <h3> {text("Hints")} </h3>
                      <ul className="mission-items">
                        {eachList(chapter.hints, hint =>
                           <li key=hint> <span> {text(hint)} </span> </li>
                         )}
                      </ul>
                    </>}
               <h3> {text("Objectives")} </h3>
               <ul className="mission-items mission-goals">
                 {eachList(
                    Story.goal_statuses(view.game),
                    status => {
                      let progress = status.Domain.progress;
                      <li
                        key={progress.label}
                        className={
                          (progress.done_ ? "completed-goal " : "")
                          ++ (status.failed ? "failed-goal" : "")
                        }
                        ariaLabel=?{
                          status.failed
                            ? Some(Utf8.to_js(progress.label ++ ": failed"))
                            : progress.done_
                                ? Some(Utf8.to_js(progress.label ++ ": completed")) : None
                        }>
                        <span> {text(progress.label)} </span>
                        {progress.total > 1.
                           ? <strong>
                               {text(number(progress.current) ++ "/" ++ number(progress.total))}
                             </strong>
                           : React.null}
                      </li>;
                    },
                  )}
               </ul>
               {Story.optional_statuses(view.game) == []
                  ? React.null
                  : <>
                      <h3> {text("Optional objectives")} </h3>
                      <ul className="mission-items mission-goals">
                        {eachList(
                           Story.optional_statuses(view.game),
                           status => {
                             let progress = status.Domain.progress;
                             <li
                               key={progress.label}
                               className={
                                 (progress.done_ ? "completed-goal " : "")
                                 ++ (status.failed ? "failed-goal" : "")
                               }
                               ariaLabel=?{
                                 status.failed
                                   ? Some(Utf8.to_js(progress.label ++ ": failed"))
                                   : progress.done_
                                       ? Some(Utf8.to_js(progress.label ++ ": completed")) : None
                               }>
                               <span> {text(progress.label)} </span>
                               {progress.total > 1.
                                  ? <strong>
                                      {text(
                                         number(progress.current)
                                         ++ "/"
                                         ++ number(progress.total),
                                       )}
                                    </strong>
                                  : React.null}
                             </li>;
                           },
                         )}
                      </ul>
                    </>}
             </section>
             {showResult
              && state.winner == Some(Domain.Battle_result.Player_victory)
              && !Story.dialogue_pending(state.campaign)
                ? <section
                    id="mission-result"
                    className="campaign-panel"
                    ariaLabel="Mission result"
                    tabIndex=(-1)>
                    <h2> {text(layout.name)} </h2>
                    <div className="mission-debrief">
                      <h3>
                        {text(campaign.completed ? "Campaign complete" : "The company regroups")}
                      </h3>
                      <p>
                        {text(
                           Option.value(
                             ~default=Option.value(~default="", campaign.outcome),
                             chapter.debrief,
                           ),
                         )}
                      </p>
                      {!campaign.completed
                         ? <>
                             <p>
                               {text(
                                  "Choose up to four survivors for the next mission. They return at full health. Carrying "
                                  ++ number(min(8., Native.gold(state.gold, "player")))
                                  ++ " gold.",
                                )}
                             </p>
                             <div className="recall-list">
                               {eachList(
                                  survivors,
                                  unit => {
                                    let checked = List.exists(id => id == unit.Domain.id, recall);
                                    let rule = Catalog.unit(view.catalog, unit.type_);
                                    <label
                                      key={unit.id}
                                      className={"recall-choice " ++ (checked ? "selected" : "")}>
                                      <input
                                        type_="checkbox"
                                        checked
                                        disabled={!checked && List.length(recall) >= 4}
                                        onChange={_ => toggle(unit.id)}
                                      />
                                      <Sprite
                                        artwork={view.artwork}
                                        name={unit.type_}
                                        width=36.
                                        height=44.
                                      />
                                      <span>
                                        <strong>
                                          {text(
                                             Option.value(
                                               ~default=rule.name,
                                               Native.unit_name(view.game, unit.id),
                                             ),
                                           )}
                                        </strong>
                                        <span>
                                          {text(
                                             number(Int32.to_float(unit.hp))
                                             ++ " / "
                                             ++ number(Int32.to_float(rule.hp))
                                             ++ " HP",
                                           )}
                                        </span>
                                        <span>
                                          {text(
                                             "Level "
                                             ++ number(rule.level)
                                             ++ " · "
                                             ++ (
                                               switch (rule.experience_needed) {
                                               | None => "Fully advanced"
                                               | Some(needed) =>
                                                 number(
                                                   Option.value(~default=0., unit.experience),
                                                 )
                                                 ++ " / "
                                                 ++ number(needed)
                                                 ++ " XP"
                                               }
                                             ),
                                           )}
                                        </span>
                                      </span>
                                    </label>;
                                  },
                                )}
                             </div>
                             <button
                               className="primary"
                               onClick={_ => {
                                 actions.next_mission(
                                   filterList(
                                     id =>
                                       List.exists(
                                         (unit: Domain.public_unit) => unit.id == id,
                                         survivors,
                                       ),
                                     recall,
                                   ),
                                 );
                                 onCollapse();
                               }}>
                               {text(
                                  "Continue to "
                                  ++ (
                                    switch (
                                      definition.missions[int_of_float(campaign.mission) + 1]
                                    ) {
                                    | mission => Catalog.mission_map(view.catalog, mission).name
                                    }
                                  ),
                                )}
                             </button>
                           </>
                         : <button className="primary" onClick={_ => onNewCampaign()}>
                             {text("Choose another campaign")}
                           </button>}
                    </div>
                  </section>
                : React.null}
           </>;
         },
       )}
      <Dialog_window
        title="Campaign conversations"
        dialog=conversations
        width="680px"
        tall=true
        releaseControls=true>
        {switch (state.campaign) {
         | Some(campaign) when campaign.dialogue_index > 0. =>
           each(
             Array.sub(
               campaign.dialogue,
               0,
               min(Array.length(campaign.dialogue), int_of_float(campaign.dialogue_index)),
             ),
             line =>
             <p> <strong> {text(line.Domain.speaker)} </strong> <br /> {text(line.text)} </p>
           )
         | _ => <p> {text("No conversations yet.")} </p>
         }}
      </Dialog_window>
    </>;
  };
};

module Defeat = {
  [@react.component]
  let make = (~view: Session.model, ~actions: Session.actions, ~onNewGame) => {
    let state = Native.state(view.game);
    let resolving =
      Option.fold(
        ~none=false,
        ~some=(frame: Combat_playback.frame) => !frame.complete,
        view.combat_playback,
      );
    state.winner != Some(Domain.Battle_result.Player_defeat)
    || Story.dialogue_pending(state.campaign)
    || resolving
      ? React.null
      : {
        let campaign = state.campaign != None;
        let reason =
          Option.fold(
            ~none="Your commander has fallen.",
            ~some=
              (campaign: Domain.campaign_state) =>
                Option.value(~default="The company cannot continue.", campaign.outcome),
            state.campaign,
          );
        <section id="battle-result" className="battle-result" ariaLabel="Battle result">
          <p className="result-context">
            {text(Catalog.map(view.catalog, state.map_id).name)}
          </p>
          <h2> {text("Defeat")} </h2>
          <p className="result-reason"> {text(reason)} </p>
          <div className="result-actions">
            <button
              id={campaign ? "retry-mission" : "retry-scenario"}
              className="primary"
              onClick={_ => actions.retry()}>
              {text(campaign ? "Retry mission" : "Retry scenario")}
            </button>
            <button id="choose-another-game" onClick={_ => onNewGame()}>
              {text("Choose another game")}
            </button>
          </div>
          {campaign
             ? <p className="result-hint">
                 {text("You can also return to an earlier turn or mission from Saved games.")}
               </p>
             : React.null}
        </section>;
      };
  };
};
