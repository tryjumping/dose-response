use collections::{Deque, RingBuf};

use emhyr::{ComponentManager, ECM, Entity, System};

use components::{AcceptsUserInput, Position};
use systems::input::commands::Command;
use super::super::CommandLogger;


define_system! {
    name: CommandLoggerSystem;
    required_components: AcceptsUserInput, Position;
    resources: commands: RingBuf<Command>, logger: CommandLogger;
    fn process_entity(&mut self, _dt_ms: uint, e: Entity) {
        let ecm = self.ecm();
        match self.commands().front() {
            Some(&command) => self.logger().log(command),
            None => {}
        }
    }
}
