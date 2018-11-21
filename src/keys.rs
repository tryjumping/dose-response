use std::{collections::VecDeque, iter::IntoIterator};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub code: KeyCode,
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyCode {
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    D0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    NumPad0,
    NumPad1,
    NumPad2,
    NumPad3,
    NumPad4,
    NumPad5,
    NumPad6,
    NumPad7,
    NumPad8,
    NumPad9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Left,
    Right,
    Up,
    Down,
    Enter,
    Space,
    Esc,
    QuestionMark,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Keys {
    keys: VecDeque<Key>,
}

impl Keys {
    pub fn new() -> Self {
        Keys {
            keys: VecDeque::new(),
        }
    }

    /// Pop the `Key` from the beginning of the queue.
    pub fn get(&mut self) -> Option<Key> {
        self.keys.pop_front()
    }

    /// Return true if any key matches the `predicate`.
    ///
    /// The keys will be checked in order they came in and the first
    /// one that matches will be taken out of the queue.
    pub fn matches<F>(&mut self, predicate: F) -> bool
    where
        F: Fn(Key) -> bool,
    {
        let mut len = self.keys.len();
        let mut processed = 0;
        let mut found = false;
        while processed < len {
            match self.keys.pop_front() {
                Some(pressed_key) if !found && predicate(pressed_key) => {
                    len -= 1;
                    found = true;
                }
                Some(pressed_key) => {
                    self.keys.push_back(pressed_key);
                }
                None => return false,
            }
            processed += 1;
        }
        found
    }

    /// Return true if any key has the specified key code.
    ///
    /// The keys will be checked in order they came in and the first
    /// one that matches will be taken out of the queue.
    pub fn matches_code(&mut self, key_code: KeyCode) -> bool {
        self.matches(|k| k.code == key_code)
    }

    pub fn extend<T: IntoIterator<Item = Key>>(&mut self, iterator: T) {
        self.keys.extend(iterator)
    }

    #[allow(dead_code)]
    pub fn push(&mut self, key: Key) {
        self.keys.push_back(key);
    }
}
