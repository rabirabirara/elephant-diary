#![allow(dead_code)]
#![allow(unused_imports)]

mod app;
mod commit;
mod config;
mod gapbuffer;
mod input;
mod util;

use app::*;

use crate::util::current_time_string;
use std::io;
use std::thread;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
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
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
        Widget, Wrap,
    },
    Frame, Terminal,
};

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    app.startup();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        if !app.run()? {
            return Ok(());
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    match app.route {
        AppRoute::Start => start_screen(f, app),
        AppRoute::Edit => edit_screen(f, app),
        AppRoute::PreQuit => prequit_screen(f, app),
        AppRoute::Quit => (), // TODO might want to provide quitting routines later on.
    }
}

fn start_screen<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(f.size());

    let center_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Percentage(100)].as_ref())
        .split(chunks[1]);

    // let middle_block = Block::default().title("mru").borders(Borders::ALL);

    let mru_title = Paragraph::new(Span::styled(
        "MOST RECENTLY USED: ",
        Style::default().add_modifier(Modifier::ITALIC),
    ))
    .block(Block::default().borders(Borders::ALL));

    // * We actually store the mru in the config.
    let mru_list_items: Vec<ListItem> = app
        .config
        .mru
        .iter()
        .map(|x| ListItem::new(x.as_ref()))
        .collect();
    let mru_list = List::new(mru_list_items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(mru_title, center_chunks[0]);
    f.render_stateful_widget(mru_list, center_chunks[1], &mut app.select_state);
}

fn edit_screen<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = f.size();

    let input = if app.edit.is_some() && app.mode == EditorMode::Editing {
        app.edit.as_ref().unwrap().edit_input.clone()
    } else {
        app.input.buffer.to_string()
    };

    // calculate the height of the input bar first!  we will need it when making the layouts.
    let input_wrap = textwrap::wrap(
        input.as_ref(),
        area.width.checked_sub(2).unwrap_or(1) as usize,
    );
    let input_line_count = usize::max(1, input_wrap.len());

    let max_height = (area.height as f32 * 0.4).ceil() as usize;

    // decide what goes into the displayed input string by using only the last max_height lines.
    let input_str = input_wrap
        .iter()
        .rev()
        .take(max_height)
        .map(|x| x.as_ref())
        .rev()
        .collect::<Vec<&str>>()
        .join("\n");

    let message_bar_height = 1; // TODO: later adapt the message bar to be variable size
    let input_bar_height = usize::min(2 + input_line_count, max_height) as u16;
    let vertical_margin = 1;
    let file_view_height = {
        || {
            area.height
                .checked_sub(2 * vertical_margin)?
                .checked_sub(input_bar_height)?
                .checked_sub(1)? // from status bar
                .checked_sub(message_bar_height) // from message bar
        }
    }()
    .unwrap_or(0);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(vertical_margin)
        .constraints(
            [
                Constraint::Length(file_view_height),
                Constraint::Length(input_bar_height),
                Constraint::Length(1),
                Constraint::Length(message_bar_height),
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

    // TODO add dates to each thing on option; remember to convert from file-stored Utc to Local
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
        .title(current_time_string()) // uses Local, not Utc
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

    f.render_widget(msg_block, chunks[0]);
    f.render_stateful_widget(msg_widget, message_chunk[0], &mut app.select_state);

    // ==== INPUT BAR ====

    let input_title = match app.mode {
        EditorMode::Normal => "Type here",
        EditorMode::Writing => "Typing... ",
        EditorMode::Editing => "Editing... ",
        EditorMode::Saving => "Saving!",
    };

    // TODO: store scroll state of this input.
    // scroll state is rows, cols
    let input_bar = Paragraph::new(input_str).scroll((0, 0)).block(
        Block::default()
            .title(input_title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    f.render_widget(input_bar, chunks[1]);

    // TODO: put cursor at the end of the input
    // you're going to need to store a cursor state, and maybe a scroll state?
    match app.mode {
        EditorMode::Normal => {}
        EditorMode::Writing => {}
        EditorMode::Editing => {}
        EditorMode::Saving => {}
    }

    // ==== STATUS BAR ====

    // TODO: show text: like what mode, what file name, whether editing or writing... etc.
    let (mode_text, mode_color) = match app.mode {
        EditorMode::Normal => ("NORMAL", Color::Blue),
        EditorMode::Writing => ("WRITE", Color::Green),
        EditorMode::Editing => ("EDIT", Color::Red),
        EditorMode::Saving => ("NORMAL", Color::Blue),
    };

    // TODO: make this a spans and calculate the spaces needed to right-justify the file name on
    // the other side, or maybe just use a layout?
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(2)
        .constraints([Constraint::Min(10), Constraint::Percentage(100)].as_ref())
        .split(chunks[2]);

    let status_bar_mode = Paragraph::new(Span::styled(mode_text, Style::default().fg(mode_color)))
        .block(Block::default().borders(Borders::NONE));
    let status_bar_title = Paragraph::new(if app.file.name.is_empty() {
        Span::styled("NEW", Style::default().add_modifier(Modifier::BOLD))
    } else {
        Span::styled(
            &app.file.name,
            Style::default().add_modifier(Modifier::ITALIC),
        )
    })
    .block(Block::default().borders(Borders::NONE))
    .alignment(Alignment::Right);
    f.render_widget(status_bar_mode, status_chunks[0]);
    f.render_widget(status_bar_title, status_chunks[1]);

    // ==== MESSAGE BAR ====
    // TODO: create a message bar, like in Vim, that expands as the status message
    // grows.  Put it beneath the current status bar
    let status_message = Paragraph::new(app.status_msg.as_ref());
    f.render_widget(status_message, chunks[3]);

    // ==== SAVE-AS POPUP WINDOW ====
    if app.mode == EditorMode::Saving {
        let center_col = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(f.size());
        let center_row = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(25),
                    Constraint::Length(5),
                    Constraint::Percentage(25),
                ]
                .as_ref(),
            )
            .split(center_col[1]);
        let center = center_row[1];

        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .constraints([Constraint::Length(1), Constraint::Length(2), Constraint::Percentage(100)].as_ref())
            .split(center);

        let popup = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double);
        let note = Paragraph::new("Enter file name: ")
            .block(Block::default().borders(Borders::BOTTOM).border_type(BorderType::Plain))
            .alignment(Alignment::Center);
        let filename = app.temp_input.clone();
        let name = Paragraph::new(filename.as_ref());

        f.render_widget(Clear, center);
        f.render_widget(popup, center);
        f.render_widget(note, popup_layout[1]);
        f.render_widget(name, popup_layout[2]);
    }
}

fn prequit_screen<B: Backend>(f: &mut Frame<B>, _app: &mut App) {
    // TODO implement quit screen that is merely a popup window asking if you want to save unsaved
    // work
    let closing_message = Paragraph::new("Quit? (Y/n)");
    f.render_widget(closing_message, f.size());
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
