extern crate core;
extern crate crossterm;
extern crate tui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    interactive_form::{self, InteractiveForm},
    layout::{Alignment, Constraint, Direction, Layout},
    macros::interactive_form,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Cell, InteractiveWidgetState, InteractiveWidgetValue, List, ListItem,
        Paragraph, Row, Table, TextInput, TextInputState,
    },
    Frame, Terminal,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

#[interactive_form]
struct Inputs {
    pub basic_input: TextInputState,
    pub button: TextInputState,
    pub placeholder_input: TextInputState,
    pub followed_input: TextInputState,
}

#[derive(Default)]
struct App {
    inputs: Inputs,
    events: Vec<Event>,
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::default();
    app.inputs.button.set_value("A Button");

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let event = event::read()?;
        app.events.push(event);

        if app.inputs.handle_event(event).is_consumed() {
            continue;
        }

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Tab => app.inputs.focus_next_input(),
                KeyCode::BackTab => app.inputs.focus_prev_input(),
                _ => {}
            },
            _ => {}
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let layout = Layout::default()
        .horizontal_margin(10)
        .vertical_margin(2)
        .constraints(
            [
                Constraint::Length(10),
                Constraint::Length(14),
                Constraint::Length(6),
                Constraint::Percentage(100),
            ]
            .as_ref(),
        )
        .split(f.size());

    let info_block = Paragraph::new(vec![
        Spans::from(Span::raw("Press 'TAB' to go to the next input")),
        Spans::from(Span::raw("Press 'SHIFT+TAB' to go to the previous input")),
        Spans::from(Span::raw("Press 'q' to quit when no input is focused")),
        Spans::from(Span::raw(
            "Supports a subset of readline keyboard shortcuts:",
        )),
        Spans::from(Span::raw(
            " - ctrl+e / ctrl+a to jump to text input end / start",
        )),
        Spans::from(Span::raw(
            " - ctrl+w delete to the start of the current word",
        )),
        Spans::from(Span::raw(
            " - alt+b / alt+f to jump backwards / forwards a word",
        )),
        Spans::from(Span::raw(" - left / right arrow keys to move the cursor")),
    ])
    .block(Block::default().title("Information").borders(Borders::ALL));
    f.render_widget(info_block, layout[0]);

    let inputs_block = Block::default().title("Inputs").borders(Borders::ALL);
    let inputs_rect = inputs_block.inner(layout[1]);
    f.render_widget(inputs_block, layout[1]);

    let inputs_layout = Layout::default()
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(inputs_rect);

    {
        let with_button = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(100), Constraint::Length(20)].as_ref())
            .split(inputs_layout[0]);

        let text_input =
            TextInput::new().block(Block::default().title("Basic Input").borders(Borders::ALL));
        f.render_interactive(text_input, with_button[0], &app.inputs.basic_input);

        let button = TextInput::new()
            .alignment(Alignment::Center)
            .disable_cursor(true)
            .block(Block::default().borders(Borders::ALL));
        f.render_interactive(button, with_button[1], &app.inputs.button);
    }

    {
        let text_input = TextInput::new()
            .block(
                Block::default()
                    .title("Has Placeholder")
                    .borders(Borders::ALL),
            )
            .placeholder_text("Type something...");
        f.render_interactive(text_input, inputs_layout[1], &app.inputs.placeholder_input);
    }
    {
        let text_input = TextInput::new()
            .text_style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .title("Followed Input")
                    .borders(Borders::ALL),
            );
        f.render_interactive(text_input, inputs_layout[2], &app.inputs.followed_input);
    }
    {
        let text_input = TextInput::new()
            .text_style(Style::default().fg(Color::LightBlue))
            .disable_cursor(true)
            .focused_style(Style::default())
            .block(
                Block::default()
                    .title("Follows Above (read only)")
                    .borders(Borders::ALL),
            );
        f.render_interactive(text_input, inputs_layout[3], &app.inputs.followed_input);
    }

    let table = Table::new(
        app.inputs
            .as_list()
            .iter()
            .map(|(input_name, input_state)| -> Row {
                Row::new(vec![
                    Cell::from(Span::raw(input_name.to_string())),
                    Cell::from(Span::styled(
                        match input_state.value() {
                            InteractiveWidgetValue::Bool(b) => {
                                if b {
                                    "true"
                                } else {
                                    "false"
                                }
                            }
                            InteractiveWidgetValue::String(s) => s,
                            InteractiveWidgetValue::None => "(none)",
                        },
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                ])
            })
            .collect::<Vec<_>>(),
    )
    .widths(&[Constraint::Min(20), Constraint::Percentage(100)])
    .block(Block::default().title("Input Values").borders(Borders::ALL));
    f.render_widget(table, layout[2]);

    let events = List::new(
        app.events
            .iter()
            .rev()
            .map(|event| ListItem::new(Span::raw(format!("{:?}", event))))
            .collect::<Vec<_>>(),
    )
    .block(Block::default().title("Events").borders(Borders::ALL));
    f.render_widget(events, layout[3]);
}
