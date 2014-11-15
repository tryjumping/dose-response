use std::from_str::FromStr;
use std::time::Duration;
use collections::{RingBuf};

use components::{AcceptsUserInput, Destination, Position, Side, Player};
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
            _ => panic!("Unknown command: '{}'", name)
        }
    }
}


// define_system! {
//     name: InputSystem;
//     resources(player: Entity, commands: RingBuf<Command>, current_side: Side);
//     fn process_all_entities(&mut self, cs: &mut Components, _dt: Duration, entities: Entities) {
//         // Don't process input if it's not your turn (otherwise it will be eaten
//         // & ignored)
//         // (NOTE: only the player can process input for now)
//         if *self.current_side() != Player { return }
//         let player = *self.player();
//         if !cs.has::<AcceptsUserInput>(player) || !cs.has::<Position>(player) { return }

//         // Clean up state from any previous commands
//         cs.unset::<Destination>(player);
//         cs.unset::<UsingItem>(player);

//         let pos = cs.get::<Position>(player);
//         match self.commands().pop_front() {
//             Some(command) => {
//                 match command {
//                     N => cs.set(Destination{x: pos.x, y: pos.y-1}, player),
//                     S => cs.set(Destination{x: pos.x, y: pos.y+1}, player),
//                     W => cs.set(Destination{x: pos.x-1, y: pos.y}, player),
//                     E => cs.set(Destination{x: pos.x+1, y: pos.y}, player),

//                     NW => cs.set(Destination{x: pos.x-1, y: pos.y-1}, player),
//                     NE => cs.set(Destination{x: pos.x+1, y: pos.y-1}, player),
//                     SW => cs.set(Destination{x: pos.x-1, y: pos.y+1}, player),
//                     SE => cs.set(Destination{x: pos.x+1, y: pos.y+1}, player),

//                     Eat => {
//                         match entity_util::get_first_owned_food(player, cs, entities) {
//                             Some(food) => cs.set(UsingItem{item: food}, player),
//                             None => (),
//                         }
//                     }
//                 };
//             },
//             None => (),
//          }
//     }
// }
