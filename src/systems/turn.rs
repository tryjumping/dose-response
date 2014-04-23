use emhyr::{ComponentManager, ECM};
use components;
use components::*;

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

pub fn system(ecm: &mut ECM,
              ) {
    fail!("TODO");
    // let switch_sides = ecm.iter().all(|e| {
    //         match ecm.has::<Turn>(e) {
    //             true => {
    //                 let turn: Turn = ecm.get(e);
    //                 (res.side != turn.side) || (turn.ap == 0)
    //             },
    //             false => true,
    //         }
    //     });
    // if switch_sides {
    //     if res.side.is_last() {
    //         res.turn += 1;
    //     }
    //     res.side = res.side.next();
    //     for e in ecm.iter() {
    //         match ecm.has::<Turn>(e) {
    //             true => {
    //                 let turn: Turn = ecm.get(e);
    //                 if turn.side == res.side {
    //                     ecm.set(e, Turn{
    //                             ap: turn.max_ap,
    //                             .. turn});
    //                 }
    //             },
    //             false => (),
    //         }
    //     }
    // }
}
