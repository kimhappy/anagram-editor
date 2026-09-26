#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History<T> {
    undo: Vec<T>,
    redo: Vec<T>,
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }
}

impl<T: PartialEq> History<T> {
    pub fn record(&mut self, before: T, after: &T) {
        if before != *after {
            self.undo.push(before);
            self.redo.clear();
        }
    }

    pub fn undo(&mut self, current: T) -> Option<T> {
        let restored = self.undo.pop()?;
        self.redo.push(current);
        Some(restored)
    }

    pub fn redo(&mut self, current: T) -> Option<T> {
        let restored = self.redo.pop()?;
        self.undo.push(current);
        Some(restored)
    }
}

#[cfg(test)]
mod tests {
    use super::History;

    #[test]
    fn undo_and_redo_walk_the_recorded_changes() {
        let mut history = History::default();
        history.record(1, &2);
        history.record(2, &2);
        history.record(2, &3);
        assert_eq!(history.undo(3), Some(2));
        assert_eq!(history.undo(2), Some(1));
        assert_eq!(history.undo(1), None);
        assert_eq!(history.redo(1), Some(2));
        history.record(2, &5);
        assert_eq!(history.redo(5), None);
        assert_eq!(history.undo(5), Some(2));
    }
}
