extern mod extra;

use std::rand;

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
    rng: rand::IsaacRng,
    map: tcod::TCOD_map_t,
    side: Side,
}

impl world_gen::WorldItem {
    fn to_glyph(self) -> char {
        match self {
            world_gen::Empty => '.',
            world_gen::Tree => '#',
            world_gen::Dose => 'i',
            world_gen::StrongDose => 'I',
            world_gen::Anxiety => 'a',
            world_gen::Depression => 'D',
            world_gen::Hunger => 'h',
            world_gen::Voices => 'v',
            world_gen::Shadows => 's',
        }
    }

    fn to_color(self) -> Color {
        match self {
            world_gen::Empty => col::empty_tile,
            world_gen::Tree => col::tree_1,
            world_gen::Dose => col::dose,
            world_gen::StrongDose => col::dose,

            world_gen::Anxiety => col::anxiety,
            world_gen::Depression => col::depression,
            world_gen::Hunger => col::hunger,
            world_gen::Voices => col::voices,
            world_gen::Shadows => col::shadows,
        }
    }

    fn is_solid(self) -> bool {
        match self {
            world_gen::Empty | world_gen::Dose | world_gen::StrongDose => false,
            _ => true,
        }
    }

    fn is_monster(self) -> bool {
        match self {
            world_gen::Anxiety |
            world_gen::Depression |
            world_gen::Hunger |
            world_gen::Voices |
            world_gen::Shadows => true,
            _ => false,
        }
    }
}

mod col {
    use engine::Color;

    pub static background: Color = Color(0, 0, 0);
    pub static dim_background: Color = Color(15, 15, 15);
    pub static foreground: Color = Color(255, 255, 255);
    pub static anxiety: Color = Color(191,0,0);
    pub static depression: Color = Color(111,63,255);
    pub static hunger: Color = Color(127,101,63);
    pub static voices: Color = Color(95,95,95);
    pub static shadows: Color = Color(95,95,95);
    pub static player: Color = Color(255,255,255);
    pub static empty_tile: Color = Color(223,223,223);
    pub static dose: Color = Color(114,184,255);
    pub static dose_glow: Color = Color(0,63,47);
    pub static tree_1: Color = Color(0,191,0);
    pub static tree_2: Color = Color(0,255,0);
    pub static tree_3: Color = Color(63,255,63);
}

fn initial_state(width: uint, height: uint) -> ~GameState {
    let mut state = ~GameState{
        entities: ~[],
        commands: ~Deque::new::<Command>(),
        rng: rand::rng(),
        map: tcod::map_new(width, height),
        side: Player,
    };
    let mut player = GameObject::new();
    player.accepts_user_input = Some(AcceptsUserInput);
    player.position = Some(Position{x: 10, y: 20});
    player.health = Some(Health(100));
    player.tile = Some(Tile{level: 2, glyph: '@', color: col::player});
    state.entities.push(player);

    let world = world_gen::forrest(&mut state.rng, width, height);
    for world.iter().advance |&(x, y, item)| {
        let mut e = GameObject::new();
        e.position = Some(Position{x: x, y: y});
        e.tile = Some(Tile{level: 0, glyph: item.to_glyph(), color: item.to_color()});
        if item.is_solid() {
            e.solid = Some(Solid);
        }
        if item.is_monster() {
            e.ai = Some(AI);
        }
        state.entities.push(e);
    }

    // Initialise the map's walkability data
    tcod::map_clear(state.map, true, true);
    for state.entities.iter().advance |e| {
        match e.position {
            Some(Position{x, y}) => {
                let solid = e.solid.is_some();
                tcod::map_set_properties(state.map, x as uint, y as uint,
                                         true, !solid);
            },
            None => (),
        }
    }

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
        systems::input_system(e, state.commands, state.side);
        systems::ai_system(e, state.map, state.side);
        systems::movement_system(e, state.map);
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
