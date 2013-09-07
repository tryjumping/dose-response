use components::*;

mod components;
mod ecm;
mod engine;
mod systems;


struct GameState {
    entities: ~[GameObject],
}

fn initial_state() -> ~GameState {
    let mut state = ~GameState{entities: ~[]};
    state.entities.push(GameObject{
        position: Some(Position{x: 10, y: 20}),
        health: Some(Health(100))});
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
