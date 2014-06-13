macro_rules! define_system (
    {name: $name:ident;
     components(
         $($component:ident),+
     );
     resources(
         $($resource:ident : $ty:ty),*
     );
     fn process_entity(&mut self, $dt_ms:ident : $uint_type:ty, $entity:ident : $entity_type:ty) $process_entity_body:expr
    } => {
        pub struct $name {
            $($resource: ::std::rc::Rc<::std::cell::RefCell<$ty>>),+
        }

        impl $name {
            pub fn new($($resource: ::std::rc::Rc<::std::cell::RefCell<$ty>>),+) -> $name {
                $name {
                    $($resource: $resource),+
                }
            }

            $(pub fn $resource<'a>(&'a self) -> ::std::cell::RefMut<'a, $ty> {self.$resource.borrow_mut()})+
        }

        impl ::emhyr::System for $name {
            fn valid_entity(&self, e: $entity_type) -> bool {
                let ecm = self.ecm.borrow();
                ecm.has_entity(e) && $(ecm.has::<$component>(e))&&+
            }

            fn process_entity(&mut self, $dt_ms: $uint_type, $entity: $entity_type) {
                $process_entity_body
            }

            fn name(&self) -> &str {
                stringify!($name)
            }
        }
    };
    {name: $name:ident;
     resources(
         $($resource:ident : $ty:ty),*
     );
     fn process_all_entities(&mut self, $dt_ms:ident : $uint_type:ty, mut $entities:ident : $entity_iter_type:ty) $process_all_entities_body:expr
    } => {
        pub struct $name {
            $($resource: ::std::rc::Rc<::std::cell::RefCell<$ty>>),+
        }

        impl $name {
            pub fn new($($resource: ::std::rc::Rc<::std::cell::RefCell<$ty>>),+) -> $name {
                $name {
                    $($resource: $resource),+
                }
            }

            $(pub fn $resource<'a>(&'a self) -> ::std::cell::RefMut<'a, $ty> {self.$resource.borrow_mut()})+
        }

        impl ::emhyr::System for $name {
            fn process_all_entities(&mut self, $dt_ms: $uint_type, mut $entities: $entity_iter_type) {
                $process_all_entities_body
            }

            fn name(&self) -> &str {
                stringify!($name)
            }
        }
    }
)


pub mod addiction;
pub mod addiction_graphics;
pub mod ai;
pub mod bump;
pub mod color_animation;
pub mod combat;
pub mod command_logger;
pub mod dose;
pub mod eating;
pub mod exploration;
pub mod fade_out;
pub mod gui;
pub mod input;
pub mod interaction;
// pub mod leave_area;
pub mod movement;
pub mod panic;
pub mod panic_effect_duration;
pub mod stun;
pub mod stun_effect_duration;
pub mod tile;
pub mod turn;
pub mod turn_tick_counter;
pub mod will;
