use std::time::{Duration, Instant};

use crate::quirpy_encoder::{self, EcLevel, Symbol};
use crate::quirpy_front::form::ProjectState;
use crate::quirpy_payload::{self, PayloadError};

const DEBOUNCE: Duration = Duration::from_millis(1000);

#[derive(Clone, PartialEq)]
struct Key {
    payload: Result<String, PayloadError>,
    ec: EcLevel,
}

pub enum Output {
    Symbol { payload: String, symbol: Symbol },
    Invalid(String),
}

pub struct Generator {
    output: Option<Output>,
    settled: Option<Key>,
    pending: Option<Key>,
    pending_since: Instant,
    pending_focus: Option<egui::Id>,
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            output: None,
            settled: None,
            pending: None,
            pending_since: Instant::now(),
            pending_focus: None,
        }
    }
}

impl Generator {
    pub fn output(&self) -> Option<&Output> {
        self.output.as_ref()
    }

    pub fn tick(&mut self, project: &ProjectState, ctx: &egui::Context) {
        let key = Key {
            payload: quirpy_payload::build(project.data_type, &project.fields),
            ec: project.ec_level,
        };

        if self.settled.as_ref() == Some(&key) {
            self.pending = None;
            return;
        }

        if self.pending.as_ref() != Some(&key) {
            self.pending = Some(key.clone());
            self.pending_since = Instant::now();
            self.pending_focus = ctx.memory(|memory| memory.focused());
        }

        let focus_now = ctx.memory(|memory| memory.focused());
        let elapsed = self.pending_since.elapsed();
        if !should_encode(self.pending_focus, focus_now, elapsed) {
            ctx.request_repaint_after(DEBOUNCE - elapsed);
            return;
        }

        self.output = Some(encode(&key));
        self.settled = self.pending.take();
    }
}

fn encode(key: &Key) -> Output {
    let payload = match &key.payload {
        Ok(payload) => payload,
        Err(error) => return Output::Invalid(error.to_string()),
    };

    match quirpy_encoder::encode(payload, key.ec) {
        Ok(symbol) => Output::Symbol {
            payload: payload.clone(),
            symbol,
        },
        Err(error) => Output::Invalid(error.to_string()),
    }
}

// Waiting only while the same widget still holds keyboard focus is what makes a combo box, a
// checkbox, a colour picker, undo and Open all regenerate immediately: none of them holds focus.
fn should_encode(
    pending_focus: Option<egui::Id>,
    focus_now: Option<egui::Id>,
    elapsed: Duration,
) -> bool {
    !(focus_now.is_some() && focus_now == pending_focus && elapsed < DEBOUNCE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str) -> Option<egui::Id> {
        Some(egui::Id::new(name))
    }

    const TYPING: Duration = Duration::from_millis(120);
    const QUIET: Duration = Duration::from_millis(1200);

    #[test]
    fn typing_in_a_text_field_defers() {
        assert!(!should_encode(field("url"), field("url"), TYPING));
    }

    #[test]
    fn a_second_of_quiet_fires() {
        assert!(should_encode(field("url"), field("url"), QUIET));
    }

    #[test]
    fn tabbing_to_another_field_fires_immediately() {
        assert!(should_encode(field("url"), field("name"), TYPING));
    }

    #[test]
    fn clicking_away_to_nothing_fires_immediately() {
        assert!(should_encode(field("url"), None, TYPING));
    }

    #[test]
    fn a_change_made_without_focus_fires_immediately() {
        assert!(should_encode(None, None, Duration::ZERO));
    }

    #[test]
    fn a_change_made_without_focus_fires_even_if_a_field_gains_focus() {
        assert!(should_encode(None, field("url"), Duration::ZERO));
    }
}
