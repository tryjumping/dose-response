use std::iter::{Enumerate, Map};
use std::vec::{VecIterator, VecMutIterator};

struct EntityManager<E> {
    priv entities: ~[E],
    priv next_id: ID,
}

struct ID(int);

impl<E> EntityManager<E> {
    pub fn new() -> EntityManager<E> {
        EntityManager{entities: ~[], next_id: ID(0)}
    }

    pub fn add(&mut self, entity: E) -> int {
        self.entities.push(entity);
        self.next_id = ID(*self.next_id + 1);
        *self.next_id - 1
    }

    pub fn get_ref<'r>(&'r self, id: ID) -> &'r E {
        &self.entities[*id]
    }

    pub fn get_ref_mut<'r>(&'r mut self, id: ID) -> &'r mut E {
        &mut self.entities[*id]
    }

    pub fn iter<'r>(&'r self) -> Map<(uint, &'r E), (&'r E, ID), Enumerate<VecIterator<'r, E>>> {
        return self.entities.iter().enumerate().map(|(id, e)| (e, ID(id as int)))
    }

    pub fn mut_iter<'r>(&'r mut self) -> Map<(uint, &'r mut E), (&'r mut E, ID), Enumerate<VecMutIterator<'r, E>>> {
        return self.entities.mut_iter().enumerate().map(|(id, e)| (e, ID(id as int)))
    }
}
