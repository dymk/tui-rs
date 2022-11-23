use crate::widgets::{InteractionOutcome, InteractiveWidgetState};
use crossterm::event::Event;

use crate::stdlib_utils::AndThenOrOption;

#[cfg(test)]
mod macro_test;

/*
 * Structs that implement `InteractiveFormBacking` hooks will automatically get an
 * implementation of `InteractiveForm`
*/
pub trait InteractiveFormHooks {
    fn get_focused_idx(&self) -> Option<usize>;
    fn set_focused_idx(&mut self, idx: Option<usize>);
    fn input_states_len(&self) -> usize;
    fn input_at_idx(&self, idx: usize) -> Option<&dyn InteractiveWidgetState>;
    fn input_at_idx_mut(&mut self, idx: usize) -> Option<&mut dyn InteractiveWidgetState>;
}

pub trait InteractiveForm {
    fn handle_event(&mut self, event: Event) -> InteractionOutcome;

    fn focus_next_input(&mut self);
    fn focus_prev_input(&mut self);

    fn is_focused(&self) -> bool;

    fn focused_input(&self) -> Option<&dyn InteractiveWidgetState>;
    fn focused_input_mut(&mut self) -> Option<&mut dyn InteractiveWidgetState>;
}

impl<T: InteractiveFormHooks> InteractiveForm for T {
    fn handle_event(&mut self, event: Event) -> InteractionOutcome {
        self.get_focused_idx()
            .map_or(InteractionOutcome::Bubble, |idx| {
                self.input_at_idx_mut(idx)
                    .map_or(InteractionOutcome::Bubble, |state| {
                        state.handle_event(event)
                    })
            })
    }

    fn focus_next_input(&mut self) {
        let input_states_len = self.input_states_len();
        let new_idx = focused_input_and_idx_mut(self).and_then_or(
            |(idx, input)| {
                input.unfocus();
                if idx + 1 == input_states_len {
                    None
                } else {
                    Some(idx + 1)
                }
            },
            Some(0),
        );

        self.set_focused_idx(new_idx);
        if let Some((_, state)) = focused_input_and_idx_mut(self) {
            state.focus()
        }
    }

    fn focus_prev_input(&mut self) {
        let input_states_len = self.input_states_len();
        let new_idx = focused_input_and_idx_mut(self).and_then_or(
            |(idx, input)| {
                input.unfocus();
                if idx == 0 {
                    None
                } else {
                    Some(idx - 1)
                }
            },
            Some(input_states_len - 1),
        );

        self.set_focused_idx(new_idx);
        if let Some((_, state)) = focused_input_and_idx_mut(self) {
            state.focus()
        }
    }

    fn is_focused(&self) -> bool {
        self.get_focused_idx().is_some()
    }

    fn focused_input(&self) -> Option<&dyn InteractiveWidgetState> {
        self.get_focused_idx()
            .and_then(|idx| self.input_at_idx(idx))
    }

    fn focused_input_mut(&mut self) -> Option<&mut dyn InteractiveWidgetState> {
        self.get_focused_idx()
            .and_then(|idx| self.input_at_idx_mut(idx))
    }
}

fn focused_input_and_idx_mut(
    this: &mut dyn InteractiveFormHooks,
) -> Option<(usize, &mut dyn InteractiveWidgetState)> {
    this.get_focused_idx()
        .and_then(|idx| this.input_at_idx_mut(idx).map(|state| (idx, state)))
}
