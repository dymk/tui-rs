use std::any::Any;

use crossterm::event::Event;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InteractionOutcome {
    Consumed,
    Bubble,
}

impl InteractionOutcome {
    pub fn is_consumed(&self) -> bool {
        matches!(self, InteractionOutcome::Consumed)
    }
    pub fn is_bubble(&self) -> bool {
        matches!(self, InteractionOutcome::Bubble)
    }
}

pub trait InteractiveWidgetState: std::fmt::Debug {
    fn handle_event(&mut self, _event: Event) -> InteractionOutcome {
        InteractionOutcome::Bubble
    }
    fn is_focused(&self) -> bool;
    fn focus(&mut self);
    fn unfocus(&mut self);

    // For downcasting a &dyn Self into a concrete type
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
