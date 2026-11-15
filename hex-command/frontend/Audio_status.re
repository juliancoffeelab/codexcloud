open Ui;
[@react.component]
let make = (~retryHud, ~renderer: Phaser_boot.t) => {
  let (hud, setHud) = React.useState(() => Observable.get(Phaser_sound.hud_readiness));
  let (battle, setBattle) = React.useState(() => Observable.get(Phaser_sound.readiness));
  React.useEffect0(() => {
    let hud = Observable.subscribe(Phaser_sound.hud_readiness, value => setHud(_ => value));
    let battle = Observable.subscribe(Phaser_sound.readiness, value => setBattle(_ => value));
    Some(
      () => {
        hud();
        battle();
      },
    );
  });
  <>
    {option(hud.error, message =>
       <div role={js|alert|js}>
         <p> {text({js|Menu audio unavailable.|js})} </p>
         <details>
           <summary> {text({js|Error details|js})} </summary>
           <pre> {text(message)} </pre>
         </details>
         <button
           onClick={_ => {
             let _ = retryHud();
             ();
           }}>
           {text({js|Retry menu audio|js})}
         </button>
       </div>
     )}
    {switch (battle.error) {
     | Some(message) =>
       <div role={js|alert|js}>
         <p> {text({js|Audio unavailable.|js})} </p>
         <details>
           <summary> {text({js|Error details|js})} </summary>
           <pre> {text(message)} </pre>
         </details>
         <button
           onClick={_ => {
             let _ = Phaser_boot.retry_audio(renderer);
             ();
           }}>
           {text({js|Retry audio|js})}
         </button>
       </div>
     | None =>
       <p className={js|status|js} role={js|status|js}>
         {text(
            battle.ready
              ? {js|Audio prepared.|js} : {js|Preparing audio… Menus remain available.|js},
          )}
       </p>
     }}
  </>;
};
