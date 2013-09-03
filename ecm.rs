pub struct EntityManager {
    entities: ~[uint],
    next_entity_id: uint,
}

impl EntityManager {
    pub fn new() -> EntityManager {
        let result = EntityManager{entities: ~[], next_entity_id: 0};
        return result;
    }

    pub fn new_entity(&mut self) -> uint {
        assert!(self.entities.len() == self.next_entity_id);
        self.entities.push(self.next_entity_id);
        self.next_entity_id += 1;
        return self.next_entity_id - 1;
    }
}