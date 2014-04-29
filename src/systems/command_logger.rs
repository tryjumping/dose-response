use std::rc::Rc;
use std::cell::RefCell;
use collections::{Deque, RingBuf};

use emhyr::{ComponentManager, ECM, Entity};
use emhyr;

use components::{AcceptsUserInput, Position};
use systems::input::commands::Command;
use super::super::CommandLogger;


pub struct System {
    ecm: Rc<RefCell<ECM>>,
    commands: Rc<RefCell<RingBuf<Command>>>,
    logger: Rc<RefCell<CommandLogger>>,
}

impl System {
    pub fn new(ecm: Rc<RefCell<ECM>>,
               commands: Rc<RefCell<RingBuf<Command>>>,
               logger: Rc<RefCell<CommandLogger>>) -> System {
        System {
            ecm: ecm,
            commands: commands,
            logger: logger,
        }
    }
}

impl emhyr::System for System {
    fn process_entity(&mut self, _dt_ms: uint, e: Entity) {
        let ecm = &*self.ecm.borrow_mut();
        ensure_components!(ecm, e, AcceptsUserInput, Position);
        match self.commands.borrow_mut().front() {
            Some(&command) => self.logger.borrow_mut().log(command),
            None => {}
        }
    }
}
