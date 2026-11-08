let reload = Browser_dom.reload;
[@react.component]
let make = (~failure: option(string)) => {
  let text = value => React.string(Utf8.to_js(value));
  <main className="battle-layout loading-shell" ariaBusy={failure == None}>
    <Header />
    <div className="battle-content">
      <div className="workspace map-only">
        <Battlefield_window
          phase={failure == None ? Battlefield_window.Loading : Battlefield_window.Ready}>
          {Option.fold(
             ~none=React.null,
             ~some=
               reason =>
                 <div className="startup-failure" role="alert">
                   <p> {text("The game could not start. Reloading keeps your saved battle.")} </p>
                   <button className="quiet" onClick={_ => reload()}>
                     {text("Reload game")}
                   </button>
                   <details>
                     <summary> {text("Error details")} </summary>
                     <pre> {text(reason)} </pre>
                   </details>
                 </div>,
             failure,
           )}
        </Battlefield_window>
      </div>
    </div>
  </main>;
};
