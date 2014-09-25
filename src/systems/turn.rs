use std::time::Duration;

use emhyr::{Components, Entities};
use components::{Side, Turn};

define_system! {
    name: TurnSystem;
    resources(side: Side, turn: int);
    fn process_all_entities(&mut self, cs: &mut Components, _dt: Duration, entities: Entities) {
        let mut current_side = self.side();
        let switch_sides = entities.clone().all(|e| {
                match cs.has::<Turn>(e) {
                    true => {
                        let turn: Turn = cs.get(e);
                        (*current_side != turn.side) || (turn.ap == 0)
                    },
                    false => true,
                }
            });
        if switch_sides {
            if current_side.is_last() {
                *self.turn() += 1;
            }
            *current_side = current_side.next();
            let mut entities = entities;
            for e in entities {
                match cs.has::<Turn>(e) {
                    true => {
                        let turn: Turn = cs.get(e);
                        if turn.side == *current_side {
                            cs.set(Turn{
                                    ap: turn.max_ap,
                                    .. turn}, e);
                        }
                    },
                    false => (),
                }
            }
        }
    }
}
