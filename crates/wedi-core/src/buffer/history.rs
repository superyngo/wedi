// 撤銷/重做歷史管理
//
// 歷史以「群組」為單位：一次撤銷還原整個群組。
// - Editor 以 begin_group/end_group 包住每個命令，多行縮排、註解等一次撤銷
// - 連續輸入的單一字元併入前一群組，直到新單字開始（一個單字一次撤銷）
// - 每個群組有唯一 id，用來判斷是否回到存檔點

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

#[derive(Debug, Clone)]
pub struct Group {
    id: u64,
    pub actions: Vec<Action>,
}

pub struct History {
    undo_stack: Vec<Group>,
    redo_stack: Vec<Group>,
    max_size: usize,
    next_id: u64,
    // 目前是否在群組內，以及該群組是否尚未收到任何動作
    in_group: bool,
    group_fresh: bool,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
            next_id: 1,
            in_group: false,
            group_fresh: false,
        }
    }

    /// Start a group: every action pushed until `end_group` undoes as one step.
    pub fn begin_group(&mut self) {
        self.in_group = true;
        self.group_fresh = true;
    }

    pub fn end_group(&mut self) {
        self.in_group = false;
    }

    /// Record an action. `protected_id` is the group at the save point, which must not grow.
    pub fn push(&mut self, action: Action, protected_id: u64) {
        self.redo_stack.clear();
        let starts_group = !self.in_group || self.group_fresh;
        self.group_fresh = false;

        if !starts_group {
            if let Some(top) = self.undo_stack.last_mut() {
                top.actions.push(action);
                return;
            }
        } else if let Some(top) = self.undo_stack.last_mut() {
            if top.id != protected_id && continues_typing(&top.actions, &action) {
                top.actions.push(action);
                return;
            }
        }

        if self.undo_stack.len() >= self.max_size {
            self.undo_stack.remove(0);
        }
        let id = self.next_id;
        self.next_id += 1;
        self.undo_stack.push(Group {
            id,
            actions: vec![action],
        });
    }

    pub fn undo(&mut self) -> Option<Group> {
        let group = self.undo_stack.pop()?;
        self.redo_stack.push(group.clone());
        Some(group)
    }

    pub fn redo(&mut self) -> Option<Group> {
        let group = self.redo_stack.pop()?;
        self.undo_stack.push(group.clone());
        Some(group)
    }

    /// Id of the newest undoable group (0 when there is none).
    pub fn top_id(&self) -> u64 {
        self.undo_stack.last().map_or(0, |g| g.id)
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// 單一字元輸入緊接在前一群組的連續輸入之後，且不是新單字的開頭
fn continues_typing(actions: &[Action], action: &Action) -> bool {
    let Action::Insert { pos, text } = action else {
        return false;
    };
    let mut chars = text.chars();
    let (Some(ch), None) = (chars.next(), chars.next()) else {
        return false;
    };
    if ch == '\n' || ch == '\r' {
        return false;
    }
    // 前一群組必須全是連續的單字元輸入
    let mut expected = None;
    let mut last_char = None;
    for a in actions {
        let Action::Insert { pos: p, text: t } = a else {
            return false;
        };
        if expected.is_some_and(|e| e != *p) || t.contains(['\n', '\r']) {
            return false;
        }
        expected = Some(p + t.chars().count());
        last_char = t.chars().last();
    }
    let word_start = last_char.is_some_and(char::is_whitespace) && !ch.is_whitespace();
    expected == Some(*pos) && !word_start
}

impl Default for History {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ins(pos: usize, text: &str) -> Action {
        Action::Insert {
            pos,
            text: text.to_string(),
        }
    }

    fn typed(history: &mut History, pos: usize, text: &str) {
        history.begin_group();
        history.push(ins(pos, text), 0);
        history.end_group();
    }

    #[test]
    fn test_typing_groups_by_word() {
        let mut history = History::default();
        for (i, c) in "ab cd".chars().enumerate() {
            typed(&mut history, i, &c.to_string());
        }
        assert_eq!(history.undo().unwrap().actions.len(), 2); // "cd"
        assert_eq!(history.undo().unwrap().actions.len(), 3); // "ab "
        assert!(history.undo().is_none());
    }

    #[test]
    fn test_group_collects_all_actions_and_save_point_is_not_extended() {
        let mut history = History::default();
        history.begin_group();
        history.push(ins(0, "x"), 0);
        history.push(ins(5, "y"), 0);
        history.end_group();
        let saved = history.top_id();
        typed(&mut history, 6, "z");
        assert_ne!(history.top_id(), saved);
        assert_eq!(history.undo().unwrap().actions.len(), 1);
        assert_eq!(history.top_id(), saved);
    }
}
