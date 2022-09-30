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

use crate::app::app::*;
use crate::ui::elphy;
use crate::util::*;

const HELPEPHANT: &'static str = r#"
   Here to                   _.-----.._____,-~~~~-._...__
                          ,-'            /         `....
   help!                ,'             ,'      .  .  \::.
                      ,'        . ''    :     . \  `./::..
It's Elphy,         ,'    ..   .     .      .  . : ;':::.
                   /     :go. :       . :    \ : ;'.::.
 the elegant,      |     ' .o8)     .  :|    : ,'. .
                  /     :   ~:'  . '   :/  . :/. .
  elephant.      /       ,  '          |   : /. .
                /       ,              |   ./.
 I talk,        L._    .       ,' .:.  /  ,'.
               /-.     :.--._,-'~~~~~~| ,'|:
  elegantly.  ,--.    /   .:/         |/::| `.
              |-.    /   .;'      .-__)::/    \
 ...._____...-|-.  ,'  .;'      .' '.'|;'      |
   ~--..._____\-_-'  .:'      .'   /  '
    ___....--~~   _.-' `.___.'   ./
      ~~------+~~_. .    ~~    .,'
                  ~:_.' . . ._:'   _ Seal _
 Have some           ~~-+-+~~
  commands.
    And press, any key 
        to move on.
"#;
// 58x26
const MIN_ELPHY_WIDTH: u16 = 35;

// TODO add the elephant somewhere.
pub fn help_screen<B: Backend>(f: &mut Frame<B>, _app: &mut App) {
    let rect = f.size();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Length(rect.width - MIN_ELPHY_WIDTH),
                Constraint::Min(MIN_ELPHY_WIDTH),
            ]
            .as_ref(),
        )
        .split(rect);

    let elphy = Paragraph::new(HELPEPHANT);
    f.render_widget(elphy, chunks[1]);

    // now, just a list of commands. first the bold of the command buttons themselves:
    let command_style = Style::default().add_modifier(Modifier::BOLD);
    let descript_style = Style::default()
        .add_modifier(Modifier::ITALIC)
        .add_modifier(Modifier::DIM);

    let command_table = Table::new(vec![
        Row::new(vec!["NORMAL"]),
        Row::new(vec![
            Cell::from(Spans::from(vec![
                Span::styled("q", command_style),
                Span::styled(" quit", descript_style),
            ])),
            Cell::from(Spans::from(vec![
                Span::styled("i", command_style),
                Span::styled(" insert", descript_style),
            ])),
        ]),
        Row::new(vec![
            Cell::from(Spans::from(vec![
                Span::styled("w", command_style),
                Span::styled(" save", descript_style),
            ])),
            Cell::from(Spans::from(vec![
                Span::styled("W", command_style),
                Span::styled(" save as", descript_style),
            ])),
        ]),
        Row::new(vec![
            Cell::from(Spans::from(vec![
                Span::styled("e", command_style),
                Span::styled(" edit selected", descript_style),
            ])),
            // Cell::from(Spans::from(vec![
            //     Span::styled("W", command_style),
            //     Span::styled(" save as", descript_style),
            // ])),
        ]).height(2),
        Row::new(vec!["WRITE/EDIT"]),
        Row::new(vec![
            Cell::from(Spans::from(vec![
                Span::styled("Esc", command_style),
                Span::styled(" normal mode", descript_style),
            ])),
            Cell::from(Spans::from(vec![
                Span::styled("S-Tab", command_style),
                Span::styled(" normal mode", descript_style),
            ])),
        ]),
    ])
    .header(Row::new(vec![""]))
    .block(
        Block::default()
            .title(" COMMAND LIST: ")
            .borders(Borders::NONE),
    )
    // .style(Style::default().fg(Color::Yellow))
    .widths(&[Constraint::Percentage(50), Constraint::Length(50)]);

    f.render_widget(command_table, chunks[0]);
}
