use components;
use components::*;
use super::super::Resources;

impl components::Side {
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

pub fn system(ecm: &mut ComponentManager,
              res: &mut Resources) {
    let switch_sides = ecm.iter().all(|e| {
            match ecm.has_turn(e) {
                true => {
                    let turn = ecm.get_turn(e);
                    (res.side != turn.side) || (turn.ap == 0)
                },
                false => true,
            }
        });
    if switch_sides {
        if res.side.is_last() {
            res.turn += 1;
        }
        res.side = res.side.next();
        for e in ecm.iter() {
            match ecm.has_turn(e) {
                true => {
                    let turn = ecm.get_turn(e);
                    if turn.side == res.side {
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
