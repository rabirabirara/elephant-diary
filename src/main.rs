#![allow(dead_code)]

mod commit;

use crate::commit::*;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, thread, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, Row, Table, Widget},
    Frame, Terminal,
};

fn ui<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(0)
        .constraints(
            [
                Constraint::Percentage(90),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    // pass the block with the file sentences in and render
    // remember to put each widget inside a block
    let block = Block::default().title("Block").borders(Borders::ALL);
    f.render_widget(block, chunks[0]);
    // pass the block with the text input in and render
    let block = Block::default().title("Block 2").borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}

fn main() -> Result<(), io::Error> {
    // let mut f = File::new();
    // let mut m = Message::new();
    // let c1 = Commit::from_data(String::from("HELLO WORLD!"));
    // let c2 = Commit::from_data(String::from("HELLO WORLD, LATER!"));
    // m.push_commit(c1);
    // f.push_msg(m.clone());
    // m.push_commit(c2);
    // f.push_msg(m);
    // f.set_name(String::from("some_file"));

    // let mut input = String::new();
    // while let Ok(_bytes) = std::io::stdin().read_line(&mut input) {
    //     let data = input.clone();
    //
    //     if data.trim().eq(":quit") {
    //         break;
    //     }
    //
    //     f.push_msg(Message::from_commit(Commit::from_data(data)));
    //     input.clear();
    // }
    // println!("{}", f);

    // great; now serialize the file and save it to disk.
    // btw, you might be able to find a better plaintext representation of these files; no need for
    // serialization... like, what for?  hell, if the file itself is readable, even better.

    // raw mode: input is sent raw to the terminal and can be processed as keystrokes.
    // it's the first step in making a tui.
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    // using stdout, allow us to enter an alternate screen where we can also use the mouse.
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture);

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        ui(f);
    })?;

    // draw to a terminal.  this will render one frame.
    /*
    for _ in 0..10 {
        terminal.draw(|f| {
            let size = f.size();
            // let block = Block::default()
            //     .title("Block")
            //     .borders(Borders::ALL);
            let items = [
                ListItem::new("This is a sentence."),
                ListItem::new("This is also a sentence, but it's on a new line."),
                ListItem::new("This is like, the third item."),
            ];
            // let block = List::new(items)
            //     .block(Block::default().title("List").borders(Borders::ALL))
            //     .style(Style::default().fg(Color::White))
            //     .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            //     .highlight_symbol(">>");
            let block = Table::new(vec![
                // Row can be created from simple strings.
                Row::new(vec!["Row11", "Row12", "Row13"]),
                // You can style the entire row.
                Row::new(vec!["Row21", "Row22", "Row23"]).style(Style::default().fg(Color::Blue)),
                // If you need more control over the styling you may need to create Cells directly
                Row::new(vec![
                    Cell::from("Row31"),
                    Cell::from("Row32").style(Style::default().fg(Color::Yellow)),
                    Cell::from(Spans::from(vec![
                        Span::raw("Row"),
                        Span::styled("33", Style::default().fg(Color::Green)),
                    ])),
                ]),
                // If a Row need to display some content over multiple lines, you just have to change
                // its height.
                Row::new(vec![
                    Cell::from("Row\n41"),
                    Cell::from("Row\n42"),
                    Cell::from("Row\n43"),
                ])
                .height(2),
            ])
            // You can set the style of the entire Table.
            .style(Style::default().fg(Color::White))
            // It has an optional header, which is simply a Row always visible at the top.
            .header(
                Row::new(vec!["Col1", "Col2", "Col3"])
                    .style(Style::default().fg(Color::Yellow))
                    // If you want some space between the header and the rest of the rows, you can always
                    // specify some margin at the bottom.
                    .bottom_margin(1),
            )
            // As any other widget, a Table can be wrapped in a Block.
            .block(Block::default().title("Table"))
            // Columns widths are constrained in the same way as Layout...
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
            ])
            // ...and they can be separated by a fixed spacing.
            .column_spacing(1)
            // If you wish to highlight a row in any specific way when it is selected...
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            // ...and potentially show a symbol in front of the selection.
            .highlight_symbol(">>");
            // a frame renders a widget given a block and a size
            f.render_widget(block, size);
        })?;
        thread::sleep(Duration::from_millis(500));
    }
    */

    // let that frame sit for 5 seconds.
    thread::sleep(Duration::from_millis(5000));

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
