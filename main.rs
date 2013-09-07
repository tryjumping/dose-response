use components::*;

mod components;
mod ecm;
mod engine;
mod tcod;

fn generate_world(w: uint, h: uint) -> ~[(uint, uint, char)] {
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut result: ~[(uint, uint, char)] = ~[];
    for std::uint::range(0, w) |x| {
        for std::uint::range(0, h) |y| {
            result.push((x, y, chars[(x * y) % chars.char_len()] as char));
        }
    }
    return result;
}

fn draw(layers: &[tcod::TCOD_console_t], world: &~[(uint, uint, char)], width: uint, height: uint) {
    let con = layers[layers.len() - 1];
    for world.iter().advance |&(x, y, glyph)| {
        tcod::console_set_char_background(con, x, y,
                                          tcod::TCOD_color_t{r: 30, g: 30, b: 30},
                                          tcod::TCOD_BKGND_SET);
        tcod::console_put_char(con, x, y, glyph, tcod::TCOD_BKGND_DEFAULT);
    }
    tcod::console_print_ex(con, width - 1, height-1,
                           tcod::TCOD_BKGND_NONE, tcod::TCOD_RIGHT,
                           fmt!("FPS: %?", tcod::sys_get_fps()));
}


fn debug_system(entity: &GameObject) {
    println(fmt!("DEBUG processing entity: %?", entity));
}

fn tile_system(entity: &GameObject) {
    if entity.position.is_none() { return }
    let pos = entity.position.get();
    println(fmt!("Tile system renders tile on pos: %?", pos));
}

fn health_system(entity: &mut GameObject) {
    if entity.health.is_none() { return }
    let health = *entity.health.get();
    entity.health = Some(Health(health - 1));
}

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
        debug_system(e);
        tile_system(e);
        health_system(e);
    }
    engine::Running
}


fn main() {
    engine::main_loop(initial_state, update);
}
