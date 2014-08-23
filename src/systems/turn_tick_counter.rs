use std::time::Duration;
use components::{Side, Turn};
use emhyr::{Components, Entity};


define_system! {
    name: TurnTickCounterSystem;
    components(Turn);
    resources(side: Side);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, e: Entity) {
        let turn: Turn = cs.get(e);
        if turn.side == *self.side() {
            cs.set(Turn{spent_this_tick: 0, .. turn}, e);
        }
    }
}
