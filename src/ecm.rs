use emhyr::EntityIterator;

pub use emhyr::{ComponentManager, Entity, System, World};


pub struct ECM {
    ecm: ::emhyr::ECM,
}

impl ECM {
    pub fn new() -> ECM {
        ECM{ecm: ::emhyr::ECM::new()}
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
        self.ecm.set(entity, component)
    }

    fn has<T: 'static>(&self, entity: Entity) -> bool {
        self.ecm.has::<T>(entity)
    }

    fn get<T: 'static+Clone>(&self, entity: Entity) -> T {
        self.ecm.get::<T>(entity)
    }

    fn remove<T: 'static>(&mut self, entity: Entity) {
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
