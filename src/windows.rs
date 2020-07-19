use serde::{Deserialize, Serialize};

pub mod call_to_action;
pub mod endgame;
pub mod help;
pub mod main_menu;
pub mod message;
pub mod settings;
pub mod sidebar;

/// A stack of windows.
///
/// There's always at least one Window present and there's always at
/// least one that's active ("on top").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Windows<T> {
    stack: Vec<T>,
}

impl<T: Clone> Windows<T> {
    pub fn new(default: T) -> Self {
        Windows {
            stack: vec![default],
        }
    }

    pub fn push(&mut self, window: T) {
        self.stack.push(window);
    }

    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn top(&self) -> T {
        self.stack.last().unwrap().clone()
    }

    pub fn top_mut(&mut self) -> &mut T {
        let idx = self.stack.len() - 1;
        &mut self.stack[idx]
    }

    pub fn windows(&self) -> impl Iterator<Item = &T> {
        self.stack.iter()
    }
}
