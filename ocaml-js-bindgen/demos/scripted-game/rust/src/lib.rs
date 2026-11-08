use ocamljs_bindgen::{Callback, ocamljs_bindgen};
#[ocamljsify::ocamljsify]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wall {
    Left,
    Right,
}

#[ocamljsify::ocamljsify]
pub struct Collision {
    pub tick: u32,
    pub x: u32,
    pub wall: Wall,
}
#[ocamljsify::ocamljsify]
pub struct Snapshot {
    pub tick: u32,
    pub x: u32,
    pub score: u32,
}
#[ocamljs_bindgen(opaque)]
pub struct World {
    tick: u32,
    x: u32,
    direction: i32,
    score: u32,
    handler: Option<Callback<Collision>>,
}
#[ocamljs_bindgen(module = World, name = "create")]
pub fn world() -> World {
    World {
        tick: 0,
        x: 1,
        direction: 1,
        score: 0,
        handler: None,
    }
}
#[ocamljs_bindgen(module = World)]
pub fn set_handler(world: &mut World, handler: Callback<Collision>) {
    world.handler = Some(handler);
}
#[ocamljs_bindgen(module = World)]
pub fn clear_handler(world: &mut World) {
    world.handler = None;
}
#[ocamljs_bindgen(module = World)]
pub fn tick(world: &mut World) -> u32 {
    world.tick += 1;
    world.x = (world.x as i32 + world.direction) as u32;
    if world.x == 0 || world.x == 7 {
        world.direction = -world.direction;
        if let Some(handler) = &world.handler {
            let _ = handler.emit(Collision {
                tick: world.tick,
                x: world.x,
                wall: if world.x == 0 {
                    Wall::Left
                } else {
                    Wall::Right
                },
            });
        }
    }
    world.tick
}
#[ocamljs_bindgen(module = World)]
pub fn award(world: &mut World, points: u32) {
    world.score = world.score.saturating_add(points);
}
#[ocamljs_bindgen(module = World)]
pub fn snapshot(world: &World) -> Snapshot {
    Snapshot {
        tick: world.tick,
        x: world.x,
        score: world.score,
    }
}
