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
       <div role="alert">
         <p> {text("Menu audio unavailable.")} </p>
         <details>
           <summary> {text("Error details")} </summary>
           <pre> {text(message)} </pre>
         </details>
         <button
           onClick={_ => {
             let _ = retryHud();
             ();
           }}> {text("Retry menu audio")} </button>
       </div>
     )}
    {switch (battle.error) {
     | Some(message) =>
       <div role="alert">
         <p> {text("Audio unavailable.")} </p>
         <details>
           <summary> {text("Error details")} </summary>
           <pre> {text(message)} </pre>
         </details>
         <button
           onClick={_ => {
             let _ = Phaser_boot.retry_audio(renderer);
             ();
           }}>
           {text("Retry audio")}
         </button>
       </div>
     | None =>
       <p className="status" role="status">
         {text(battle.ready ? "Audio prepared." : "Preparing audio… Menus remain available.")}
       </p>
     }}
  </>;
};
