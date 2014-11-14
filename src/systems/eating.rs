use std::time::Duration;
use components::{InventoryItem, Edible, ExplosionEffect,
                 Position, Turn, UsingItem};
use emhyr::{Components, Entity};


// define_system! {
//     name: EatingSystem;
//     components(UsingItem, Position, Turn);
//     resources(position_cache: PositionCache);
//     fn process_entity(&mut self, cs: &mut Components, _dt: Duration, entity: Entity) {
//         let food = cs.get::<UsingItem>(entity).item;
//         if !cs.has::<Edible>(food) {
//             println!("item {} isn't edible", food);
//             return;
//         }
//         assert!(cs.has::<InventoryItem>(food));
//         let turn = cs.get::<Turn>(entity);
//         if turn.ap <= 0 {
//             return;
//         }
//         println!("{} eats food {}", entity, food);
//         cs.unset::<InventoryItem>(food);
//         cs.unset::<UsingItem>(entity);
//         cs.set(turn.spend_ap(1), entity);
//         if cs.has::<ExplosionEffect>(food) {
//             println!("Eating kills off nearby enemies");
//             let pos = cs.get::<Position>(entity);
//             let radius = cs.get::<ExplosionEffect>(food).radius;
//             explosion(&*self.position_cache(), cs, pos, radius);
//         } else {
//             println!("The food doesn't have enemy-killing effect.");
//         }
//     }
// }
