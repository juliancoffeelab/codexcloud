type client_position = { client_x : float; client_y : float }
type scene_position = { scene_x : float; scene_y : float }
type source = Mouse | Touch
type button = Primary | Middle | Secondary | Other of int
type intent = Select_or_pan | Pan_only

type sample = {
  id : int;
  source : source;
  button : button;
  client : client_position;
  scene : scene_position;
  time : float;
  cancelled : bool;
}

type press = { sample : sample; intent : intent }

let sample (pointer : Phaser.pointer) =
  let p = Phaser_input_dom.position pointer in
  {
    id = pointer##id;
    source = (if pointer##wasTouch then Touch else Mouse);
    button =
      (match pointer##button with 0 -> Primary | 1 -> Middle | 2 -> Secondary | n -> Other n);
    client = { client_x = p.x; client_y = p.y };
    scene = { scene_x = pointer##x; scene_y = pointer##y };
    time = Phaser_input_dom.event_time pointer##event;
    cancelled = pointer##wasCanceled;
  }

let press (pointer : Phaser.pointer) =
  (* primaryDown follows the initiating button, and Phaser clears it for macOS
     control-click. Held-button masks may already be zero for trackpad taps. *)
  let intent =
    if pointer##wasTouch || (pointer##button = 0 && pointer##primaryDown) then Some Select_or_pan
    else if pointer##button = 1 then Some Pan_only
    else None
  in
  Option.map (fun intent -> { sample = sample pointer; intent }) intent
