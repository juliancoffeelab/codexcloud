open Ui;
[@mel.module "virtual:terrain-prebuilt"]
external terrainSprites: Js.Dict.t(Terrain.sprite) = "default";

module Settings = {
  [@react.component]
  let make =
      (~view: Session.model, ~actions: Session.actions, ~dialog, ~onNewBattle, ~onNewCampaign) => {
    let state = Native.state(view.game);
    <dialog id="settings-dialog" ref={ReactDOM.Ref.domRef(dialog)} onClick=backdrop>
      <div className="settings-heading">
        <h2> {text("Settings")} </h2>
        <button
          id="close-settings"
          className="icon-button"
          ariaLabel="Close settings"
          onClick={_ => close(dialog)}>
          {text({js|×|js})}
        </button>
      </div>
      <label className="sound-control" htmlFor="sound-effects">
        <span> {text("Sound effects")} </span>
        <input
          id="sound-effects"
          type_="checkbox"
          checked={view.sound}
          onChange={event => ignorePromise(actions.set_sound(bindChecked(event)))}
        />
      </label>
      <button id="new-campaign" className="primary" onClick={_ => onNewCampaign()}>
        {text("New campaign")}
      </button>
      <button
        id="new-game"
        className="quiet"
        disabled={state.turn == Domain.side_enemy && Js.Nullable.toOption(state.winner) == None}
        onClick={_ => onNewBattle()}>
        {text("New battle")}
      </button>
      <details className="panel">
        <summary> {text("Field manual")} </summary>
        <div className="manual">
          <p> {text("Each unit moves once and attacks once. Attacking ends its turn.")} </p>
          <p>
            {text("Capture a village by entering it. Each owned village earns ")}
            <strong> {text("2 gold")} </strong>
            {text(" at the start of your turn and heals a friendly occupant by ")}
            <strong> {text("4 HP")} </strong>
            {text(".")}
          </p>
          <p>
            {text(
               "Select an enemy, then tap a weapon to attack. Use the magnifying glass for damage details. Each unit attacks once per turn. A surviving defender replies once with its strongest weapon of the same kind: melee answers melee; ranged answers ranged. Both weapons must reach. Forests and hills block ranged shots through them.",
             )}
          </p>
          <p>
            {text(
               "Blade, Pierce, and Impact damage interact with armor. Soldiers take 1 less Blade and Pierce damage, and 1 more Impact damage. Commanders take 1 less Blade damage. Archers and scouts have no modifiers. Armor and weakness change base damage first, then terrain cover and shallow-water exposure apply. No rounding is needed. Damage cannot fall below 0. A fully blocked hit still spends the attack, and a surviving defender can reply. The combat preview shows the exact outcome; there are no hit-chance rolls.",
             )}
          </p>
          <p>
            {text(
               "Each unit's Movement costs list shows how much movement it spends entering each terrain. Occupied hexes block movement. Units have no zone of control and do not intercept attacks; protect a fragile unit by blocking the enemy's approach, rather than standing behind it. Forests reduce ranged damage by 1 for land races. Hills reduce their damage received by 1. Land races cannot cross deep water; bridges provide a crossing. Shallow water and marsh add 1 to their damage received. Merfolk swim through deep water, receive no shallow-water penalty, and take 1 less damage in marsh. Forests, groves and hills slow merfolk and add 1 to their damage received. Merfolk move normally on plains.",
             )}
          </p>
          <p>
            {text(
               "Keep your leader on your keep, then select an adjacent empty hex passable to the recruit. Merfolk may recruit into water. The unit appears on that hex and acts next turn.",
             )}
          </p>
          <p>
            {text(
               "Unit cards show level and experience, with next rank names in the progress description. All unit families have a finite advancement ladder. Dealing damage earns experience; defeating an enemy earns more, including during retaliation. Select a ready unit with enough XP to advance. Advancement preserves wounds and consumes its turn. Returning companions and recalled survivors keep their XP and rank between campaign missions. Retrying restores the company that entered that mission.",
             )}
          </p>
          <p>
            {text(
               "In skirmishes, defeat the enemy commander to win. In campaigns, follow the mission objectives and protect your leader. Groves heal their owner’s occupants; forts protect against all attacks; ruins shelter against ranged attacks. Marsh slows heavy units. Bones resist arrows but are weak against impact; wolves are fast but vulnerable to pierce damage. Keyboard: Tab to the map, use arrow keys to browse hexes, Enter to select. Escape clears selection.",
             )}
          </p>
        </div>
      </details>
      {option(state.campaign, campaign =>
         <details className="panel">
           <summary> {text("Campaign conversations")} </summary>
           {each(
              Array.sub(
                campaign.Domain.dialogue,
                0,
                min(Array.length(campaign.dialogue), int_of_float(campaign.dialogueIndex)),
              ),
              line =>
              <p> <strong> {text(line.Domain.speaker)} </strong> {text(" " ++ line.text)} </p>
            )}
         </details>
       )}
      <details className="dispatch">
        <summary> {text("Battle journal")} </summary>
        <ol id="log"> {each(state.log, entry => <li> {text(entry)} </li>)} </ol>
      </details>
    </dialog>;
  };
};

module MapPicker = {
  [@react.component]
  let make = (~catalog, ~selected, ~onSelect) => {
    let maps = Catalog.battle_maps(catalog);
    React.useEffect1(
      () => {
        Terrain.preload_manifest(terrainSprites, selected);
        None;
      },
      [|selected|],
    );
    <fieldset id="map-picker">
      <legend> {text("Battlefield")} </legend>
      {each(
         maps,
         layout => {
           let size =
             Geometry.preview_dimensions(
               ~cols=int_of_float(layout.Domain.cols),
               ~rows=int_of_float(layout.rows),
             );
           let terrain = Js.Dict.empty();
           Array.iter(
             (tile: Domain.tile) => Js.Dict.set(terrain, tile.id, tile.terrain),
             layout.terrain,
           );
           <label key={layout.id} className="map-choice">
             <input
               type_="radio"
               name="battle-map"
               value={layout.id}
               checked={selected == layout.id}
               onChange={_ => onSelect(layout.id)}
             />
             <span>
               <svg
                 className="map-preview"
                 viewBox={"0 0 " ++ number(size.width) ++ " " ++ number(size.height)}
                 ariaHidden=true>
                 {each(
                    Array.init(
                      int_of_float(layout.rows *. layout.cols),
                      index => {
                        let col = index mod int_of_float(layout.cols);
                        let row = index / int_of_float(layout.cols);
                        let p = Geometry.center(~col, ~row);
                        let id = string_of_int(col) ++ "," ++ string_of_int(row);
                        <polygon
                          key=id
                          points={Geometry.hex_points(p.x, p.y)}
                          fill={Geometry.preview_color(
                            Option.value(~default="plains", Js.Dict.get(terrain, id)),
                          )}
                          stroke="#263322"
                          strokeWidth="2"
                        />;
                      },
                    ),
                    value =>
                    value
                  )}
               </svg>
               <strong> {text(layout.name)} </strong>
               <span className="map-description"> {text(layout.description)} </span>
             </span>
           </label>;
         },
       )}
    </fieldset>;
  };
};

module CampaignPicker = {
  [@react.component]
  let make = (~view: Session.model, ~selected, ~onSelect) =>
    <fieldset className="campaign-picker">
      <legend> {text("Choose a story")} </legend>
      {each(Catalog.campaigns(view.catalog), campaign =>
         <label
           key={campaign.Domain.id}
           className={"campaign-choice " ++ (selected == campaign.id ? "selected" : "")}>
           <input
             type_="radio"
             name="campaign"
             value={campaign.id}
             checked={selected == campaign.id}
             onChange={_ => onSelect(campaign.id)}
           />
           <div className="campaign-portrait">
             <Sprite artwork={view.artwork} name={campaign.leader} width=70. height=90. />
           </div>
           <div>
             <h3> {text(campaign.name)} </h3>
             <p className="campaign-tagline"> {text(campaign.tagline)} </p>
             <p> {text(campaign.description)} </p>
             <span className="campaign-length">
               {text("Three missions as " ++ (campaign.race :> string) ++ " armies")}
             </span>
             {selected == campaign.id
                ? <>
                    <ol className="mission-list">
                      {React.array(
                         Array.mapi(
                           (index, mission) =>
                             <li key={mission.Domain.map.id}>
                               <strong>
                                 {text(
                                    string_of_int(index + 1)
                                    ++ ". "
                                    ++ Catalog.mission_map(view.catalog, mission).name,
                                  )}
                               </strong>
                               {option(mission.subtitle, subtitle =>
                                  <span> {text(subtitle)} </span>
                                )}
                             </li>,
                           campaign.missions,
                         ),
                       )}
                    </ol>
                    <p className="campaign-length">
                      {text(
                         "Led by a "
                         ++ Js.String.toLowerCase(
                              Catalog.unit(view.catalog, campaign.leader).name,
                            ),
                       )}
                    </p>
                  </>
                : React.null}
           </div>
         </label>
       )}
    </fieldset>;
};

module Dialogue = {
  [@react.component]
  let make = (~view: Session.model, ~actions: Session.actions) => {
    let state = Native.state(view.game);
    let dialog = React.useRef(Js.Nullable.null);
    let nextButton = React.useRef(Js.Nullable.null);
    let pending = Story.dialogue_pending(state.campaign);
    React.useLayoutEffect2(
      () => {
        if (pending) {
          switch (Js.Nullable.toOption(dialog.current)) {
          | Some(element) when !isOpen(element) =>
            showModal(element);
            Js.Nullable.toOption(nextButton.current) |> Option.iter(focus);
          | _ => ()
          };
        } else {
          let wasOpen =
            switch (Js.Nullable.toOption(dialog.current)) {
            | None => false
            | Some(element) => isOpen(element)
            };
          close(dialog);
          if (wasOpen && Js.Nullable.toOption(state.winner) != None) {
            byId("mission-result")
            |> Js.Nullable.toOption
            |> Option.iter(element => {
                 focus(element);
                 scroll(element, { "block": "start" });
               });
          };
        };
        None;
      },
      (pending, view.game),
    );
    let navigate = event =>
      if (!React.Event.Keyboard.altKey(event)
          && !React.Event.Keyboard.ctrlKey(event)
          && !React.Event.Keyboard.metaKey(event)
          && !React.Event.Keyboard.shiftKey(event)) {
        switch (React.Event.Keyboard.key(event)) {
        | "ArrowLeft" =>
          React.Event.Keyboard.preventDefault(event);
          if (!React.Event.Keyboard.repeat(event)) {
            actions.previous_dialogue();
          };
        | "ArrowRight" =>
          React.Event.Keyboard.preventDefault(event);
          if (!React.Event.Keyboard.repeat(event)) {
            actions.dialogue(false);
          };
        | _ => ()
        };
      };
    let props =
      ReactDOM.domProps(
        ~className="story-dialog",
        ~ref=ReactDOM.Ref.domRef(dialog),
        ~onKeyDown=navigate,
        ~ariaLabelledby="dialogue-speaker",
        (),
      );
    onCancel(props, React.Event.Form.preventDefault);
    ReactDOM.createElement(
      "dialog",
      ~props,
      [|
        option(
          state.campaign,
          campaign => {
            let index = int_of_float(campaign.Domain.dialogueIndex);
            if (!pending || index < 0 || index >= Array.length(campaign.dialogue)) {
              React.null;
            } else {
              let line = campaign.dialogue[index];
              let chapter = Story.mission(view.catalog, campaign);
              let forward =
                index + 1 == Array.length(campaign.dialogue)
                  ? Js.Nullable.toOption(state.winner) != None
                      ? "Review mission result" : "Return to battle"
                  : "Next line";
              <>
                <p className="story-heading">
                  {text(
                     Catalog.mission_map(view.catalog, chapter.source).name
                     ++ (
                       Js.Nullable.toOption(state.winner) != None
                         ? {js| — After the battle|js} : ""
                     ),
                   )}
                </p>
                <div className="story-line" ariaLive="polite" ariaAtomic=true>
                  <div className="story-portrait">
                    <Sprite
                      artwork={view.artwork}
                      kind="portraits"
                      name={line.portrait}
                      width=90.
                      height=120.
                    />
                  </div>
                  <div>
                    <h2 id="dialogue-speaker"> {text(line.speaker)} </h2>
                    <p className="story-text"> {text(line.text)} </p>
                  </div>
                </div>
                <nav className="story-navigation" ariaLabel="Conversation">
                  <button
                    className="story-back"
                    disabled={index == 0}
                    onClick={_ => actions.previous_dialogue()}
                    ariaLabel="Previous line"
                    title="Previous line (Left arrow)">
                    <Icon name="left" />
                  </button>
                  <span className="story-count">
                    {text(
                       string_of_int(index + 1)
                       ++ " / "
                       ++ string_of_int(Array.length(campaign.dialogue)),
                     )}
                  </span>
                  <button
                    className="story-next"
                    ref={ReactDOM.Ref.domRef(nextButton)}
                    onClick={_ => actions.dialogue(false)}
                    ariaLabel=forward
                    title={forward ++ " (Right arrow)"}>
                    <Icon name="right" />
                  </button>
                </nav>
              </>;
            };
          },
        ),
      |],
    );
  };
};
