use std::from_str::FromStr;
use std::time::Duration;
use collections::{Deque, RingBuf};

use components::{AcceptsUserInput, Destination, Position, UsingItem, Side, Player};
use emhyr::{Components, Entity};
use self::commands::*;
use entity_util;


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


define_system! {
    name: InputSystem;
    components(AcceptsUserInput, Position);
    resources(commands: RingBuf<Command>, current_side: Side);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, e: Entity) {
        // Don't process input if it's not your turn (otherwise it will be eaten
        // & ignored)
        // (NOTE: only the player can process input for now)
        if *self.current_side() != Player { return }

        // Clean up state from any previous commands
        cs.unset::<Destination>(e);
        cs.unset::<UsingItem>(e);

        let pos = cs.get::<Position>(e);
        match self.commands().pop_front() {
            Some(command) => {
                match command {
                    N => cs.set(Destination{x: pos.x, y: pos.y-1}, e),
                    S => cs.set(Destination{x: pos.x, y: pos.y+1}, e),
                    W => cs.set(Destination{x: pos.x-1, y: pos.y}, e),
                    E => cs.set(Destination{x: pos.x+1, y: pos.y}, e),

                    NW => cs.set(Destination{x: pos.x-1, y: pos.y-1}, e),
                    NE => cs.set(Destination{x: pos.x+1, y: pos.y-1}, e),
                    SW => cs.set(Destination{x: pos.x-1, y: pos.y+1}, e),
                    SE => cs.set(Destination{x: pos.x+1, y: pos.y+1}, e),

                    Eat => {
                        match entity_util::get_first_owned_food(cs, e) {
                            Some(food) => cs.set(UsingItem{item: food}, e),
                            None => (),
                        }
                    }
                };
            },
            None => (),
         }
    }
}
