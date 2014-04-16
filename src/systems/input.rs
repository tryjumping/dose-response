use components::{Destination, Player, Position, UsingItem};
use emhyr::{ComponentManager, ECM, Entity};
use std::from_str::FromStr;
use self::commands::*;
use super::super::Resources;
use collections::Deque;

pub mod commands {
    #[deriving(Rand, Show)]
    pub enum Command {
        N, E, S, W, NE, NW, SE, SW,
        Eat,
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
            "Eat" => Some(Eat),
            _ => fail!("Unknown command: '{}'", name)
        }
    }
}

pub fn system(e: Entity,
              ecm: &mut ECM,
              res: &mut Resources) {
    // ensure_components!(ecm, e, AcceptsUserInput, Position);
    if res.side != Player {return}

    // Clean up state from any previous commands
    ecm.remove::<Destination>(e);
    ecm.remove::<UsingItem>(e);

    let pos = ecm.get::<Position>(e);
    match res.commands.pop_front() {
        Some(command) => {
            res.command_logger.log(command);
            match command {
                N => ecm.set(e, Destination{x: pos.x, y: pos.y-1}),
                S => ecm.set(e, Destination{x: pos.x, y: pos.y+1}),
                W => ecm.set(e, Destination{x: pos.x-1, y: pos.y}),
                E => ecm.set(e, Destination{x: pos.x+1, y: pos.y}),

                NW => ecm.set(e, Destination{x: pos.x-1, y: pos.y-1}),
                NE => ecm.set(e, Destination{x: pos.x+1, y: pos.y-1}),
                SW => ecm.set(e, Destination{x: pos.x-1, y: pos.y+1}),
                SE => ecm.set(e, Destination{x: pos.x+1, y: pos.y+1}),

                Eat => {
                    fail!("TODO");
                    // match super::eating::get_first_owned_food(ecm, e) {
                    //     Some(food) => ecm.set_using_item(e, UsingItem{item: food}),
                    //     None => (),
                    // }
                }
            };
        },
        None => (),
    }
}
