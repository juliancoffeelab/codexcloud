type 'a t = { mutable value : 'a; listeners : ('a -> unit) Host_collections.Set.t }
type 'a reader = 'a t

let reader signal = signal
let create value = { value; listeners = Host_collections.Set.create () }
let get signal = signal.value

let set signal value =
  signal.value <- value;
  Host_collections.Set.each signal.listeners (fun[@u] listener -> listener value)

let subscribe signal listener =
  let _ = Host_collections.Set.add signal.listeners listener in
  listener signal.value;
  fun () ->
    let _ = Host_collections.Set.delete signal.listeners listener in
    ()
