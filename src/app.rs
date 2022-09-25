use crate::commit;

use crate::util::current_time_string;
use std::io;
use std::thread;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::widgets::ListState;

#[derive(PartialEq)]
pub enum EditorMode {
    // normal: can select messages and view information about them and select options on them
    // clicking with the mouse enters normal mode; the current input should be saved
    Normal,
    Writing,
    Editing,
}

// Contains the state of the application.
pub struct App {
    // what screen am I on right now?
    pub route: AppRoute,

    // the state of the start screen
    pub start: Start,

    // not really planning to have more than one file open at a time.  it's a diary for god's sake.
    pub file: commit::Diary,

    // the state of the current input
    pub input: Input,


    // the state of the current edit
    pub edit: Option<Edit>,
    // the current input mode: am I writing right now?
    pub mode: EditorMode,

    // state of the message view - do I have something selected right now?
    // I either have nothing selected (input, I guess), or I have a message selected.
    // I can press I to enter Input from normal no matter where I am.  I can press E while
    // selecting a message to edit it.
    // ListState works by telling it whe index you are selecting (Some(index)), or nothing if you
    // are not (None).
    pub select_state: ListState,
}

#[derive(PartialEq)]
pub enum AppRoute {
    Start,
    Edit,
    PreQuit,
    Quit,
}

#[derive(Default)]
pub struct Start {
    // contains a vec of paths to the most recently used files
    pub mru: Vec<String>,
}

#[derive(Clone, Default)]
pub struct Input {
    pub write_input: String,
    pub scroll: (usize, usize),
}

pub struct Edit {
    pub edit_input: String,
    pub index: usize,
}

impl Edit {
    fn from(edit_input: String, index: usize) -> Self {
        Edit { edit_input, index }
    }
    fn update(&mut self, s: String) {
        self.edit_input = s;
    }
}

impl App {
    // call startup routine
    pub fn startup(&mut self) {
        // TODO open config
        // TODO open mru list
        self.start = Start { mru: vec!["new".to_string(), "old".to_string()] };
    }
    // call normal loop routine
    pub fn run(&mut self) -> io::Result<bool> {
        if self.route == AppRoute::Quit {
            return Ok(false);
        }
        if let Ok(true) = event::poll(std::time::Duration::from_secs(0)) {
            if let Event::Key(key) = event::read()? {
                match self.route {
                    AppRoute::Start => self.run_start(key),
                    AppRoute::Edit => self.run_edit(key),
                    AppRoute::PreQuit => self.run_quit(key),      // TODO should run a quit protocol - if not saved, don't quit yet, try and confirm!
                    _ => unreachable!(),
                }
            }
        }
        Ok(true)
    }
    fn run_start(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.route = AppRoute::PreQuit,
            KeyCode::Char('n') => {
                self.route = AppRoute::Edit;
            }
            KeyCode::Enter => {
                if let Some(msg_idx) = self.selected() {
                    // TODO app should open an mru list stored somewhere, find the appropriate
                    // index, then open that file and transition into AppRoute::Edit
                    
                    // open the file and completely disregard the new file created by default.
                    // too lazy to refactor app.file into Option<Diary>...
                    if let Some(filepath) = self.start.mru.iter().nth(msg_idx) {
                        if let Ok(diary) = commit::Diary::read_from_path(filepath.into()) {
                            self.file = diary;
                        } else {
                            eprintln!("Err: failed to open {} into a valid diary.  This is probably because Rust failed to parse the file at the path.", filepath);
                        }
                    } else {
                        eprintln!("Err: ui has {} selected but that entry is not found in app.start.mru", msg_idx);
                    }

                    self.route = AppRoute::Edit;
                }
            }
            KeyCode::Up => self.select_down(self.start.mru.len()),
            KeyCode::Down => self.select_up(self.start.mru.len()),
            KeyCode::Esc => self.unselect(),
            _ => (),
        }
        // eprintln!("{}", self.select_state.selected().unwrap_or(10));
    }
    fn run_edit(&mut self, key: KeyEvent) {
        match self.mode {
            EditorMode::Normal => {
                match key.code {
                    KeyCode::Char('i') => {
                        self.mode = EditorMode::Writing;
                        self.unselect(); // leave message selection
                    }
                    KeyCode::Char('e') => {
                        if let Some(msg_idx) = self.selected() {
                            self.mode = EditorMode::Editing;

                            // so, msg_idx is usually the complement of the actual index in the
                            // file. e.g. msg_idx = 0? then the most recent message is chosen.
                            let msg_count = self.file.messages.len();
                            let file_idx = msg_count - msg_idx - 1;
                            // could also do iter().rev().nth(msg_idx)...

                            // take the message at file_idx, get its most recent commit,
                            let mrc = self
                                .file
                                .messages
                                .iter()
                                .nth(file_idx)
                                .expect(
                                    format!(
                                        "shoulda had a message at mi {}, fi {}",
                                        msg_idx, file_idx
                                    )
                                    .as_ref(),
                                )
                                .most_recent()
                                .expect("a message should have a commit... it's not possible to be without one")
                                .data();

                            // set the editing input bar to that and show it
                            self.edit = Some(Edit::from(mrc.to_string(), file_idx));
                        } else {
                            // he didn't select a message, so just put him in Write mode,
                            // on a new message.
                            self.mode = EditorMode::Writing;
                        }
                    }
                    KeyCode::Char('w') => {
                        // TODO: write file; give file name to write to; if no file name provided
                        // then prompt for a valid file name; else, give file's current file
                        // name.
                        // for now just save to file called "new"
                        self.file.write_to_path("new".to_string());
                        // TODO: write to mru when file has a filename and is being opened, or when
                        // a file is new and is being written.
                    }
                    KeyCode::Char('q') => self.route = AppRoute::PreQuit,
                    KeyCode::Up => self.select_up(self.file.messages.len()),
                    KeyCode::Down => self.select_down(self.file.messages.len()),
                    KeyCode::Esc => self.unselect(),
                    _ => {}
                }
            }
            // TODO: Implement Shift+Tab to go to Normal as well.
            EditorMode::Writing => {
                match key.code {
                    KeyCode::Enter => {
                        let input = self.input.write_input.trim_end();
                        self.file.push_string(input.to_string());
                        self.input.write_input.clear();
                    }
                    KeyCode::Char(c) => {
                        self.input.write_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.input.write_input.pop();
                    }
                    KeyCode::Esc | KeyCode::BackTab => {
                        self.mode = EditorMode::Normal;
                    }
                    // TODO: implement cursor as well as inserting characters anywhere in the
                    // input bar, not just at the end.
                    _ => {}
                }
            }
            EditorMode::Editing => {
                if let Some(edit) = &mut self.edit {
                    match key.code {
                        KeyCode::Enter => {
                            let input = edit.edit_input.trim_end();

                            // find the Message at edit.index and commit the current
                            // edit.input to it.
                            self.file
                                .messages
                                .iter_mut()
                                .nth(edit.index)
                                .expect(
                                    format!("should have message at file index {}", edit.index)
                                        .as_ref(),
                                )
                                .push_commit(commit::Commit::from_data(input.to_string()));

                            // now return to normal mode and unselect.
                            self.mode = EditorMode::Normal;
                            self.edit = None;
                            self.unselect();
                        }
                        KeyCode::Char(c) => {
                            edit.edit_input.push(c);
                        }
                        KeyCode::Backspace => {
                            edit.edit_input.pop();
                        }
                        KeyCode::Esc | KeyCode::BackTab => {
                            // clears the current edit state.  sorry, if you want to scroll
                            // up, need mouse.
                            // TODO: find a way to let people scroll while preserving the
                            // edit state. i.e. without clearing self.edit outright.
                            self.mode = EditorMode::Normal;
                            self.edit = None;
                        }
                        _ => {}
                    }
                } else {
                    // if we somehow are in editing mode without any information about what
                    // to edit, just quit back to write mode.
                    self.mode = EditorMode::Writing;
                    self.unselect();
                }
            }
        }
    }
    fn run_quit(&mut self, key: KeyEvent) {
        match key.code {
            _ => self.route = AppRoute::Quit,
        }
        // unimplemented!()    // * this quit screen is part of the app; you can have a closing protocol outside in run_app in main.
    }
    fn unselect(&mut self) {
        self.select_state.select(None);
    }
    fn select_up(&mut self, len: usize) {
        let idx = match self.select_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    // Going back to the input bar.
                    None
                } else {
                    Some(i + 1)
                }
            }
            None => {
                if len == 0 {
                    None
                } else {
                    Some(0)
                }
            }
        };
        self.select_state.select(idx);
    }
    fn select_down(&mut self, len: usize) {
        let idx = match self.select_state.selected() {
            Some(i) => {
                if i == 0 {
                    None
                } else {
                    Some(i - 1)
                }
            }
            None => {
                if len == 0 {
                    None
                } else {
                    Some(len - 1)
                }
            }
        };
        self.select_state.select(idx);
    }
    fn selected(&self) -> Option<usize> {
        self.select_state.selected()
    }
}

impl Default for App {
    fn default() -> App {
        // locate mru.  if mru not found, 
        let app = App {
            route: AppRoute::Start,
            start: Start::default(),
            file: commit::Diary::new(),
            input: Input::default(),
            edit: None,
            mode: EditorMode::Normal,
            select_state: ListState::default(),
        };

        app
    }
}
