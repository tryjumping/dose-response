macro_rules! ensure_components(
    ($ecm:expr, $entity:expr, $($component:ident),+) => (
        if !$ecm.has_entity($entity) || $(!$ecm.has::<$component>($entity))||+ {return}
    )
)

macro_rules! define_system (
    {name: $name:ident;
     required_components: $($component:ident),+;
     resources: $($resource:ident : $ty:ty),+;
     fn process_entity(&mut self, $dt_ms:ident : $uint_type:ty, $entity:ident : $entity_type:ty) $process_entity_body:expr
    } => {
        pub struct $name {
            ecm: ::std::rc::Rc<::std::cell::RefCell<ECM>>,
            $($resource: ::std::rc::Rc<::std::cell::RefCell<$ty>>),+
        }

        impl $name {
            pub fn new(ecm: ::std::rc::Rc<::std::cell::RefCell<ECM>>,
                       $($resource: ::std::rc::Rc<::std::cell::RefCell<$ty>>),+) -> $name {
                $name {
                    ecm: ecm,
                    $($resource: $resource),+
                }
            }

            pub fn ecm<'a>(&'a self) -> ::std::cell::RefMut<'a, ECM> {
                self.ecm.borrow_mut()
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
        }
    }
)


// pub mod addiction;
pub mod addiction_graphics;
// pub mod ai;
// pub mod bump;
pub mod color_fade;
// pub mod combat;
pub mod command_logger;
// pub mod dose;
// pub mod eating;
// pub mod effect_duration;
// pub mod exploration;
// pub mod fade_out;
pub mod gui;
pub mod input;
// pub mod interaction;
// pub mod leave_area;
pub mod movement;
// pub mod panic;
// pub mod player_dead;
// pub mod stun;
pub mod tile;
pub mod turn;
// pub mod turn_tick_counter;
// pub mod will;
