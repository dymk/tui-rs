use std::any::Any;

use crossterm::event::Event;

pub trait InteractiveWidgetState: std::fmt::Debug {
    /// Handle an event, updating the widget state if needed
    fn handle_event(&mut self, _event: Event) -> InteractionOutcome {
        InteractionOutcome::Bubble
    }

    /// Did the state change in response to the last handle_event?
    fn changed(&self) -> bool;

    /// Is the widget currently being focused?
    fn is_focused(&self) -> bool;

    /// Focus the widget
    fn focus(&mut self);

    /// Unfocus the widget
    fn unfocus(&mut self);

    /// For downcasting a &dyn Self into a concrete type
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

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
