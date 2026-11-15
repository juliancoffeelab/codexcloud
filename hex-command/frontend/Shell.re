let reload = Browser_dom.reload;
[@react.component]
let make = (~failure: option(string)) => {
  let text = value => React.string(value);
  <main className={js|battle-layout loading-shell|js} ariaBusy={failure == None}>
    <Header />
    <div className={js|battle-content|js}>
      <div className={js|workspace map-only|js}>
        <Battlefield_window
          phase={failure == None ? Battlefield_window.Loading : Battlefield_window.Ready}>
          {Option.fold(
             ~none=React.null,
             ~some=
               reason =>
                 <div className={js|startup-failure|js} role={js|alert|js}>
                   <p>
                     {text({js|The game could not start. Reloading keeps your saved battle.|js})}
                   </p>
                   <button className={js|quiet|js} onClick={_ => reload()}>
                     {text({js|Reload game|js})}
                   </button>
                   <details>
                     <summary> {text({js|Error details|js})} </summary>
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
