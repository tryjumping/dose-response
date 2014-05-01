#![feature(macro_rules)]

extern crate collections;

use collections::RingBuf;

struct ECM;
struct Entity;
struct Display;
struct Command;
struct Position;
struct Tile;


macro_rules! define_system (
    {name: $name:ident;
     required_components: $($component:ident),+;
     resources: $($resource:ident : $ty:ty),+;
    } => {
        struct $name {
            ecm: std::rc::Rc<std::cell::RefCell<ECM>>,
            $($resource: std::rc::Rc<std::cell::RefCell<$ty>>),+
        }

        impl $name {
            pub fn new(ecm: std::rc::Rc<std::cell::RefCell<ECM>>,
                       $($resource: std::rc::Rc<std::cell::RefCell<$ty>>),+) -> $name {
                $name {
                    ecm: ecm,
                    $($resource: $resource),+
                }
            }

            pub fn ecm<'a>(&'a self) -> std::cell::RefMut<'a, ECM> {
                self.ecm.borrow_mut()
            }

            $(pub fn $resource<'a>(&'a self) -> std::cell::RefMut<'a, $ty> {self.$resource.borrow_mut()})+

            pub fn valid_entity(&self, e: Entity) -> bool {
                true
                //return self.ecm().has_entity(e) && $(self.ecm().has::<$component>(e))&&+
            }
        }
    }
)

define_system! {
    name: TileSystem;
    required_components: Position, Tile;
    resources: display: Display, commands: RingBuf<Command>;
}

impl TileSystem {
    pub fn process_entity(&mut self, e: Entity) {

    }
}


fn main() {
    use std::rc::Rc;
    use std::cell::RefCell;
    let mut s = TileSystem::new(Rc::new(RefCell::new(ECM)), Rc::new(RefCell::new(Display)), Rc::new(RefCell::new(RingBuf::<Command>::new())));
    s.valid_entity(Entity);
    s.process_entity(Entity);
}
