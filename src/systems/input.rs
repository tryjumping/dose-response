use components::{AcceptsUserInput, Destination, Player, Position, Side, UsingItem};
use emhyr::{ComponentManager, ECM, Entity};
use emhyr;
use std::from_str::FromStr;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use self::commands::*;
use collections::{Deque, RingBuf};

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

pub struct System {
    ecm: Rc<RefCell<ECM>>,
    commands: Rc<RefCell<RingBuf<Command>>>,
    player: Entity,
    current_side: Rc<Cell<Side>>,
}

impl System {
    pub fn new(ecm: Rc<RefCell<ECM>>,
               commands: Rc<RefCell<RingBuf<Command>>>,
               player: Entity,
               current_side: Rc<Cell<Side>>) -> System {
        System{
            ecm: ecm,
            commands: commands,
            player: player,
            current_side: current_side,
        }
    }
}

impl emhyr::System for System {
    fn process_entity(&mut self, _dt_ms: uint, e: Entity) {
        let mut ecm = &mut *self.ecm.borrow_mut();
        ensure_components!(ecm, e, AcceptsUserInput, Position);
        if self.current_side.get() != Player {return}

        // Clean up state from any previous commands
        ecm.remove::<Destination>(e);
        ecm.remove::<UsingItem>(e);

        let pos = ecm.get::<Position>(e);
        match self.commands.borrow_mut().pop_front() {
            Some(command) => {
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
}
