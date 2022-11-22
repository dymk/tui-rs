#[cfg(feature = "crossterm")]
pub mod interactive;
mod state;
mod widget;

pub use state::TextInputState;
pub use widget::TextInput;
