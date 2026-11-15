open Ui;
module KeepTogether = {
  [@react.component]
  let make = (~value) => {
    let parts =
      Js.String.splitByRe(
        ~regexp=Js.Re.fromStringWithFlags({js|(\\([^)]*\\))|js}, ~flags={js|g|js}),
        value,
      );
    each(
      parts,
      part => {
        let part = Option.value(~default={js||js}, part);
        Js.String.length(part) > 0 && Js.String.get(part, 0) == {js|(|js}
          ? <span className={js|keep-together|js}> {text(part)} </span> : text(part);
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
    let progress = {
      let js_text_part_3 =
        switch (definition.experience_needed) {
        | Some(needed) when needed > 0. =>
          let js_text_part_0 = number(Option.value(~default=0., unit.experience));
          let js_text_part_2 = number(needed);
          {j|$(js_text_part_0)/$(js_text_part_2) XP|j};
        | _ => {js|Max|js}
        };
      {j|L$(level) · $(js_text_part_3)|j};
    };
    let hostile =
      Option.fold(
        ~none=false,
        ~some=(f: Domain.unit_faction) => f.relation == Domain.Relation.Hostile,
        faction,
      );
    let controllable =
      Option.fold(~none=false, ~some=(f: Domain.unit_faction) => f.controllable, faction);
    <div
      className={
                  let js_text_part_1 = hostile ? {js|enemy|js} : {js|player|js};
                  {j|combatant $(js_text_part_1)|j};
                }
      role={js|group|js}
      ariaLabel={
        Option.fold(
          ~none=false,
          ~some=(faction: Domain.unit_faction) => faction.relation == Domain.Relation.Neutral,
          faction,
        )
          ? {js|Neutral unit|js}
          : controllable ? {js|Your unit|js} : hostile ? {js|Enemy unit|js} : {js|Allied unit|js}
      }>
      <Sprite
        artwork={view.artwork}
        name={unit.type_}
        side={Option.fold(
          ~none={js|passive|js},
          ~some=(faction: Domain.unit_faction) => faction.palette,
          faction,
        )}
        width=72.
        height=90.
        className={
                    let js_text_part_1 = hostile ? {js|enemy-unit|js} : {js||js};
                    {j|portrait $(js_text_part_1)|j};
                  }
      />
      <div className={js|unit-heading|js}>
        <h2>
          {text(
             showName
               ? Option.value(~default=definition.name, Native.unit_name(view.game, unit.id))
               : definition.name,
           )}
        </h2>
        {showProgress
           ? <span className={js|unit-progress|js} title={j|Level $(level)|j}>
               {text(progress)}
             </span>
           : React.null}
      </div>
      {Option.value(~default=false, unit.retreating)
         ? <span className={js|retreat-label|js}> {text({js|Retreating|js})} </span> : React.null}
      {showName && Native.unit_name(view.game, unit.id) != None
         ? <span className={js|named-unit-type|js}> {text(definition.name)} </span> : React.null}
      <div className={js|health-numbers|js}>
        <b> {num(health)} </b>
        <span>
          {text(
             {
               let js_text_part_1 = number(Int32.to_float(definition.hp));
               {j| / $(js_text_part_1) HP|j};
             },
           )}
        </span>
      </div>
      <div
        className={js|health-track|js}
        role={js|img|js}
        ariaLabel={
                    let js_text_part_0 = number(health);
                    let js_text_part_2 = number(Int32.to_float(definition.hp));
                    {j|$(js_text_part_0) of $(js_text_part_2) current health|j};
                  }>
        <span
          className={js|health-left|js}
          style={percent(health /. Int32.to_float(definition.hp) *. 100.)}
        />
      </div>
    </div>;
  };
};
module ItemList = {
  [@react.component]
  let make = (~items: array(Domain.item), ~artwork: Art.t, ~onDrop: option(string => unit)=?) =>
    <ul className={js|item-list|js}>
      {items
       |> Array.mapi((index, item: Domain.item) =>
            <li
              key={
                    let js_text_part_0 = item.id;
                    let js_text_part_2 = string_of_int(index);
                    {j|$(js_text_part_0)-$(js_text_part_2)|j};
                  }
              className={js|item-row|js}>
              <span className={js|item-icon|js} ariaHidden=true>
                {option(
                   Art.find(
                     artwork,
                     {
                       let js_text_part_1 = item.id;
                       {j|terrain_prop_items_$(js_text_part_1)|j};
                     },
                   ),
                   frame =>
                   <img
                     className={js|pixel-art|js}
                     src={frame.url}
                     alt={js||js}
                     width={js|64|js}
                     height={js|64|js}
                   />
                 )}
              </span>
              <div className={js|item-copy|js}>
                <div className={js|item-heading|js}>
                  <strong> {text(item.name)} </strong>
                  {option(onDrop, drop =>
                     <button
                       className={js|item-drop|js}
                       ariaLabel={
                                   let js_text_part_1 = item.name;
                                   {j|Drop $(js_text_part_1)|j};
                                 }
                       onClick={_ => drop(item.id)}>
                       {text({js|Drop|js})}
                     </button>
                   )}
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
  let make = (~artwork: Art.t, ~items: array(Domain.item)) =>
    <section
      id={js|ground-items-panel|js}
      className={js|panel|js}
      ariaLabel={js|Items on the ground|js}
      hidden={Array.length(items) == 0}>
      <h2> {text({js|On the ground|js})} </h2>
      <ItemList items artwork />
    </section>;
};
module Arsenal = {
  [@react.component]
  let make =
      (~view: Hud_details.context, ~unit: Domain.public_unit, ~onDrop: option(string => unit)=?) =>
    <>
      {Array.length(unit.items) == 0
         ? React.null
         : <section className={js|equipment|js} ariaLabel={js|Equipment|js}>
             <h3> {text({js|Equipment|js})} </h3>
             <ItemList items={unit.items} artwork={view.artwork} ?onDrop />
           </section>}
      <div className={js|arsenal|js}>
        <h3> {text({js|Attacks|js})} </h3>
        <table className={js|weapon-table|js} ariaLabel={js|Unit attacks|js}>
          <thead>
            <tr>
              <th scope={js|col|js}> {text({js|Attack|js})} </th>
              <th scope={js|col|js}> {text({js|Damage|js})} </th>
              <th scope={js|col|js}> {text({js|Range|js})} </th>
            </tr>
          </thead>
          <tbody>
            {eachList(Presentation.weapons(view.catalog, unit), weapon =>
               <tr key={weapon.Domain.id}>
                 <th scope={js|row|js}>
                   <span className={js|weapon-name|js}>
                     <Icon name={Enum.string_of_damage_type(weapon.damage_type)} />
                     <strong> {text(weapon.name)} </strong>
                   </span>
                   <span className={js|weapon-detail weapon-category|js}>
                     {text(
                        weapon.category == Domain.Category.Melee ? {js|Melee|js} : {js|Ranged|js},
                      )}
                   </span>
                 </th>
                 <td className={js|weapon-damage|js}>
                   <b>
                     {text(
                        {
                          let js_text_part_0 = number(float_of_int(weapon.count));
                          let js_text_part_2 = Presentation.weapon_damage(weapon);
                          {j|$(js_text_part_0) × $(js_text_part_2)|j};
                        },
                      )}
                   </b>
                   <span className={js|weapon-detail|js}>
                     {text(Presentation.damage_name(weapon.damage_type))}
                   </span>
                 </td>
                 <td> {text(Presentation.weapon_range(weapon))} </td>
               </tr>
             )}
          </tbody>
        </table>
      </div>
      <div className={js|resistances|js}>
        <h3> {text({js|Damage received|js})} </h3>
        <div className={js|resistance-list|js}>
          {eachList(Presentation.armor(view.catalog, unit), modifier =>
             <span
               key={modifier.Presentation.name}
               className={
                           let js_text_part_1 = modifier.class_name;
                           {j|resistance $(js_text_part_1)|j};
                         }>
               <span> {text(modifier.name)} </span>
               <b> {text(modifier.label)} </b>
             </span>
           )}
        </div>
      </div>
    </>;
};
module Details = {
  [@react.component]
  let make =
      (~view: Hud_details.context, ~actions: Session.actions, ~details: Presentation.unit_details) =>
    <div className={js|unit-detail|js}>
      <UnitCard view unit={details.unit} showName=true showProgress=true />
      <div className={js|unit-info|js}>
        <div className={js|unit-stats|js}>
          <span className={js|unit-stat|js}>
            <Icon name={js|move|js} />
            <span> {text({js|Move: |js})} <b> {num(details.definition.move)} </b> </span>
          </span>
          <p className={js|unit-state|js}>
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
           <p className={js|concealment-status|js}> {text({js|Concealment active|js})} </p>
         | Some({ concealment: Domain.Unit_concealment_appearance.Exposed, _ }) =>
           <p className={js|concealment-status revealed|js}> {text({js|Exposed|js})} </p>
         | _ => React.null
         }}
        <p className={js|ability|js}> {text(details.ability.name)} </p>
        {eachList(details.ability.facts, fact =>
           <p key=fact className={js|ability-fact|js}> {text(fact)} </p>
         )}
        <p> {text(details.ability.description)} </p>
        {eachList(details.promotions, promotion =>
           <React.Fragment key={promotion.Domain.id}>
             <button className={js|primary|js} onClick={_ => actions.advance(promotion.id)}>
               {text(
                  {
                    let js_text_part_1 = promotion.name;
                    let js_text_part_3 = number(Int32.to_float(promotion.hp));
                    {j|Advance to $(js_text_part_1) · $(js_text_part_3) HP|j};
                  },
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
        <div className={js|movement-costs|js}>
          <h3> {text({js|Terrain effects|js})} </h3>
          <table
            className={js|weapon-table terrain-effects-table|js}
            ariaLabel={js|Unit terrain effects|js}>
            <thead>
              <tr>
                <th scope={js|col|js}> {text({js|Terrain|js})} </th>
                <th scope={js|col|js}> {text({js|Cost|js})} </th>
                <th scope={js|col|js}> {text({js|Damage received|js})} </th>
              </tr>
            </thead>
            <tbody>
              {eachList(details.terrain_effects, terrain =>
                 <tr key={terrain.Presentation.terrain} className={terrain.class_name}>
                   <th scope={js|row|js}> {text(terrain.name)} </th>
                   <td
                     className={js|terrain-cost|js}
                     title=?{
                       terrain.class_name == {js|blocked|js} ? Some({js|Impassable|js}) : None
                     }>
                     <b> {text(terrain.label)} </b>
                   </td>
                   <td>
                     {eachList(terrain.facts, fact =>
                        <span key={fact.Presentation.label} className={js|terrain-damage-fact|js}>
                          <b
                            className={
                              fact.value < 0.
                                ? {js|resistant|js}
                                : fact.value > 0. ? {js|vulnerable|js} : {js||js}
                            }>
                            {text(Presentation.signed_value(fact.value))}
                          </b>
                          {text({js| |js})}
                          <span className={js|terrain-damage-kind|js}>
                            {text(
                               fact.label == {js||js}
                                 ? {js|Damage|js}
                                 : {
                                   let js_text_part_0 = fact.label;
                                   {j|$(js_text_part_0) Damage|j};
                                 },
                             )}
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
               {js|Cost is movement spent to enter. Damage is the change per hit, in addition to Damage received above.|js},
             )}
          </p>
        </div>
      </div>
    </div>;
};
module Forecast = {
  [@react.component]
  let make = (~forecast: Presentation.forecast) => {
    let props =
      ReactDOM.domProps(
        ~className={
          let js_text_part_1 = forecast.palette;
          {j|attack-health $(js_text_part_1)|j};
        },
        (),
      );
    dataForecast(props, forecast.palette);
    ReactDOM.createElement(
      {js|span|js},
      ~props,
      [|
        <span className={js|forecast-label|js}>
          <span>
            {text(
               {
                 let js_text_part_0 = number(forecast.current_hp);
                 {j|$(js_text_part_0) → |j};
               },
             )}
            <b className={forecast.remaining_class}> {text(forecast.remaining_label)} </b>
            {text(
               {
                 let js_text_part_1 = number(forecast.maximum_hp);
                 {j| / $(js_text_part_1) HP|j};
               },
             )}
          </span>
        </span>,
        <span className={js|health-track|js} role={js|img|js} ariaLabel={forecast.health_label}>
          <span
            className={js|forecast-loss|js}
            style={ReactDOM.Style.make(
              ~left=
                {let js_text_part_0 = number(forecast.maximum_percent);
                 {j|$(js_text_part_0)%|j}},
              ~width=
                {let js_text_part_0 =
                   number(max(0., forecast.current_percent -. forecast.maximum_percent));
                 {j|$(js_text_part_0)%|j}},
              (),
            )}
          />
          <span
            className={js|forecast-possible|js}
            style={ReactDOM.Style.make(
              ~left=
                {let js_text_part_0 =
                   number(min(forecast.minimum_percent, forecast.current_percent));
                 {j|$(js_text_part_0)%|j}},
              ~width=
                {let js_text_part_0 =
                   number(
                     min(forecast.maximum_percent, forecast.current_percent)
                     -. min(forecast.minimum_percent, forecast.current_percent),
                   );
                 {j|$(js_text_part_0)%|j}},
              (),
            )}
          />
          <span
            className={js|forecast-certain|js}
            style={percent(min(forecast.minimum_percent, forecast.current_percent))}
          />
          <span
            className={js|forecast-healing|js}
            style={ReactDOM.Style.make(
              ~left=
                {let js_text_part_0 = number(forecast.current_percent);
                 {j|$(js_text_part_0)%|j}},
              ~width=
                {let js_text_part_0 =
                   number(max(0., forecast.maximum_percent -. forecast.current_percent));
                 {j|$(js_text_part_0)%|j}},
              (),
            )}
          />
          <span
            className={js|forecast-current|js}
            style={ReactDOM.Style.make(
              ~left=
                {let js_text_part_0 = number(forecast.current_percent);
                 {j|$(js_text_part_0)%|j}},
              (),
            )}
            title={
                    let js_text_part_1 = number(forecast.current_hp);
                    {j|Current HP: $(js_text_part_1)|j};
                  }
          />
        </span>,
        <span className={js|forecast-impact|js}>
          <KeepTogether value={forecast.impact_label} />
        </span>,
      |],
    );
  };
};
module Attack = {
  [@react.component]
  let make = (~choice: Presentation.attack, ~onAttack, ~busy=false) => {
    let props =
      ReactDOM.domProps(
        ~className={js|attack-action|js},
        ~ariaLabel=choice.button_label,
        ~disabled=busy || choice.result == None,
        ~onClick=_ => onAttack(choice.weapon.id),
        (),
      );
    dataAttack(props, choice.weapon.id);
    <Radix.Collapsible.Root asChild=true>
      <div className={js|attack-action-row|js}>
        <div className={js|attack-box|js}>
          {ReactDOM.createElement(
             {js|button|js},
             ~props,
             [|
               <span
                 className={
                             let js_text_part_1 =
                               choice.result != None ? {js| paired|js} : {js||js};
                             {j|attack-heading$(js_text_part_1)|j};
                           }>
                 {eachList(
                    choice.result == None ? [choice.name] : [choice.name, choice.response_name],
                    name =>
                    <strong key=name> <KeepTogether value=name /> </strong>
                  )}
                 {choice.result == None
                    ? <span> {text({js|Out of reach or blocked|js})} </span> : React.null}
               </span>,
               option(choice.result, result =>
                 <span className={js|attack-forecasts|js}>
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
                   ~className={js|attack-inspect|js},
                   ~ariaLabel={
                     let js_text_part_1 = Js.String.toLowerCase(choice.weapon.name);
                     {j|Inspect $(js_text_part_1) damage calculation|j};
                   },
                   ~dangerouslySetInnerHTML={ "__html": Art.icon_markup({js|search|js}) },
                   (),
                 );
               dataInspect(props, choice.weapon.id);
               <Radix.Collapsible.Trigger asChild=true>
                 {ReactDOM.createElement({js|button|js}, ~props, [||])}
               </Radix.Collapsible.Trigger>;
             },
           )}
        </div>
        {option(choice.result, result =>
           <Radix.Collapsible.Content asChild=true>
             <div className={js|combat-explanation|js}>
               <div className={js|attack-calculation|js}>
                 <p> {text(result.Presentation.attack.description)} </p>
                 <p className={js|damage-equation|js}> {text(result.attack.equation)} </p>
               </div>
               <div className={js|attack-calculation|js}>
                 <p> {text(result.retaliation.description)} </p>
                 <p className={js|damage-equation|js}> {text(result.retaliation.equation)} </p>
               </div>
             </div>
           </Radix.Collapsible.Content>
         )}
      </div>
    </Radix.Collapsible.Root>;
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
    let units = combat.units;
    let player = units[0].faction == {js|player|js} ? 0 : 1;
    let opponent = 1 - player;
    let finalHealth =
      Array.map(Int32.to_float, combat.strikes[Array.length(combat.strikes) - 1].Domain.health);
    let playerChange = finalHealth[player] -. Int32.to_float(units[player].hp);
    let opponentChange = finalHealth[opponent] -. Int32.to_float(units[opponent].hp);
    <Radix.Accordion.Root _type={js|single|js} collapsible=true asChild=true>
      <ol className={js|exchange-log|js} role={js|log|js} ariaLabel={js|Combat exchange|js}>
        {eachList(List.filter(entry => entry.Presentation.strike < shown, entries), entry =>
           <Radix.Accordion.Item key={entry.id} value={entry.id} asChild=true>
             <li
               className={
                           let js_text_part_1 = entry.good ? {js|good|js} : {js|bad|js};
                           {j|exchange-event $(js_text_part_1)|j};
                         }>
               <Radix.Accordion.Header asChild=true>
                 <div className={js|exchange-line|js}>
                   <strong> {text(entry.text)} </strong>
                   <Radix.Accordion.Trigger asChild=true>
                     <button
                       className={js|icon-button exchange-inspect|js}
                       ariaLabel={
                                   let js_text_part_1 = entry.text;
                                   {j|Inspect calculation: $(js_text_part_1)|j};
                                 }
                       dangerouslySetInnerHTML={ "__html": Art.icon_markup({js|search|js}) }
                     />
                   </Radix.Accordion.Trigger>
                 </div>
               </Radix.Accordion.Header>
               <Radix.Accordion.Content asChild=true>
                 <div className={js|exchange-calculation|js}>
                   {eachList(entry.details, detail =>
                      <p key=detail className={js|damage-equation|js}> {text(detail)} </p>
                    )}
                 </div>
               </Radix.Accordion.Content>
             </li>
           </Radix.Accordion.Item>
         )}
        {complete
           ? <>
               {finalHealth[opponent] == 0.
                  ? <li className={js|exchange-outcome good|js}>
                      {text({js|Your opponent died|js})}
                    </li>
                  : React.null}
               {finalHealth[player] == 0.
                  ? <li className={js|exchange-outcome bad|js}> {text({js|You died|js})} </li>
                  : React.null}
               {finalHealth[0] > 0. && finalHealth[1] > 0.
                  ? <li className={js|exchange-outcome|js}> {text({js|Exchange ended|js})} </li>
                  : React.null}
               <li className={js|exchange-totals|js}>
                 {playerChange > 0.
                    ? <span className={js|good|js}>
                        {text(
                           {
                             let js_text_part_1 = number(playerChange);
                             {j|You healed +$(js_text_part_1) HP|j};
                           },
                         )}
                      </span>
                    : <span className={js|bad|js}>
                        {text(
                           {
                             let js_text_part_1 = Presentation.loss(-. playerChange);
                             {j|You took $(js_text_part_1) HP|j};
                           },
                         )}
                      </span>}
                 {opponentChange > 0.
                    ? <span className={js|bad|js}>
                        {text(
                           {
                             let js_text_part_1 = number(opponentChange);
                             {j|Opponent healed +$(js_text_part_1) HP|j};
                           },
                         )}
                      </span>
                    : <span className={js|good|js}>
                        {text(
                           {
                             let js_text_part_1 = Presentation.loss(-. opponentChange);
                             {j|You dealt $(js_text_part_1) HP|j};
                           },
                         )}
                      </span>}
               </li>
             </>
           : React.null}
      </ol>
    </Radix.Accordion.Root>;
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
      id={js|unit-panel|js}
      className={
                  let js_text_part_1 = busy ? {js| resolving|js} : {js||js};
                  {j|panel$(js_text_part_1)|j};
                }
      ariaLabel={js|Selected unit|js}
      hidden={panel == Presentation.Hidden}>
      {switch (panel) {
       | Hidden => React.null
       | Outcome(title, description) =>
         <>
           <div className={js|unit-title|js}> <h2> {text(title)} </h2> </div>
           <p> {text(description)} </p>
           <button id={js|play-again|js} className={js|primary|js} onClick={_ => onNewBattle()}>
             {text({js|Play again|js})}
           </button>
         </>
       | Details(details) => <Details view actions details />
       | Combat(attacker, defender, reset) =>
         <>
           <div className={js|duel|js}>
             <UnitCard view game unit=attacker hp=?{Option.map(health => health[0], health)} />
             <span className={js|duel-divider|js} ariaHidden=true> {text({js|⚔|js})} </span>
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
              <div key=reset className={js|attack-actions|js} ariaLabel={js|Attack actions|js}>
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
                   setError(_ =>
                     Some(
                       {
                         let js_text_part_1 = frame.url;
                         {j|Terrain preview could not load: $(js_text_part_1)|j};
                       },
                     )
                   );
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
      <div className={js|preview-image|js} ref={ReactDOM.Ref.domRef(host)} ariaHidden=true />
      {loading
         ? <span className={js|preview-status|js} role={js|status|js}>
             {text({js|Loading preview…|js})}
           </span>
         : option(error, message =>
             <div className={js|preview-status|js} role={js|alert|js} title=message>
               {text({js|Preview could not load.|js})}
               <button
                 onClick={_ => {
                   let _ =
                     Terrain_preview.retry(cache, frame)
                     |> Js.Promise.then_(_ => Js.Promise.resolve())
                     |> Js.Promise.catch(_ => Js.Promise.resolve());
                   setAttempt(previous => previous + 1);
                 }}>
                 {text({js|Retry|js})}
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
      id={js|terrain-panel|js}
      className={js|panel|js}
      ariaLabel={js|Selected terrain|js}
      hidden={panel == None}>
      {option(panel, panel =>
         <>
           {option(panel.Presentation.recruitment, recruitment =>
              <div className={js|terrain-recruitment|js}>
                <h3> {text({js|Recruit here|js})} </h3>
                <div className={js|recruit-list|js}>
                  {eachList(
                     recruitment.options,
                     recruit => {
                       let definition = recruit.Presentation.definition;
                       let props =
                         ReactDOM.domProps(
                           ~className={js|recruit-button|js},
                           ~title=definition.description,
                           ~disabled=!recruit.enabled,
                           ~onClick=_ => actions.recruit(definition.id, panel.tile.id),
                           (),
                         );
                       dataRecruit(props, definition.id);
                       ReactDOM.createElement(
                         {js|button|js},
                         ~props,
                         [|
                           <span>
                             {text(definition.name)}
                             {state.campaign != None
                                ? <small className={js|recruit-preview|js}>
                                    {text(
                                       {
                                         let js_text_part_0 =
                                           number(Int32.to_float(definition.hp));
                                         let js_text_part_2 = number(definition.move);
                                         {j|$(js_text_part_0) HP   $(js_text_part_2) move|j};
                                       },
                                     )}
                                  </small>
                                : React.null}
                           </span>,
                           <span>
                             {text(
                                {
                                  let js_text_part_0 = number(recruit.cost);
                                  {j|$(js_text_part_0) gold|j};
                                },
                              )}
                           </span>,
                         |],
                       );
                     },
                   )}
                </div>
                <p className={js|recruit-hint|js}> {text(recruitment.hint)} </p>
              </div>
            )}
           <div className={js|terrain-detail|js}>
             <div className={js|terrain-portrait|js}>
               {option(Art.terrain_portrait(view.artwork, panel.tile.terrain), frame =>
                  <TerrainPreview cache=previews frame />
                )}
               <span className={js|unit-side|js}> {text({js|Terrain|js})} </span>
             </div>
             <div className={js|terrain-info|js}>
               <h2> {text(panel.name)} </h2>
               {option(panel.ownership, ((label, className)) =>
                  <p className={j|terrain-owner $(className)|j}> {text(label)} </p>
                )}
               <p> {text(panel.description)} </p>
               <ul className={js|terrain-facts|js} role={js|list|js}>
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
    let (conversationsOpen, setConversationsOpen) = React.useState(() => false);
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
               (unit: Domain.public_unit) =>
                 unit.faction == {js|player|js}
                 && unit.role != Some(Domain.Role.Leader)
                 && unit.role != Some(Domain.Role.Courier)
                 && !
                      Array.exists(
                        name => Some(name) == unit.character,
                        mission.required_companions,
                      )
                 && !Story.returns_with_story(view.catalog, campaign, unit),
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
               id={js|mission-overview|js}
               className={js|campaign-panel mission-overview|js}
               hidden={!expanded}
               ariaLabel={js|Mission briefing|js}>
               <h2> {text(layout.name)} </h2>
               <p className={js|mission-heading|js}>
                 {text(
                    {
                      let js_text_part_0 = definition.name;
                      let js_text_part_2 = number(campaign.mission +. 1.);
                      let js_text_part_4 = string_of_int(Array.length(definition.missions));
                      {j|$(js_text_part_0) $(js_text_part_2)/$(js_text_part_4)|j};
                    },
                  )}
               </p>
               <h3> {text({js|Briefing|js})} </h3>
               <p>
                 {text(
                    {
                      let js_text_part_0 = chapter.briefing;
                      {j|$(js_text_part_0) |j};
                    },
                  )}
                 <Dialog_window
                   trigger={
                     <button
                       className={js|conversation-icon|js}
                       ariaLabel={js|Campaign conversations|js}
                       title={js|Campaign conversations|js}>
                       <svg viewBox={js|0 0 24 24|js} ariaHidden=true focusable={js|false|js}>
                         <path
                           d={js|M14 3H4a2 2 0 0 0-2 2v11l4-4h8a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2Z|js}
                         />
                         <path d={js|M8 16v1a2 2 0 0 0 2 2h8l4 3V11a2 2 0 0 0-2-2|js} />
                       </svg>
                     </button>
                   }
                   title={js|Campaign conversations|js}
                   opened=conversationsOpen
                   onOpenChange={value => setConversationsOpen(_ => value)}
                   width={js|680px|js}
                   tall=true>
                   {switch (state.campaign) {
                    | Some(campaign) when campaign.dialogue_index > 0. =>
                      each(
                        Array.sub(
                          campaign.dialogue,
                          0,
                          min(
                            Array.length(campaign.dialogue),
                            int_of_float(campaign.dialogue_index),
                          ),
                        ),
                        line =>
                        <p>
                          <strong> {text(line.Domain.speaker)} </strong>
                          <br />
                          {text(line.text)}
                        </p>
                      )
                    | _ => <p> {text({js|No conversations yet.|js})} </p>
                    }}
                 </Dialog_window>
               </p>
               {chapter.hints == []
                  ? React.null
                  : <>
                      <h3> {text({js|Hints|js})} </h3>
                      <ul className={js|mission-items|js}>
                        {eachList(chapter.hints, hint =>
                           <li key=hint> <span> {text(hint)} </span> </li>
                         )}
                      </ul>
                    </>}
               <h3> {text({js|Objectives|js})} </h3>
               <ul className={js|mission-items mission-goals|js}>
                 {eachList(
                    Story.goal_statuses(view.game),
                    status => {
                      let progress = status.Domain.progress;
                      <li
                        key={progress.label}
                        className={
                                    let js_text_part_0 =
                                      progress.done_ ? {js|completed-goal |js} : {js||js};
                                    let js_text_part_1 =
                                      status.failed ? {js|failed-goal|js} : {js||js};
                                    {j|$(js_text_part_0)$(js_text_part_1)|j};
                                  }
                        ariaLabel=?{
                          status.failed
                            ? Some(
                                {
                                  let js_text_part_0 = progress.label;
                                  {j|$(js_text_part_0): failed|j};
                                },
                              )
                            : progress.done_
                                ? Some(
                                    {
                                      let js_text_part_0 = progress.label;
                                      {j|$(js_text_part_0): completed|j};
                                    },
                                  )
                                : None
                        }>
                        <span> {text(progress.label)} </span>
                        {progress.total > 1.
                           ? <strong>
                               {text(
                                  {
                                    let js_text_part_0 = number(progress.current);
                                    let js_text_part_2 = number(progress.total);
                                    {j|$(js_text_part_0)/$(js_text_part_2)|j};
                                  },
                                )}
                             </strong>
                           : React.null}
                      </li>;
                    },
                  )}
               </ul>
               {Story.optional_statuses(view.game) == []
                  ? React.null
                  : <>
                      <h3> {text({js|Optional objectives|js})} </h3>
                      <ul className={js|mission-items mission-goals|js}>
                        {eachList(
                           Story.optional_statuses(view.game),
                           status => {
                             let progress = status.Domain.progress;
                             <li
                               key={progress.label}
                               className={
                                           let js_text_part_0 =
                                             progress.done_ ? {js|completed-goal |js} : {js||js};
                                           let js_text_part_1 =
                                             status.failed ? {js|failed-goal|js} : {js||js};
                                           {j|$(js_text_part_0)$(js_text_part_1)|j};
                                         }
                               ariaLabel=?{
                                 status.failed
                                   ? Some(
                                       {
                                         let js_text_part_0 = progress.label;
                                         {j|$(js_text_part_0): failed|j};
                                       },
                                     )
                                   : progress.done_
                                       ? Some(
                                           {
                                             let js_text_part_0 = progress.label;
                                             {j|$(js_text_part_0): completed|j};
                                           },
                                         )
                                       : None
                               }>
                               <span> {text(progress.label)} </span>
                               {progress.total > 1.
                                  ? <strong>
                                      {text(
                                         {
                                           let js_text_part_0 = number(progress.current);
                                           let js_text_part_2 = number(progress.total);
                                           {j|$(js_text_part_0)/$(js_text_part_2)|j};
                                         },
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
                    id={js|mission-result|js}
                    className={js|campaign-panel|js}
                    ariaLabel={js|Mission result|js}
                    tabIndex=(-1)>
                    <h2> {text(layout.name)} </h2>
                    <div className={js|mission-debrief|js}>
                      <h3>
                        {text(
                           campaign.completed
                             ? {js|Campaign complete|js} : {js|The company regroups|js},
                         )}
                      </h3>
                      <p>
                        {text(
                           Option.value(
                             ~default=Option.value(~default={js||js}, campaign.outcome),
                             chapter.debrief,
                           ),
                         )}
                      </p>
                      {!campaign.completed
                         ? <>
                             <p>
                               {text(
                                  {
                                    let js_text_part_1 =
                                      number(min(8., Native.gold(state.gold, {js|player|js})));
                                    {j|Choose up to four survivors for the next mission. They return at full health. Carrying $(js_text_part_1) gold.|j};
                                  },
                                )}
                             </p>
                             <div className={js|recall-list|js}>
                               {eachList(
                                  survivors,
                                  unit => {
                                    let checked = List.exists(id => id == unit.Domain.id, recall);
                                    let rule = Catalog.unit(view.catalog, unit.type_);
                                    <label
                                      key={unit.id}
                                      className={
                                                  let js_text_part_1 =
                                                    checked ? {js|selected|js} : {js||js};
                                                  {j|recall-choice $(js_text_part_1)|j};
                                                }>
                                      <input
                                        type_={js|checkbox|js}
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
                                             {
                                               let js_text_part_0 =
                                                 number(Int32.to_float(unit.hp));
                                               let js_text_part_2 =
                                                 number(Int32.to_float(rule.hp));
                                               {j|$(js_text_part_0) / $(js_text_part_2) HP|j};
                                             },
                                           )}
                                        </span>
                                        <span>
                                          {text(
                                             {
                                               let js_text_part_1 = number(rule.level);
                                               let js_text_part_3 =
                                                 switch (rule.experience_needed) {
                                                 | None => {js|Fully advanced|js}

                                                 | Some(needed) =>
                                                   let js_text_part_0 =
                                                     number(
                                                       Option.value(~default=0., unit.experience),
                                                     );
                                                   let js_text_part_2 = number(needed);
                                                   {j|$(js_text_part_0) / $(js_text_part_2) XP|j};
                                                 };
                                               {j|Level $(js_text_part_1) · $(js_text_part_3)|j};
                                             },
                                           )}
                                        </span>
                                      </span>
                                    </label>;
                                  },
                                )}
                             </div>
                             <button
                               className={js|primary|js}
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
                                  {
                                    let js_text_part_1 =
                                      switch (
                                        definition.missions[int_of_float(campaign.mission) + 1]
                                      ) {
                                      | mission => Catalog.mission_map(view.catalog, mission).name
                                      };
                                    {j|Continue to $(js_text_part_1)|j};
                                  },
                                )}
                             </button>
                           </>
                         : <button className={js|primary|js} onClick={_ => onNewCampaign()}>
                             {text({js|Choose another campaign|js})}
                           </button>}
                    </div>
                  </section>
                : React.null}
           </>;
         },
       )}
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
            ~none={js|Your commander has fallen.|js},
            ~some=
              (campaign: Domain.campaign_state) =>
                Option.value(~default={js|The company cannot continue.|js}, campaign.outcome),
            state.campaign,
          );
        <section
          id={js|battle-result|js} className={js|battle-result|js} ariaLabel={js|Battle result|js}>
          <p className={js|result-context|js}>
            {text(Catalog.map(view.catalog, state.map_id).name)}
          </p>
          <h2> {text({js|Defeat|js})} </h2>
          <p className={js|result-reason|js}> {text(reason)} </p>
          <div className={js|result-actions|js}>
            <button
              id={campaign ? {js|retry-mission|js} : {js|retry-scenario|js}}
              className={js|primary|js}
              onClick={_ => actions.retry()}>
              {text(campaign ? {js|Retry mission|js} : {js|Retry scenario|js})}
            </button>
            <button id={js|choose-another-game|js} onClick={_ => onNewGame()}>
              {text({js|Choose another game|js})}
            </button>
          </div>
          {campaign
             ? <p className={js|result-hint|js}>
                 {text(
                    {js|You can also return to an earlier turn or mission from Saved games.|js},
                  )}
               </p>
             : React.null}
        </section>;
      };
  };
};
