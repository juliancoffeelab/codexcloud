[@mel.scope "window.location"] external reload: unit => unit = "reload";
[@react.component]
let make = (~failure: option(string)) => {
  let text = React.string;
  <main className="battle-layout loading-shell" ariaBusy={failure == None}>
    <Header />
    <div className="battle-content">
      <div className="workspace map-only">
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
    </div>
  </main>;
};
