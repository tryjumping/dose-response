#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowStack<T> {
    stack: Vec<T>,
}

impl<T: Clone> WindowStack<T> {
    pub fn new(default: T) -> Self {
        WindowStack {
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

    pub fn windows(&self) -> impl Iterator<Item = &T> {
        self.stack.iter()
    }
}
