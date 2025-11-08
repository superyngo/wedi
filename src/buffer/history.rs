// 撤銷/重做歷史管理

#[derive(Debug, Clone)]
pub enum Action {
    Insert {
        pos: usize,
        text: String,
    },
    Delete {
        pos: usize,
        text: String,
    },
    DeleteRange {
        start: usize,
        end: usize,
        text: String,
    },
}

pub struct History {
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
    max_size: usize,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, action: Action) {
        if self.undo_stack.len() >= self.max_size {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<Action> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Action> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    #[allow(dead_code)]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(1000)
    }
}
