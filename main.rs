extern mod extra;

use components::*;
use engine::{Display, Color, MainLoopState};

mod components;
mod ecm;
mod engine;
mod systems;
mod world_gen;

struct GameState {
    entities: ~[GameObject],
}

fn initial_state(width: uint, height: uint) -> ~GameState {
    let mut state = ~GameState{entities: ~[]};
    state.entities.push(GameObject{
        position: Some(Position{x: 10, y: 20}),
        health: Some(Health(100)),
        tile: Some(Tile{level: 2, glyph: '@', color: Color(255, 0, 255)}),
    });
    let world = world_gen::forrest(width, height);
    for world.iter().advance |&(x, y, item)| {
        state.entities.push(GameObject{
            position: Some(Position{x: x, y: y}),
            health: None,
            tile: Some(Tile{level: 0, glyph: '.', color: Color(0, 255, 255)}),
        })
    }
    state.entities.push(GameObject{
        position: Some(Position{x: 1, y: 1}),
        health: None,
        tile: None,
    });
    state
}

fn update(state: &mut GameState,
          display: &mut Display,
          keys: &mut extra::deque::Deque<char>) -> MainLoopState {
    if keys.len() > 0 {
        keys.clear();
    }
    for state.entities.mut_iter().advance |e| {
        systems::tile_system(e, display);
        systems::health_system(e);
    }
    engine::Running
}


fn main() {
    let (width, height) = (80, 50);
    engine::main_loop(width, height, "Dose Response",
                      "./fonts/dejavu16x16_gs_tc.png", initial_state, update);
}
