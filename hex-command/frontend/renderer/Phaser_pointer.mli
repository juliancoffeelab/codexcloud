type client_position = { client_x : float; client_y : float }
(** Immutable event samples at the mutable Phaser/DOM boundary. *)

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

type press = private { sample : sample; intent : intent }

val sample : Phaser.pointer -> sample
val press : Phaser.pointer -> press option
