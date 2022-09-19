#![allow(dead_code)]
#![allow(unused_imports)]

mod commit;
mod util;

// use crate::commit::*;
use crate::util::current_time_string;
use std::io;
use std::thread;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use textwrap;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Widget,
        Wrap,
    },
    Frame, Terminal,
};

enum EditorMode {
    // normal: can select messages and view information about them and select options on them
    // clicking with the mouse enters normal mode; the current input should be saved
    Normal,
    Writing,
    Editing,
}

// Contains the state of the application.
struct App {
    // not really planning to have more than one file open at a time.  it's a diary for god's sake.
    file: commit::File,
    // the state of the current input
    input: String,
    // the current input mode: am I writing right now?
    mode: EditorMode,

    // state of the message view - do I have something selected right now?
    // I either have nothing selected (input, I guess), or I have a message selected.
    // I can press I to enter Input from normal no matter where I am.  I can press E while
    // selecting a message to edit it.
    // ListState works by telling it whe index you are selecting (Some(index)), or nothing if you
    // are not (None).
    selected_msg_state: ListState,
}

impl App {
    fn unselect(&mut self) {
        self.selected_msg_state.select(None);
    }
    fn select_down(&mut self) {
        let idx = match self.selected_msg_state.selected() {
            Some(i) => {
                if i >= self.file.messages.len() - 1 {
                    // Going back to the input bar.
                    None
                } else {
                    Some(i + 1)
                }
            }
            None => {
                if self.file.messages.is_empty() {
                    None
                } else {
                    Some(0)
                }
            }
        };
        self.selected_msg_state.select(idx);
    }
    fn select_up(&mut self) {
        let idx = match self.selected_msg_state.selected() {
            Some(i) => {
                if i == 0 {
                    None
                } else {
                    Some(i - 1)
                }
            }
            None => {
                if self.file.messages.is_empty() {
                    None
                } else {
                    Some(self.file.messages.len() - 1)
                }
            }
        };
        self.selected_msg_state.select(idx);
    }
}

impl Default for App {
    fn default() -> App {
        App {
            file: commit::File::new(),
            input: String::new(),
            mode: EditorMode::Normal,
            selected_msg_state: ListState::default(),
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // app stuff
        // draw the ui first, then determine the app's state.  the app is only concerned with the
        // input bar, the open file...

        // first, need to read the keypress on that frame.
        if let Ok(true) = event::poll(std::time::Duration::from_secs(0)) {
            if let Event::Key(key) = event::read()? {
                match app.mode {
                    EditorMode::Normal => {
                        match key.code {
                            KeyCode::Char('i') => {
                                app.mode = EditorMode::Writing;
                                app.unselect(); // leave message selection
                            }
                            KeyCode::Char('w') => {
                                // TODO: write file; give file name to write to; if no file name provided
                                // then prompt for a valid file name; else, give file's current file
                                // name.
                            }
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Up => app.select_up(),
                            KeyCode::Down => app.select_down(),
                            KeyCode::Esc => app.unselect(),
                            _ => {}
                        }
                    }
                    // TODO: Implement Shift+Tab to go to Normal as well.
                    EditorMode::Writing | EditorMode::Editing => {
                        match key.code {
                            KeyCode::Enter => {
                                app.file.push_string(app.input.drain(..).collect());
                            }
                            KeyCode::Char(c) => {
                                app.input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Esc | KeyCode::BackTab => {
                                app.mode = EditorMode::Normal;
                            }
                            // TODO: implement cursor as well as inserting characters anywhere in the
                            // input bar, not just at the end.
                            _ => {}
                        }
                    }
                }
            } else {
                continue;
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = f.size();

    // calculate the height of the input bar first!  we will need it when making the layouts.
    let input_wrap = textwrap::wrap(
        app.input.as_ref(),
        area.width.checked_sub(2).unwrap_or(1) as usize,
    );
    let input_line_count = usize::max(1, input_wrap.len());

    let max_height = (area.height as f32 * 0.2).ceil() as usize;

    // decide what goes into the displayed input string by using only the last max_height lines.
    let input_str = input_wrap
        .iter()
        .rev()
        .take(max_height)
        .map(|x| x.as_ref())
        .rev()
        .collect::<Vec<&str>>()
        .join("\n");

    let input_bar_height = 2 + input_line_count as u16;
    let vertical_margin = 1;
    let file_view_height = area
        .height
        .checked_sub(2 * vertical_margin)
        .unwrap_or(0)
        .checked_sub(input_bar_height)
        .unwrap_or(0)
        .checked_sub(1)
        .unwrap_or(0); // 1 from the status bar

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(vertical_margin)
        .constraints(
            [
                Constraint::Length(file_view_height),
                Constraint::Length(input_bar_height),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    // ==== MESSAGE VIEW ====

    let message_chunk = Layout::default()
        .horizontal_margin(3)
        .vertical_margin(2)
        .constraints([Constraint::Percentage(100)].as_ref())
        // .split(f.size()); generates an interesting effect; you can draw widgets over others!
        .split(chunks[0]);

    // TODO add dates to each thing on option
    let mut msg_vec = Vec::new();
    for msg in app.file.messages.iter().rev() {
        let mut m = textwrap::fill(
            msg.most_recent()
                .expect("expected the msg to have an actual commit...")
                .data(),
            message_chunk[0].width as usize,
        );
        m.push('\n');
        msg_vec.push(ListItem::new(m));
    }

    let msg_block = Block::default()
        .title(current_time_string())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let msg_widget = List::new(msg_vec)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED)) // Modifier::REVERSED means reversed colors, not reversed text.
        .start_corner(Corner::BottomLeft);
    // .alignment(Alignment::Center)
    // .scroll()        TODO: don't forget about scrolling this later on

    // TODO: eventually, have messages begin displaying from the bottom
    f.render_widget(msg_block, chunks[0]);
    f.render_stateful_widget(msg_widget, message_chunk[0], &mut app.selected_msg_state);

    // ==== INPUT BAR ====

    let input_title = if app.input.is_empty() {
        "Type here"
    } else {
        "Typing..."
    };

    let input_bar = Paragraph::new(input_str).block(
        Block::default()
            .title(input_title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    f.render_widget(input_bar, chunks[1]);

    // ==== STATUS BAR ====

    // TODO: show text: like what mode, what file name, whether editing or writing... etc.
    let (mode_text, mode_color) = match app.mode {
        EditorMode::Normal => ("NORMAL", Color::Blue),
        EditorMode::Writing => ("WRITE", Color::Green),
        EditorMode::Editing => ("EDIT", Color::Red),
    };

    // TODO: make this a spans and calculate the spaces needed to right-justify the file name on
    // the other side, or maybe just use a layout?
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(2)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(chunks[2]);

    let status_bar_mode = Paragraph::new(Span::styled(mode_text, Style::default().fg(mode_color)))
        .block(Block::default().borders(Borders::NONE));
    // TODO change this to display the time instead
    let status_bar_time = Paragraph::new(if app.file.name().is_empty() {
        Span::styled("NEW", Style::default().add_modifier(Modifier::BOLD))
    } else {
        Span::styled(
            app.file.name(),
            Style::default().add_modifier(Modifier::ITALIC),
        )
    })
    .block(Block::default().borders(Borders::NONE))
    .alignment(Alignment::Right);
    f.render_widget(status_bar_mode, status_chunks[0]);
    f.render_widget(status_bar_time, status_chunks[1]);
}

fn main() -> Result<(), io::Error> {
    // raw mode: input is sent raw to the terminal and can be processed as keystrokes.
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    // using stdout, allow us to enter an alternate screen where we can also use the mouse.
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::default();
    run_app(&mut terminal, app)?;

    // Restore terminal.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
