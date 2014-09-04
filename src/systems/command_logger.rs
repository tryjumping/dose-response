use std::time::Duration;
use std::collections::{Deque, RingBuf};

use components::{AcceptsUserInput, Position};
use emhyr::{Components, Entity};
use systems::input::commands::Command;
use super::super::CommandLogger;


define_system! {
    name: CommandLoggerSystem;
    components(AcceptsUserInput, Position);
    resources(commands: RingBuf<Command>, logger: CommandLogger);
    fn process_entity(&mut self, _cs: &mut Components, _dt: Duration, _e: Entity) {
        match self.commands().front() {
            Some(&command) => self.logger().log(command),
            None => {}
        }
    }
}
