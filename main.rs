extern mod extra;

use components::*;
use engine::{Display, Color, MainLoopState, Key};
use extra::deque::Deque;
use systems::{Command};

mod components;
mod ecm;
mod engine;
mod systems;
pub mod tcod;
mod world_gen;


struct GameState {
    entities: ~[GameObject],
    commands: ~Deque<Command>,
}

impl world_gen::WorldItem {
    fn to_glyph(self) -> char {
        match self {
            world_gen::Empty => '.',
            world_gen::Tree => '#',
            world_gen::Dose => 'i',
            world_gen::Monster => 'a',
        }
    }
}

fn initial_state(width: uint, height: uint) -> ~GameState {
    let mut state = ~GameState{entities: ~[], commands: ~Deque::new::<Command>()};
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
            tile: Some(Tile{level: 0, glyph: item.to_glyph(), color: Color(0, 255, 255)}),
        })
    }
    state.entities.push(GameObject{
        position: Some(Position{x: 1, y: 1}),
        health: None,
        tile: None,
    });
    state
}

fn escape_pressed(keys: &Deque<Key>) -> bool {
    for keys.iter().advance |&key| {
        if key.char as int == 27 { return true; }
    }
    false
}

fn process_input(keys: &mut Deque<Key>, commands: &mut Deque<Command>) {
    while !keys.is_empty() {
        let key = keys.pop_front();
        match key.code {
            // Up
            14 => commands.add_back(systems::N),
            // Down
            17 => commands.add_back(systems::S),
            // Left
            15 => match (key.ctrl(), key.shift()) {
                (false, true) => commands.add_back(systems::NW),
                (true, false) => commands.add_back(systems::SW),
                _ => commands.add_back(systems::W),
            },
            // Right
            16 => match (key.ctrl(), key.shift()) {
                (false, true) => commands.add_back(systems::NE),
                (true, false) => commands.add_back(systems::SE),
                _ => commands.add_back(systems::E),
            },
            _ => (),
        }
    }
}



fn update(state: &mut GameState,
          display: &mut Display,
          keys: &mut Deque<Key>) -> MainLoopState {
    if escape_pressed(keys) { return engine::Exit }

    process_input(keys, state.commands);
    for state.entities.mut_iter().advance |e| {
        systems::input_system(e, state.commands);
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
