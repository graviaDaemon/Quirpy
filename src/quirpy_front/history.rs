use crate::quirpy_front::form::ProjectState;

const DEPTH: usize = 50;

pub struct History {
    undo: Vec<ProjectState>,
    redo: Vec<ProjectState>,
    last: ProjectState,
}

impl History {
    pub fn new(state: &ProjectState) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            last: state.clone(),
        }
    }

    pub fn reset(&mut self, state: &ProjectState) {
        self.undo.clear();
        self.redo.clear();
        self.last = state.clone();
    }

    // Skipping the commit while a widget holds keyboard focus is what turns a whole text-field
    // edit into a single undo step: the snapshot lands only once focus leaves the field.
    pub fn maybe_commit(&mut self, current: &ProjectState, ctx: &egui::Context) {
        if ctx.memory(|memory| memory.focused()).is_some() {
            return;
        }
        self.commit(current);
    }

    pub fn undo(&mut self, current: &mut ProjectState) {
        if let Some(previous) = self.undo.pop() {
            self.redo.push(current.clone());
            *current = previous;
            self.last = current.clone();
            tracing::debug!(depth = self.undo.len(), "undo");
        }
    }

    pub fn redo(&mut self, current: &mut ProjectState) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(current.clone());
            *current = next;
            self.last = current.clone();
            tracing::debug!(depth = self.redo.len(), "redo");
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn commit(&mut self, current: &ProjectState) {
        if *current == self.last {
            return;
        }
        self.undo
            .push(std::mem::replace(&mut self.last, current.clone()));
        if self.undo.len() > DEPTH {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> ProjectState {
        ProjectState {
            name: name.to_owned(),
            ..ProjectState::default()
        }
    }

    #[test]
    fn commit_then_undo_restores_the_previous_state() {
        let mut state = named("one");
        let mut history = History::new(&state);
        assert!(!history.can_undo());

        state = named("two");
        history.commit(&state);
        assert!(history.can_undo());

        history.undo(&mut state);
        assert_eq!(state.name, "one");
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn redo_reapplies_an_undone_change() {
        let mut state = named("one");
        let mut history = History::new(&state);
        state = named("two");
        history.commit(&state);
        history.undo(&mut state);

        history.redo(&mut state);
        assert_eq!(state.name, "two");
        assert!(!history.can_redo());
        assert!(history.can_undo());
    }

    #[test]
    fn a_new_commit_clears_the_redo_stack() {
        let mut state = named("one");
        let mut history = History::new(&state);
        state = named("two");
        history.commit(&state);
        history.undo(&mut state);
        assert!(history.can_redo());

        state = named("three");
        history.commit(&state);
        assert!(!history.can_redo());
    }

    #[test]
    fn an_unchanged_state_does_not_commit() {
        let state = named("one");
        let mut history = History::new(&state);
        history.commit(&state);
        assert!(!history.can_undo());
    }

    #[test]
    fn the_depth_cap_drops_the_oldest_entry() {
        let mut state = named("start");
        let mut history = History::new(&state);
        for index in 0..(DEPTH + 10) {
            state = named(&format!("step{index}"));
            history.commit(&state);
        }

        assert_eq!(history.undo.len(), DEPTH);
        for _ in 0..DEPTH {
            history.undo(&mut state);
        }
        assert_eq!(state.name, "step9");
    }

    #[test]
    fn reset_clears_both_stacks() {
        let mut state = named("one");
        let mut history = History::new(&state);
        state = named("two");
        history.commit(&state);
        history.undo(&mut state);

        history.reset(&state);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
