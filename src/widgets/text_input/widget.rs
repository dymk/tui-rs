use std::borrow::Cow;

use crate::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, InteractiveWidget, Paragraph},
};

use super::state::TextInputState;

#[derive(Debug, Clone)]
pub struct TextInput<'a> {
    // Block to draw the text input inside (convenience function) - default: None
    optional_block: Option<Block<'a>>,
    // Placeholder text - what's shown if the state value is "" - default: None
    placeholder: Option<Text<'a>>,
    // Render as a read-only input - that is, it will not be focused - default: false
    disable_cursor: bool,
    // Style to render the widget when focused - default: Bold style
    focused_style: Style,
    // Style to apply to displayed text - overriden by focused_style when focused
    text_style: Style,
    alignment: Alignment,
}

impl<'a> TextInput<'a> {
    pub fn new() -> TextInput<'a> {
        Default::default()
    }

    pub fn block(mut self, block: Block<'a>) -> TextInput<'a> {
        self.optional_block = Some(block);
        self
    }

    pub fn disable_cursor(mut self, disable_cursor: bool) -> TextInput<'a> {
        self.disable_cursor = disable_cursor;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> TextInput<'a> {
        self.alignment = alignment;
        self
    }

    pub fn placeholder_text<T>(mut self, placeholder_text: T) -> TextInput<'a>
    where
        T: Into<Cow<'a, str>>,
    {
        self.placeholder = Some(
            Span::styled(
                placeholder_text,
                Style::default()
                    .fg(Color::Black)
                    .add_modifier(Modifier::ITALIC),
            )
            .into(),
        );
        self
    }

    pub fn placeholder(mut self, placeholder: Text<'a>) -> TextInput<'a> {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn focused_style(mut self, style: Style) -> TextInput<'a> {
        self.focused_style = style;
        self
    }

    pub fn text_style(mut self, style: Style) -> TextInput<'a> {
        self.text_style = style;
        self
    }
}

impl<'a> Default for TextInput<'a> {
    fn default() -> Self {
        Self {
            optional_block: Default::default(),
            placeholder: Default::default(),
            disable_cursor: false,
            focused_style: Style::default().add_modifier(Modifier::BOLD),
            text_style: Default::default(),
            alignment: Alignment::Left,
        }
    }
}

impl<'a> InteractiveWidget for TextInput<'a> {
    type State = TextInputState;

    fn render<'b, B: crate::backend::Backend + 'b>(
        mut self,
        area: Rect,
        frame: &mut crate::Frame<'b, B>,
        state: &Self::State,
    ) {
        let is_focused = state.is_focused();

        let area = if let Some(block) = self.optional_block.take() {
            let block = if is_focused {
                block.style(self.focused_style)
            } else {
                block
            };

            let inner = block.inner(area);
            frame.render_widget(block, area);
            inner
        } else {
            area
        };

        let contents = if state.get_value().is_empty() {
            match self.placeholder {
                Some(placeholder) => placeholder,
                None => "".into(),
            }
        } else {
            let value = state.get_value();
            if is_focused {
                Span::styled(value, self.focused_style).into()
            } else {
                Span::styled(value, self.text_style).into()
            }
        };

        let paragraph = Paragraph::new(contents).alignment(self.alignment);
        frame.render_widget(paragraph, area);
        if !self.disable_cursor && is_focused {
            frame.set_cursor(area.x + (state.get_cursor_pos() as u16), area.y);
        }
    }
}
