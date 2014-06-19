use components::{InventoryItem, Edible, ExplosionEffect,
                 Position, Turn, UsingItem};
use ecm::{ComponentManager, ECM, Entity};
use entity_util;


define_system! {
    name: EatingSystem;
    components(UsingItem, Position, Turn);
    resources(ecm: ECM);
    fn process_entity(&mut self, _dt_ms: uint, entity: Entity) {
        let ecm = &mut *self.ecm();
        let food = ecm.get::<UsingItem>(entity).item;
        if !ecm.has::<Edible>(food) {
            println!("item {} isn't edible", food);
            return;
        }
        assert!(ecm.has::<InventoryItem>(food));
        let turn = ecm.get::<Turn>(entity);
        if turn.ap <= 0 {
            return;
        }
        println!("{} eats food {}", entity, food);
        ecm.remove::<InventoryItem>(food);
        ecm.remove::<UsingItem>(entity);
        ecm.set(entity, turn.spend_ap(1));
        if ecm.has::<ExplosionEffect>(food) {
            println!("Eating kills off nearby enemies");
            let pos = ecm.get::<Position>(entity);
            let radius = ecm.get::<ExplosionEffect>(food).radius;
            entity_util::explosion(ecm, pos, radius);
        } else {
            println!("The food doesn't have enemy-killing effect.");
        }
    }
}
