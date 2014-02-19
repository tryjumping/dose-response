use components::*;
use self::commands::*;
use super::super::Resources;
use extra::container::Deque;

pub mod commands {
    #[deriving(Rand, ToStr)]
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

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, AcceptsUserInput, Position);
    if res.side != Player {return}

    // Clean up state from any previous commands
    ecm.remove_destination(e);
    ecm.remove_using_item(e);

    let pos = ecm.get_position(e);
    match res.commands.pop_front() {
        Some(command) => {
            res.command_logger.log(command);
            match command {
                N => ecm.set_destination(e, Destination{x: pos.x, y: pos.y-1}),
                S => ecm.set_destination(e, Destination{x: pos.x, y: pos.y+1}),
                W => ecm.set_destination(e, Destination{x: pos.x-1, y: pos.y}),
                E => ecm.set_destination(e, Destination{x: pos.x+1, y: pos.y}),

                NW => ecm.set_destination(e, Destination{x: pos.x-1, y: pos.y-1}),
                NE => ecm.set_destination(e, Destination{x: pos.x+1, y: pos.y-1}),
                SW => ecm.set_destination(e, Destination{x: pos.x-1, y: pos.y+1}),
                SE => ecm.set_destination(e, Destination{x: pos.x+1, y: pos.y+1}),

                Eat => {
                    match super::eating::get_first_owned_food(ecm, e) {
                        Some(food) => ecm.set_using_item(e, UsingItem{item: food}),
                        None => (),
                    }
                }
            };
        },
        None => (),
    }
}
