#[derive(Debug, Clone)]
pub struct TextInputState {
    // Underlying value of the text input field
    pub(super) value: String,
    // Position in the text input to insert / remove text from
    pub(super) cursor_pos: usize,
    // Is the input focused?
    is_focused: bool,
    // Can the input take focus?
    can_take_focus: bool,
    // Did the input change when the last event was handled?
    pub(super) changed: bool,
}

// default initializer for TextInputState from &str
impl From<&str> for TextInputState {
    fn from(value: &str) -> TextInputState {
        TextInputState::with_value(value)
    }
}

impl TextInputState {
    pub fn with_value(value: &str) -> TextInputState {
        TextInputState {
            value: value.to_string(),
            cursor_pos: value.len(),
            ..Default::default()
        }
    }

    pub fn can_take_focus(&mut self, can_take_focus: bool) {
        self.can_take_focus = can_take_focus;
        if !can_take_focus {
            self.unfocus();
        }
    }
    pub fn is_focused(&self) -> bool {
        self.can_take_focus && self.is_focused
    }
    pub fn focus(&mut self) {
        if self.can_take_focus {
            self.is_focused = true;
        }
    }
    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }
    pub fn set_value(&mut self, val: &str) {
        self.value = val.to_string();
        self.cursor_pos = std::cmp::min(self.cursor_pos, self.value.len());
    }
    pub fn set_cursor_pos(&mut self, pos: usize) {
        self.cursor_pos = pos;
    }
    pub fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }
    pub fn get_value(&self) -> &str {
        &self.value
    }
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            value: Default::default(),
            is_focused: false,
            cursor_pos: 0,
            can_take_focus: true,
            changed: false,
        }
    }
}
