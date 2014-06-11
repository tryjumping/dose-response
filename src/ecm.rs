use std::any::{Any, AnyRefExt};
use std::collections::HashMap;
use std::intrinsics::TypeId;
use std::vec::MoveItems;
use emhyr::EntityIterator;
use components::Position;


pub use emhyr::{ComponentManager, Entity, System, World};


pub struct ECM {
    ecm: ::emhyr::ECM,
    position_cache: HashMap<(int, int), Vec<Entity>>,
}

impl ECM {
    pub fn new() -> ECM {
        ECM{
            ecm: ::emhyr::ECM::new(),
            position_cache: HashMap::new(),
        }
    }

    pub fn entities_on_pos(&self, pos: (int, int)) -> MoveItems<Entity> {
        match self.position_cache.find(&pos) {
            Some(entities) => entities.clone().move_iter(),
            None => vec![].move_iter(),
        }
    }
}


impl ComponentManager<EntityIterator> for ECM {
    fn new_entity(&mut self) -> Entity { self.ecm.new_entity() }

    fn has_entity(&self, entity: Entity) -> bool {
        self.ecm.has_entity(entity)
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.ecm.remove_entity(entity)
    }

    fn remove_all_entities(&mut self) {
        self.ecm.remove_all_entities()
    }

    fn set<T: Send+Clone>(&mut self, entity: Entity, component: T) {
        match (&component as &Any).as_ref::<Position>() {
            Some(pos) => {
                // Removes any previous position from the cache
                self.remove::<Position>(entity);
                let cache = self.position_cache.find_or_insert((pos.x, pos.y), vec![]);
                cache.push(entity);
            }
            None => {}
        }
        self.ecm.set(entity, component)
    }

    fn has<T: 'static>(&self, entity: Entity) -> bool {
        self.ecm.has::<T>(entity)
    }

    fn get<T: 'static+Clone>(&self, entity: Entity) -> T {
        self.ecm.get::<T>(entity)
    }

    fn remove<T: 'static>(&mut self, entity: Entity) {
        if (TypeId::of::<T>() == TypeId::of::<Position>()) && self.has::<Position>(entity) {
            let pos: Position = self.get(entity);
            let cache = self.position_cache.get_mut(&(pos.x, pos.y));
            let cached_entity_index = match cache.iter().position(|&i| i == entity) {
                Some(index) => index,
                None => fail!("Position cache is missing the entity {}", entity),
            };
            cache.remove(cached_entity_index);
        }
        self.ecm.remove::<T>(entity)
    }

    fn make<T>(&mut self, entity: Entity) {
        self.ecm.make::<T>(entity)
    }

    fn is<T>(&self, entity: Entity) -> bool {
        self.ecm.is::<T>(entity)
    }
    fn iter(&self) -> EntityIterator {
        self.ecm.iter()
    }
}
