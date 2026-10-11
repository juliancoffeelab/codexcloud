[@mel.scope "window.location"] external reload: unit => unit = "reload";
[@react.component]
let make = (~failure: option(string)) => {
  let text = React.string;
  <main className="loading-shell" ariaBusy={failure == None}>
    <header className="battle-bar" ariaLabel="Battle controls">
      <div className="header-economy" ariaLabel="Battle statistics loading">
        <span className="treasury">
          <i className="coin" ariaHidden=true />
          <b> {text("—")} </b>
        </span>
        <span className="income"> <b> {text("—")} </b> <span> {text("/turn")} </span> </span>
        <span className="villages" ariaLabel="Villages"> {text("—/—")} </span>
      </div>
      <button id="end-turn" className="primary" disabled=true>
        <span> {text("End turn")} </span>
        <span className="round"> {text("Turn —")} </span>
      </button>
      <button className="icon-button" ariaLabel="Undo move" disabled=true>
        <svg viewBox="0 0 24 24" ariaHidden=true>
          <path d="M9 4 4 9l5 5M4 9h9a7 7 0 0 1 7 7v4" />
        </svg>
      </button>
      <button className="icon-button" ariaLabel="Settings" disabled=true>
        <svg viewBox="0 0 24 24" ariaHidden=true>
          <path
            d="m9 3 1-1h4l1 3 3 1 3-1 2 4-2 2v3l2 2-2 4-3-1-3 1-1 3h-4l-1-3-3-1-3 1-2-4 2-2v-3L1 9l2-4 3 1 3-1z"
          />
          <circle cx="12" cy="12" r="3" />
        </svg>
      </button>
    </header>
    <div className="workspace">
      <section className="map-section" ariaLabel="Battlefield loading">
        <div className="map-scroll loading-map">
          <div className="loading-message" role="status" ariaLive="polite">
            {switch (failure) {
             | None => <p> {text("Loading the game…")} </p>
             | Some(reason) =>
               <>
                 <p> {text("The game could not start. Reloading keeps your saved battle.")} </p>
                 <button className="quiet" onClick={_ => reload()}>
                   {text("Reload game")}
                 </button>
                 <details>
                   <summary> {text("Error details")} </summary>
                   <pre> {text(reason)} </pre>
                 </details>
               </>
             }}
          </div>
        </div>
      </section>
    </div>
  </main>;
};
