open Ui;

module UnitCard = {
  [@react.component]
  let make = (~view: Session.model, ~unit: Domain.public_unit, ~showName=false) => {
    let definition = Catalog.unit(view.catalog, unit.type_);
    <div className={"combatant " ++ side(unit.side)}>
      <span className="unit-side">
        {text(unit.side == Domain.side_player ? "Your unit" : "Enemy unit")}
      </span>
      <Sprite
        artwork={view.artwork}
        name={unit.type_}
        side={side(unit.side)}
        width=72.
        height=90.
        className={"portrait " ++ (unit.side == Domain.side_enemy ? "enemy-unit" : "")}
      />
      <h2>
        {text(showName ? Option.value(~default=definition.name, unit.name) : definition.name)}
      </h2>
      {Option.value(~default=false, unit.retreating)
         ? <span className="retreat-label"> {text("Retreating")} </span> : React.null}
      {showName && unit.name != None
         ? <span className="named-unit-type"> {text(definition.name)} </span> : React.null}
      <div className="health-numbers">
        <b> {num(unit.hp)} </b>
        <span> {text(" / " ++ number(definition.hp) ++ " HP")} </span>
      </div>
      <div
        className="health-track"
        role="img"
        ariaLabel={number(unit.hp) ++ " of " ++ number(definition.hp) ++ " current health"}>
        <span className="health-left" style={percent(unit.hp /. definition.hp *. 100.)} />
      </div>
    </div>;
  };
};
module Arsenal = {
  [@react.component]
  let make = (~view: Session.model, ~unit: Domain.public_unit) => {
    let definition = Catalog.unit(view.catalog, unit.type_);
    <>
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
            {each(definition.attacks, weapon =>
               <tr key={weapon.Domain.id}>
                 <th scope="row">
                   <span className="weapon-name">
                     <Icon name=(weapon.damageType :> string) />
                     <strong> {text(weapon.name)} </strong>
                   </span>
                   <span className="weapon-detail weapon-category">
                     {text(weapon.category == Domain.category_melee ? "Melee" : "Ranged")}
                   </span>
                 </th>
                 <td className="weapon-damage">
                   <b> {num(weapon.damage)} </b>
                   {text(" " ++ Presentation.damage_name(weapon.damageType))}
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
          {each(Presentation.armor(view.catalog, unit), modifier =>
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
      (~view: Session.model, ~actions: Session.actions, ~details: Presentation.unit_details) =>
    <div className="unit-detail">
      <UnitCard view unit={details.unit} showName=true />
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
        <p className="ability"> {text(details.ability.name)} </p>
        {each(details.ability.facts, fact =>
           <p key=fact className="ability-fact"> {text(fact)} </p>
         )}
        <p> {text(details.ability.description)} </p>
        <p>
          {text(
             "Level "
             ++ number(Option.value(~default=1., details.definition.level))
             ++ {js| · |js}
             ++ (
               switch (Js.Nullable.toOption(details.definition.experienceNeeded)) {
               | None => "Fully advanced"
               | Some(needed) =>
                 number(Option.value(~default=0., details.unit.experience))
                 ++ " / "
                 ++ number(needed)
                 ++ " XP"
               }
             ),
           )}
        </p>
        {Js.Nullable.toOption(details.definition.experienceNeeded) != None
           ? <p>
               {text(
                  "Earn experience by dealing damage and defeating enemies. Advance while ready; wounds carry over and advancement uses this turn.",
                )}
             </p>
           : React.null}
        {each(details.promotions, promotion =>
           <React.Fragment key={promotion.Domain.id}>
             <button className="primary" onClick={_ => actions.advance(promotion.id)}>
               {text(
                  "Advance to " ++ promotion.name ++ {js| · |js} ++ number(promotion.hp) ++ " HP",
                )}
             </button>
             <p> {text(promotion.description)} </p>
           </React.Fragment>
         )}
        <Arsenal view unit={details.unit} />
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
              {each(details.terrain_effects, terrain =>
                 <tr key={terrain.Presentation.terrain} className={terrain.class_name}>
                   <th scope="row"> {text(terrain.name)} </th>
                   <td className="terrain-cost"> <b> {text(terrain.label)} </b> </td>
                   <td>
                     {each(terrain.facts, fact =>
                        <span key={fact.Presentation.label} className="terrain-damage-fact">
                          <b
                            className={
                              fact.value < 0. ? "resistant" : fact.value > 0. ? "vulnerable" : ""
                            }>
                            {text(Presentation.signed_value(fact.value))}
                          </b>
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
            {text(number(forecast.current_hp) ++ {js| → |js})}
            <b className={forecast.remaining_hp <= 0. ? "fatal" : ""}>
              {num(forecast.remaining_hp)}
            </b>
            {text(" / " ++ number(forecast.maximum_hp) ++ " HP")}
          </span>
        </span>,
        <span className="health-track" role="img" ariaLabel={forecast.health_label}>
          <span className="health-lost" style={percent(forecast.current_percent)} />
          <span className="health-left" style={percent(forecast.remaining_percent)} />
        </span>,
        <span className="forecast-impact"> {text(forecast.impact_label)} </span>,
      |],
    );
  };
};
module Attack = {
  [@react.component]
  let make = (~choice: Presentation.attack, ~actions: Session.actions) => {
    let (expanded, setExpanded) = React.useState(() => false);
    let id = "attack-details-" ++ choice.weapon.id;
    let props =
      ReactDOM.domProps(
        ~className="attack-action",
        ~ariaLabel=choice.button_label,
        ~disabled=choice.result == None,
        ~onClick=_ => actions.attack(choice.weapon.id),
        (),
      );
    dataAttack(props, choice.weapon.id);
    <div className="attack-action-row">
      <div className="attack-box">
        {ReactDOM.createElement(
           "button",
           ~props,
           [|
             <span className="attack-heading">
               <strong> {text(choice.weapon.name)} </strong>
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
                   "Inspect "
                   ++ Js.String.toLowerCase(choice.weapon.name)
                   ++ " damage calculation",
                 ~ariaExpanded=expanded,
                 ~ariaControls=id,
                 ~onClick=_ => setExpanded(value => !value),
                 (),
               );
             dataInspect(props, choice.weapon.id);
             ReactDOM.createElement("button", ~props, [|<Icon name="search" />|]);
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
module Unit = {
  [@react.component]
  let make = (~view: Session.model, ~actions: Session.actions, ~onNewBattle) => {
    let panel =
      Presentation.unit_panel(
        view.catalog,
        view.game,
        ~selected=view.selected,
        ~target=view.target,
        ~revision=float_of_int(view.revision),
      );
    <section
      id="unit-panel"
      className="panel"
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
             <UnitCard view unit=attacker />
             <span className="duel-divider" ariaHidden=true> {text({js|⚔|js})} </span>
             <UnitCard view unit=defender />
           </div>
           <div key=reset className="attack-actions">
             <h3> {text("Attack")} </h3>
             {each(Presentation.attacks(view.catalog, view.game, attacker, defender), choice =>
                <Attack key={choice.Presentation.weapon.id} choice actions />
              )}
           </div>
         </>
       }}
    </section>;
  };
};
module Terrain = {
  [@react.component]
  let make = (~view: Session.model, ~actions: Session.actions) => {
    let panel =
      Presentation.terrain_panel(
        view.catalog,
        view.game,
        ~selected=view.selected,
        ~inspected=view.inspected,
      );
    let state = Native.state(view.game);
    <section
      id="terrain-panel" className="panel" ariaLabel="Selected terrain" hidden={panel == None}>
      {option(panel, panel =>
         <>
           {option(panel.Presentation.recruitment, recruitment =>
              <div className="terrain-recruitment">
                <h3> {text("Recruit here")} </h3>
                <div className="recruit-list">
                  {each(
                     recruitment.options,
                     recruit => {
                       let definition = recruit.Presentation.definition;
                       let props =
                         ReactDOM.domProps(
                           ~className="recruit-button",
                           ~title=definition.description,
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
                                       number(definition.hp)
                                       ++ " HP   "
                                       ++ number(definition.move)
                                       ++ " move",
                                     )}
                                  </small>
                                : React.null}
                           </span>,
                           <span> {text(number(definition.cost) ++ " gold")} </span>,
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
               <Sprite
                 artwork={view.artwork}
                 kind="terrain"
                 name={Terrain_plan.layout_name(panel.tile, state.mapId)}
                 floor={panel.tile.floor}
                 width=80.
                 height=80.
                 className="terrain-card-art"
               />
               <span className="unit-side"> {text("Terrain")} </span>
             </div>
             <div className="terrain-info">
               <h2> {text(panel.name)} </h2>
               {option(panel.ownership, ((label, className)) =>
                  <p className={"terrain-owner " ++ className}> {text(label)} </p>
                )}
               <p> {text(panel.description)} </p>
               <ul className="terrain-facts" role="list">
                 {each(panel.facts, fact => <li key=fact> {text(fact)} </li>)}
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
      (~view: Session.model, ~actions: Session.actions, ~expanded, ~onCollapse, ~onNewCampaign) => {
    let state = Native.state(view.game);
    let (recall, setRecall) = React.useState(() => [||]);
    React.useEffect2(
      () => {
        setRecall(_ => [||]);
        None;
      },
      (state.mapId, state.winner),
    );
    option(
      state.campaign,
      campaign => {
        let definition = Catalog.campaign(view.catalog, campaign.Domain.id);
        let chapter = Story.mission(view.catalog, campaign);
        let mission = chapter.Story.source;
        let layout = Catalog.mission_map(view.catalog, mission);
        let survivors =
          filter(
            (unit: Domain.public_unit) => {
              let rule = Catalog.unit(view.catalog, unit.Domain.type_);
              unit.side == Domain.side_player
              && !Option.value(~default=false, rule.leader)
              && unit.role != Some(Domain.role_courier)
              && !
                   Array.exists(
                     name => Some(name) == unit.name,
                     Option.value(~default=[||], mission.requiredCompanions),
                   )
              && !Story.returns_with_story(view.catalog, campaign, unit);
            },
            state.units,
          );
        let toggle = id =>
          setRecall(previous =>
            Array.exists(value => value == id, previous)
              ? filter(value => value != id, previous)
              : Array.length(previous) < 4 ? Array.append(previous, [|id|]) : previous
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
            <p> {text(chapter.briefing)} </p>
            <h3> {text("Hints")} </h3>
            <ul className="mission-items">
              {each(chapter.hints, hint => <li key=hint> <span> {text(hint)} </span> </li>)}
            </ul>
            <h3> {text("Objectives")} </h3>
            <ul className="mission-items mission-goals">
              {each(
                 mission.goals,
                 goal => {
                   let progress = Story.progress(view.catalog, view.game, goal);
                   <li key={progress.label} className={progress.done_ ? "completed-goal" : ""}>
                     <span> {text(progress.label)} </span>
                     <strong>
                       {text(number(progress.current) ++ "/" ++ number(progress.total))}
                     </strong>
                   </li>;
                 },
               )}
              <li
                className={
                  Js.Nullable.toOption(state.winner) == Some(Domain.side_player)
                    ? "completed-goal" : ""
                }>
                <span>
                  {text(
                     "Keep your leader alive"
                     ++ (
                       Array.exists(
                         goal =>
                           switch (goal) {
                           | Domain.Goal_escort(_) => true
                           | _ => false
                           },
                         mission.goals,
                       )
                       && Array.length(Option.value(~default=[||], mission.requiredCompanions))
                       == 0
                         ? " and protect the courier." : "."
                     ),
                   )}
                </span>
              </li>
              {each(Option.value(~default=[||], mission.requiredCompanions), name =>
                 <li key=name> <span> {text("Keep " ++ name ++ " alive.")} </span> </li>
               )}
              {each(
                 filter(
                   (event: Domain.event) =>
                     event.label != None
                     && !
                          Array.exists(
                            goal =>
                              switch (goal) {
                              | Domain.Goal_event(value) => value.event == event.id
                              | _ => false
                              },
                            mission.goals,
                          ),
                   mission.events,
                 ),
                 event =>
                 <li
                   key={event.Domain.id}
                   className={
                     Array.exists(mark => mark == "event:" ++ event.id, campaign.marks)
                       ? "completed-goal" : ""
                   }>
                   <span>
                     {text(
                        Option.value(~default="", event.label)
                        ++ (
                          switch (event.gold) {
                          | None => ""
                          | Some(gold) => " (+" ++ number(gold) ++ " gold)"
                          }
                        ),
                      )}
                     <span className="optional-label"> {text("[optional]")} </span>
                   </span>
                 </li>
               )}
            </ul>
          </section>
          {Js.Nullable.toOption(state.winner) != None && !Story.dialogue_pending(state.campaign)
             ? <section
                 id="mission-result"
                 className="campaign-panel"
                 ariaLabel="Mission result"
                 tabIndex=(-1)>
                 <h2> {text(layout.name)} </h2>
                 <div className="mission-debrief">
                   <h3>
                     {text(
                        Js.Nullable.toOption(state.winner) == Some(Domain.side_enemy)
                          ? "The company must try again"
                          : campaign.completed ? "Campaign complete" : "The company regroups",
                      )}
                   </h3>
                   <p>
                     {text(
                        Js.Nullable.toOption(state.winner) == Some(Domain.side_player)
                          ? Option.value(
                              ~default=
                                Option.value(
                                  ~default="",
                                  Js.Nullable.toOption(campaign.outcome),
                                ),
                              chapter.debrief,
                            )
                          : Option.value(~default="", Js.Nullable.toOption(campaign.outcome)),
                      )}
                   </p>
                   {Js.Nullable.toOption(state.winner) == Some(Domain.side_player)
                    && !campaign.completed
                      ? <>
                          <p>
                            {text(
                               "Choose up to four survivors for the next mission. They return at full health. Carrying "
                               ++ number(min(8., state.gold.player))
                               ++ " gold.",
                             )}
                          </p>
                          <div className="recall-list">
                            {each(
                               survivors,
                               unit => {
                                 let checked = Array.exists(id => id == unit.Domain.id, recall);
                                 let rule = Catalog.unit(view.catalog, unit.type_);
                                 <label
                                   key={unit.id}
                                   className={"recall-choice " ++ (checked ? "selected" : "")}>
                                   <input
                                     type_="checkbox"
                                     checked
                                     disabled={!checked && Array.length(recall) >= 4}
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
                                       {text(Option.value(~default=rule.name, unit.name))}
                                     </strong>
                                     <span>
                                       {text(
                                          number(unit.hp) ++ " / " ++ number(rule.hp) ++ " HP",
                                        )}
                                     </span>
                                     <span>
                                       {text(
                                          "Level "
                                          ++ number(Option.value(~default=1., rule.level))
                                          ++ {js| · |js}
                                          ++ (
                                            switch (Js.Nullable.toOption(rule.experienceNeeded)) {
                                            | None => "Fully advanced"
                                            | Some(needed) =>
                                              number(Option.value(~default=0., unit.experience))
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
                                filter(
                                  id =>
                                    Array.exists(
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
                                 switch (definition.missions[int_of_float(campaign.mission) + 1]) {
                                 | mission => Catalog.mission_map(view.catalog, mission).name
                                 }
                               ),
                             )}
                          </button>
                        </>
                      : Js.Nullable.toOption(state.winner) == Some(Domain.side_enemy)
                          ? <button
                              className="primary"
                              onClick={_ => {
                                actions.retry();
                                onCollapse();
                              }}>
                              {text("Retry mission")}
                            </button>
                          : <button className="primary" onClick={_ => onNewCampaign()}>
                              {text("Choose another campaign")}
                            </button>}
                 </div>
               </section>
             : React.null}
        </>;
      },
    );
  };
};
