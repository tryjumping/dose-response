pub trait ComponentType { fn to_index(&self) -> uint; }

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

    pub fn set<T: ComponentType>(&mut self, e: uint, ctype: T, c: C) {
        assert!(self.components[e].len() >= ctype.to_index());
        self.components[e].push(c);
    }

    pub fn get<'r, T: ComponentType>(&'r self, e: uint, ctype: T) -> &'r C {
        assert!(self.components[e].len() > ctype.to_index());
        &self.components[e][ctype.to_index()]
    }
}