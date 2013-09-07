use components::*;

mod components;
mod ecm;
mod engine;
mod systems;
mod world_gen;

struct GameState {
    entities: ~[GameObject],
}

fn initial_state() -> ~GameState {
    let width = 3;
    let height = 3;
    let mut state = ~GameState{entities: ~[]};
    state.entities.push(GameObject{
        position: Some(Position{x: 10, y: 20}),
        health: Some(Health(100))});
    let world = world_gen::forrest(width, height);
    for world.iter().advance |&(x, y, item)| {
        state.entities.push(GameObject{
            position: Some(Position{x: x, y: y}),
            health: None,
        })
    }
    state.entities.push(GameObject{
        position: Some(Position{x: 1, y: 1}),
        health: None});
    state
}

fn update(state: &mut GameState) -> engine::MainLoopState {
    for state.entities.mut_iter().advance |e| {
        systems::debug_system(e);
        systems::tile_system(e);
        systems::health_system(e);
    }
    engine::Running
}


fn main() {
    engine::main_loop(initial_state, update);
}
