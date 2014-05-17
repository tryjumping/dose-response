use ecm::{ComponentManager, ECM, Entity};
use components::{Computer, Player, Side, Turn};

impl ::components::Side {
    fn next(&self) -> Side {
        match *self {
            Player => Computer,
            Computer => Player,
        }
    }

    fn is_last(&self) -> bool {
        *self == Computer
    }
}

define_system! {
    name: TurnSystem;
    resources(ecm: ECM, side: Side, turn: int);
    fn process_all_entities(&mut self, dt_ms: uint, mut entities: &mut Iterator<Entity>) {
        let mut ecm = self.ecm();
        let mut current_side = self.side();
        let switch_sides = ecm.iter().all(|e| {
                match ecm.has::<Turn>(e) {
                    true => {
                        let turn: Turn = ecm.get(e);
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
            for e in ecm.iter() {
                match ecm.has::<Turn>(e) {
                    true => {
                        let turn: Turn = ecm.get(e);
                        if turn.side == *current_side {
                            ecm.set(e, Turn{
                                    ap: turn.max_ap,
                                    .. turn});
                        }
                    },
                    false => (),
                }
            }
        }
    }
}
