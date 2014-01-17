use components::*;
use self::commands::*;
use super::super::Resources;
use extra::container::Deque;

pub mod commands {
    #[deriving(Rand, ToStr)]
    pub enum Command {
        N, E, S, W, NE, NW, SE, SW,
    }
}

impl FromStr for Command {
    fn from_str(name: &str) -> Option<Command> {
        match name {
            "N" => Some(N),
            "E" => Some(E),
            "S" => Some(S),
            "W" => Some(W),
            "NE" => Some(NE),
            "NW" => Some(NW),
            "SE" => Some(SE),
            "SW" => Some(SW),
            _ => fail!("Unknown command: '{}'", name)
        }
    }
}

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, AcceptsUserInput, Position);
    if res.side != Player {return}

    let pos = ecm.get_position(e);
    match res.commands.pop_front() {
        Some(command) => {
            res.command_logger.log(command);
            let dest = match command {
                N => Destination{x: pos.x, y: pos.y-1},
                S => Destination{x: pos.x, y: pos.y+1},
                W => Destination{x: pos.x-1, y: pos.y},
                E => Destination{x: pos.x+1, y: pos.y},

                NW => Destination{x: pos.x-1, y: pos.y-1},
                NE => Destination{x: pos.x+1, y: pos.y-1},
                SW => Destination{x: pos.x-1, y: pos.y+1},
                SE => Destination{x: pos.x+1, y: pos.y+1},
            };
            ecm.set_destination(e, dest);
        },
        None => (),
    }
}
