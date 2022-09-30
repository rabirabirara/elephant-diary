#![allow(dead_code)]
#![allow(unused_imports)]

mod app;
mod commit;
mod config;
mod text;
mod ui;
mod util;

use app::app::*;

use crate::ui::edit::*;
use crate::ui::help::*;
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
    // if there are no routes, don't render anything; the app will close anyway once this function
    // is done. remember, ui draws before app logic updates.
    if let Some(route) = app.route() {
        match route {
            AppRoute::Start => start_screen(f, app),
            AppRoute::Edit => edit_screen(f, app),
            AppRoute::Help => help_screen(f, app),
            AppRoute::PreQuit => prequit_screen(f, app),
            // AppRoute::Quit => (), // TODO might want to provide quitting routines later on.
        }
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


fn prequit_screen<B: Backend>(f: &mut Frame<B>, _app: &mut App) {
    // TODO implement quit screen that is merely a popup window asking if you want to save unsaved
    // work
    // TODO ADD THE ELEPHANT
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
