use std::collections::VecDeque;

use engine::{Key, KeyCode};


#[derive(Debug)]
pub struct Keys {
    pub keys: VecDeque<Key>,
}



impl Keys {

    pub fn new() -> Self {
        Keys {
            keys: VecDeque::new(),
        }
    }

    /// Return true if the given key is located anywhere in the event buffer.
    pub fn key_pressed(&self, key: Key) -> bool {
        for &pressed_key in self.keys.iter() {
            if pressed_key == key {
                return true;
            }
        }
        false
    }

    /// Consumes the first occurence of the given key in the buffer.
    ///
    /// This is useful when we have a multiple keys in the queue but we want to
    /// check for a presence of a key which should be processed immediately.
    ///
    /// Returns `true` if the key has been in the buffer.
    ///
    /// TODO: investigate using a priority queue instead.
    pub fn read_key(&mut self, key: KeyCode) -> bool {
        let mut len = self.keys.len();
        let mut processed = 0;
        let mut found = false;
        while processed < len {
            match self.keys.pop_front() {
                Some(pressed_key) if !found && pressed_key.code == key => {
                    len -= 1;
                    found = true;
                }
                Some(pressed_key) => {
                    self.keys.push_back(pressed_key);
                }
                None => return false
            }
            processed += 1;
        }
        return found;
    }

}
