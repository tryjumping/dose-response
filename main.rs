extern mod extra;

use std::io;
use std::rand;
use std::rand::RngUtil;
use std::os;

use components::*;
use engine::{Display, Color, MainLoopState, Key};
use extra::deque::Deque;
use extra::time;
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
    logger: CommandLogger,
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

fn initial_state(width: uint, height: uint, commands: ~Deque<Command>,
                 rng: rand::IsaacRng, logger: CommandLogger) -> ~GameState {
    let mut state = ~GameState{
        entities: ~[],
        commands: commands,
        rng: rng,
        logger: logger,
        map: tcod::map_new(width, height),
        side: Player,
    };
    let mut player = GameObject::new();
    player.accepts_user_input = Some(AcceptsUserInput);
    player.position = Some(Position{x: 10, y: 20});
    player.health = Some(Health(100));
    player.tile = Some(Tile{level: 2, glyph: '@', color: col::player});
    player.turn = Some(Turn{side: Player, ap: 1, max_ap: 1, spent_this_turn: 0});
    player.solid = Some(Solid);
    state.entities.push(player);

    let world = world_gen::forrest(&mut state.rng, width, height);
    for world.iter().advance |&(x, y, item)| {
        let mut bg = GameObject::new();
        bg.position = Some(Position{x: x, y: y});
        if item == world_gen::Tree {
            bg.tile = Some(Tile{level: 0, glyph: item.to_glyph(), color: item.to_color()});
            bg.solid = Some(Solid);
        } else { // put an empty item as the background
            bg.tile = Some(Tile{level: 0, glyph: world_gen::Empty.to_glyph(), color: world_gen::Empty.to_color()});
        }
        state.entities.push(bg);
        if item != world_gen::Tree && item != world_gen::Empty {
            let mut e = GameObject::new();
            e.position = Some(Position{x: x, y: y});
            let mut level = 1;
            if item.is_monster() {
                e.ai = Some(AI);
                e.turn = Some(Turn{side: Computer, ap: 0, max_ap: 1, spent_this_turn: 0});
                e.solid = Some(Solid);
                level = 2;
            }
            e.tile = Some(Tile{level: level, glyph: item.to_glyph(), color: item.to_color()});
            state.entities.push(e);
        }
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
        systems::turn_system(e, state.side);
        systems::input_system(e, state.commands, state.logger, state.side);
        systems::ai_system(e, &mut state.rng, state.map, state.side);
        systems::movement_system(e, state.map);
        systems::tile_system(e, display);
        systems::idle_ai_system(e, state.side);
    }
    systems::end_of_turn_system(state.entities, &mut state.side);
    engine::Running
}

fn seed_from_str(source: &str) -> ~[u8] {
    match std::int::from_str(source) {
        Some(n) => int_to_bytes(n),
        None => fail!("The seed must be a number"),
    }
}

fn int_to_bytes(n: int) -> ~[u8] {
    do io::with_bytes_writer |wr| {
        do n.iter_bytes(true) |bytes| {
            wr.write(bytes);
            true
        };
    }
}

struct NullWriter;

impl io::Writer for NullWriter {
    fn write(&self, _v: &[u8]) {}
    fn seek(&self, _a: int, _s: io::SeekStyle) {}
    fn tell(&self) -> uint { 0 }
    fn flush(&self) -> int { 0 }
    fn get_type(&self) -> io::WriterType { io::File }
}

struct CommandLogger {
    priv writer: @io::Writer,
}

impl CommandLogger {
    fn log(&self, command: Command) {
        self.writer.write_line(command.to_str());
        self.writer.flush();
    }
}


fn main() {
    let (width, height) = (80, 50);
    let title = "Dose Response";
    let font_path = Path("./fonts/dejavu16x16_gs_tc.png");

    let mut rng = rand::IsaacRng::new();
    let seed: ~[u8];
    let writer: @io::Writer;
    let mut commands = ~Deque::new::<Command>();

    match os::args().len() {
        1 => {  // Run the game with a new seed, create the replay log
            let seed_int = rng.gen_int_range(0, 10000);
            seed = int_to_bytes(seed_int);
            let cur_time = time::now();
            let timestamp = time::strftime("%FT%T.", &cur_time) +
                (cur_time.tm_nsec / 1000000).to_str();
            let replay_path = Path("./replays/replay-" + timestamp);
            match io::file_writer(&replay_path, [io::Create, io::Append]) {
                Ok(w) => {
                    writer = w;
                    writer.write_line(seed_int.to_str());
                },
                Err(e) => fail!(fmt!("Failed to open the replay file: %s", e)),
            };
        },
        2 => {  // Replay the game from the entered log
            let replay_path = &Path(os::args()[1]);
            match io::read_whole_file_str(replay_path) {
                Ok(contents) => {
                    let mut lines_it = contents.any_line_iter();
                    match lines_it.next() {
                        Some(seed_str) => seed = seed_from_str(seed_str),
                        None => fail!(fmt!("The replay file is empty")),
                    }
                    for lines_it.advance |line| {
                        match Command::from_str(line) {
                            Some(command) => commands.add_back(command),
                            None => fail!(fmt!("Unknown command: %?", line)),
                        }
                    }
                    writer = @NullWriter as @Writer;
                },
                Err(e) => fail!(fmt!("Failed to read the replay file: %s", e))
            }
        },
        _ => fail!("You must pass either pass zero or one arguments."),
    };
    rng = rand::IsaacRng::new_seeded(seed);

    let logger = CommandLogger{writer: writer};
    engine::main_loop(width, height, title, font_path,
                      initial_state(width, height, commands, rng, logger),
                      update);
}
