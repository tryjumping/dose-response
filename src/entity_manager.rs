use std::iter::{Map, Range, Zip};
use std::vec::{VecIterator, VecMutIterator};

struct EntityManager<E> {
    priv entities: ~[E],
    priv initial_id: ID,
    priv next_id: ID,
}

#[deriving(Eq)]
pub struct ID(int);

impl<E> EntityManager<E> {
    pub fn new() -> EntityManager<E> {
        EntityManager{entities: ~[], initial_id: ID(0), next_id: ID(0)}
    }

    pub fn add(&mut self, entity: E) -> ID {
        self.entities.push(entity);
        self.next_id = ID(*self.next_id + 1);
        ID(*self.next_id - 1)
    }

    pub fn get_ref<'r>(&'r self, id: ID) -> Option<&'r E> {
        let index = *id - *self.initial_id;
        if index >= 0 && index < (self.entities.len() as int) {
            Some(&self.entities[index])
        } else {
            None
        }
    }

    pub fn get_mut_ref<'r>(&'r mut self, id: ID) -> Option<&'r mut E> {
        let index = *id - *self.initial_id;
        if index >= 0 && index < (self.entities.len() as int) {
            Some(&mut self.entities[index])
        } else {
            None
        }
    }

    pub fn take_out(&mut self, id: ID) -> E {
        let index = (*id - *self.initial_id) as uint;
        if index < 0 || index >= self.entities.len() {
            fail!(format!("Invalid entity ID {}", index))
        } else {
            self.entities.remove(index)
        }
    }

    pub fn iter<'r>(&'r self) -> Zip<VecIterator<'r, E>, Map<int, ID, Range<int>>> {
        self.entities.iter().zip(self.id_iter())
    }

    pub fn mut_iter<'r>(&'r mut self) -> Zip<VecMutIterator<'r, E>, Map<int, ID, Range<int>>> {
        let ids = self.id_iter();
        self.entities.mut_iter().zip(ids)
    }

    pub fn id_iter(&self) -> Map<int, ID, Range<int>> {
        range(*self.initial_id, *self.next_id).map(|index| ID(index))
    }

    pub fn clear(&mut self) {
        self.entities.truncate(0);
        self.initial_id = self.next_id;
    }
}


#[cfg(test)]
mod test {
    use super::{EntityManager, ID};

    #[test]
    fn clear() {
        let mut ecm = EntityManager::<~str>::new();
        ecm.add(~"e1");
        ecm.add(~"e2");
        assert_eq!(ecm.id_iter().len(), 2);
        assert_eq!(ecm.iter().len(), 2);
        assert_eq!(ecm.mut_iter().len(), 2);
        assert!(ecm.get_ref(ID(0)).is_some());
        assert!(ecm.get_ref(ID(1)).is_some());
        assert!(ecm.get_ref(ID(2)).is_none());
        for id in ecm.id_iter() {
            assert!(ecm.get_ref(id).is_some());
        }
        for (e, id) in ecm.iter() {
            assert!(ecm.get_ref(id).is_some());
            assert_eq!(e, ecm.get_ref(id).unwrap());
        }
        ecm.clear();
        assert!(ecm.get_ref(ID(0)).is_none());
        assert!(ecm.get_ref(ID(1)).is_none());
        assert_eq!(ecm.id_iter().len(), 0);
        assert_eq!(ecm.iter().len(), 0);
        assert_eq!(ecm.mut_iter().len(), 0);
        ecm.add(~"e3");
        ecm.add(~"e4");
        ecm.add(~"e5");
        assert_eq!(ecm.id_iter().len(), 3);
        assert_eq!(ecm.iter().len(), 3);
        assert_eq!(ecm.mut_iter().len(), 3);
        assert!(ecm.get_ref(ID(0)).is_none());
        assert!(ecm.get_ref(ID(1)).is_none());
        assert!(ecm.get_ref(ID(2)).is_some());
        assert!(ecm.get_ref(ID(3)).is_some());
        assert!(ecm.get_ref(ID(4)).is_some());
        assert!(ecm.get_ref(ID(5)).is_none());
        assert!(ecm.get_ref(ID(6)).is_none());
        for id in ecm.id_iter() {
            assert!(ecm.get_ref(id).is_some());
        }
        for (e, id) in ecm.iter() {
            assert!(ecm.get_ref(id).is_some());
            assert_eq!(e, ecm.get_ref(id).unwrap());
        }
    }

    #[test]
    fn entity_id_on_add() {
        let mut ecm = EntityManager::<~str>::new();
        let e1_id = ecm.add(~"e1");
        let e2_id = ecm.add(~"e2");
        assert!(ecm.get_ref(e1_id).is_some());
        assert_eq!(e1_id, ID(0));
        assert!(ecm.get_ref(e2_id).is_some());
        assert_eq!(e2_id, ID(1));
    }
}
