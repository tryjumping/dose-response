use collections::{Deque, RingBuf};

use components::{AcceptsUserInput, Position};
use ecm::{ComponentManager, ECM, Entity};
use systems::input::commands::Command;
use super::super::CommandLogger;


define_system! {
    name: CommandLoggerSystem;
    components(AcceptsUserInput, Position);
    resources(ecm: ECM, commands: RingBuf<Command>, logger: CommandLogger);
    fn process_entity(&mut self, _dt_ms: uint, _e: Entity) {
        match self.commands().front() {
            Some(&command) => self.logger().log(command),
            None => {}
        }
    }
}
