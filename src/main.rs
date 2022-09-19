#![allow(dead_code)]

/*
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum EditorMode {
    Normal,
    Editing,
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    mode: EditorMode,
    /// History of recorded messages
    messages: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            mode: EditorMode::Normal,
            messages: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                EditorMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.mode = EditorMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                EditorMode::Editing => match key.code {
                    KeyCode::Enter => {
                        app.messages.push(app.input.drain(..).collect());
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.mode = EditorMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.mode {
        EditorMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        EditorMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.mode {
            EditorMode::Normal => Style::default(),
            EditorMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    match app.mode {
        EditorMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        EditorMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[2]);
}
*/

mod commit;

use crate::commit::*;

use textwrap;
use unicode_linebreak::*;
use unicode_width::UnicodeWidthStr;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::borrow::Cow;
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table, Widget, Wrap,
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
}

impl Default for App {
    fn default() -> App {
        App {
            file: commit::File::new(),
            input: String::new(),
            mode: EditorMode::Normal,
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // app stuff
        // draw the ui first, then determine the app's state.  the app is only concerned with the
        // input bar, the open file...

        // first, need to read the keypress on that frame.
        if let Event::Key(key) = event::read()? {
            match app.mode {
                EditorMode::Normal => {
                    match key.code {
                        KeyCode::Char('i') => {
                            app.mode = EditorMode::Writing;
                        }
                        KeyCode::Char('w') => {
                            // TODO: write file; give file name to write to; if no file name provided
                            // then prompt for a valid file name; else, give file's current file
                            // name.
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
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
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
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

    // TODO add dates to each thing
    let mut msg_str = String::new();
    for msg in app.file.messages.iter() {
        msg_str.push_str(&textwrap::fill(
            msg.most_recent()
                .expect("expected the msg to have an actual commit...")
                .data(),
            message_chunk[0].width as usize,
        ));
        msg_str.push('\n');
    }

    let msg_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let msg_widget = Paragraph::new(Text::from(msg_str))
        .block(
            Block::default()
                // .title(app.file.name())
                // .title("<INSERT CURRENT TIME HERE>")
                .borders(Borders::NONE)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default())
        // .alignment(Alignment::Center)
        // .scroll()        TODO: don't forget about scrolling this later on
        .wrap(Wrap { trim: false });

    // TODO: eventually, have messages begin displaying from the bottom
    f.render_widget(msg_block, chunks[0]);
    f.render_widget(msg_widget, message_chunk[0]);

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
    let status_bar_filename = Paragraph::new(if app.file.name().is_empty() {
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
    f.render_widget(status_bar_filename, status_chunks[1]);
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
