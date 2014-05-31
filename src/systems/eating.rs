use components::{AcceptsUserInput, AI, InventoryItem, Edible, ExplosionEffect,
                 Position, Turn, UsingItem};
use ecm::{ComponentManager, ECM, Entity};
use super::combat;
use point;


pub fn get_first_owned_food(ecm: &ECM, owner: Entity) -> Option<Entity> {
    // TODO: sloooooooow. Add some caching like with Position?
    for e in ecm.iter() {
        if ecm.has::<InventoryItem>(e) {
            let item = ecm.get::<InventoryItem>(e);
            if item.owner == owner {
                return Some(e);
            }
        }
    }
    None
}

pub fn explosion<T: point::Point>(ecm: &mut ECM, center: T, radius: int) {
    for (x, y) in point::points_within_radius(center, radius) {
        for e in ecm.entities_on_pos((x, y)) {
            if ecm.has_entity(e) && ecm.has::<AI>(e) {
                combat::kill_entity(e, ecm);
            }
        }
    }
}


define_system! {
    name: EatingSystem;
    components(UsingItem, Position, Turn);
    resources(ecm: ECM);
    fn process_entity(&mut self, dt_ms: uint, entity: Entity) {
        let ecm = &mut *self.ecm();
        let food = ecm.get::<UsingItem>(entity).item;
        if !ecm.has::<Edible>(food) {
            println!("item {:?} isn't edible", food);
            return;
        }
        assert!(ecm.has::<InventoryItem>(food));
        let turn = ecm.get::<Turn>(entity);
        if turn.ap <= 0 {
            return;
        }
        println!("{:?} eats food {:?}", entity, food);
        ecm.remove::<InventoryItem>(food);
        ecm.remove::<UsingItem>(entity);
        ecm.set(entity, turn.spend_ap(1));
        if ecm.has::<ExplosionEffect>(food) {
            println!("Eating kills off nearby enemies");
            let pos = ecm.get::<Position>(entity);
            let radius = ecm.get::<ExplosionEffect>(food).radius;
            explosion(ecm, pos, radius);
        } else {
            println!("The food doesn't have enemy-killing effect.");
        }
    }
}
