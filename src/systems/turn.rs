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
    resources(ecm: ECM, side: Side);
    fn process_all_entities(&mut self, dt_ms: uint, mut entities: &mut Iterator<Entity>) {
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
}
