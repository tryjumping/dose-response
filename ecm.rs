pub struct EntityManager<C> {
    entities: ~[uint],
    next_entity_id: uint,
    components: ~[~[C]],
}

impl<C: Copy> EntityManager<C> {
    pub fn new() -> EntityManager<C> {
        EntityManager{
            entities: ~[],
            next_entity_id: 0,
            components: ~[],
        }
    }

    pub fn new_entity(&mut self) -> uint {
        assert!(self.entities.len() == self.next_entity_id);
        self.entities.push(self.next_entity_id);
        self.components.push(~[]);
        self.next_entity_id += 1;
        return self.next_entity_id - 1;
    }

    pub fn set(&mut self, e: uint, ctype: uint, c: C) {
        assert!(self.components[e].len() >= ctype);
        self.components[e].push(c);
    }

    pub fn get<'r>(&'r self, e: uint, ctype: uint) -> &'r C {
        assert!(self.components[e].len() > ctype);
        &self.components[e][ctype]
    }
}