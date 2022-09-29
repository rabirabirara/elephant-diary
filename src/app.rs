use crate::commit;
use crate::text::gapbuffer::GapBuffer;
use crate::config::DiaryConfig;
use crate::util::current_time_string;
use std::fs::canonicalize;
use std::io;
use std::path::PathBuf;
use std::thread;

use chrono::prelude::{DateTime, Local, Utc};
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
    Saving,
}

// Contains the state of the application.
pub struct App {
    // the configuration of the application.
    pub config: DiaryConfig,
    // what screen am I on right now?
    pub route: AppRoute,

    // not really planning to have more than one file open at a time.  it's a diary for god's sake.
    pub file: commit::Diary,

    // the state of the current input
    pub input: GapBuffer,

    // the state of the current edit
    pub edit: Option<Edit>,

    // the current input mode: am I writing right now?
    pub mode: EditorMode,
    // the status bar message
    pub status_msg: String,

    // temporary input, used for anything where you need a general input for a single screen.
    pub temp_input: String,

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
    pub fn startup(&mut self) {}
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
                    AppRoute::PreQuit => self.run_prequit(key), // TODO should run a quit protocol - if not saved, don't quit yet, try and confirm!
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
                    if let Some(filepath) = self.config.mru.get(msg_idx) {
                        // TODO: filepath should be absolute.
                        if let Ok(diary) = commit::Diary::read_from_path(filepath) {
                            self.file = diary;
                            self.config.update_mru_with(filepath.into());
                        } else {
                            eprintln!("Err: failed to open {} into a valid diary.  This is probably because Rust failed to parse the file at the path.", filepath);
                        }
                    } else {
                        eprintln!(
                            "Err: ui has {} selected but that entry is not found in app.start.mru",
                            msg_idx
                        );
                    }

                    self.route = AppRoute::Edit;
                }
            }
            KeyCode::Up => self.select_down(self.config.mru.len()),
            KeyCode::Down => self.select_up(self.config.mru.len()),
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
                        self.change_mode(EditorMode::Writing);
                        self.unselect(); // leave message selection
                    }
                    KeyCode::Char('e') => {
                        if let Some(msg_idx) = self.selected() {
                            self.change_mode(EditorMode::Editing);

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
                            self.change_mode(EditorMode::Writing);
                        }
                    }
                    KeyCode::Char('W') => {
                        if !self.file.messages.is_empty() {
                            self.change_mode(EditorMode::Saving);
                        } else {
                            self.set_status(String::from("Nothing to write!..."));
                        }
                    }
                    KeyCode::Char('w') => {
                        // TODO: write file; give file name to write to; if no file name provided
                        // then prompt for a valid file name; else, give file's current file
                        // TODO make a whole saving screen, actually, since save-as will be useful
                        // later on.

                        // for now, if file is new, just save to file named after date of oldest message
                        if self.file.messages.is_empty() {
                            self.set_status(String::from("Nothing to write!..."));
                        } else {
                            if self.file.name.is_empty() {
                                self.change_mode(EditorMode::Saving);
                            } else {
                                // write to filename
                                self.file.write_to_path(self.file.name.clone());
                            }
                        }

                        // write to mru when file has a filename and is being opened, or when
                        // a file is new and is being written.
                    }
                    KeyCode::Char('q') => self.route = AppRoute::PreQuit,
                    KeyCode::Up => self.select_up(self.file.messages.len()),
                    KeyCode::Down => self.select_down(self.file.messages.len()),
                    KeyCode::Esc => self.unselect(),
                    _ => {}
                }
            }
            EditorMode::Writing => {
                match key.code {
                    KeyCode::Enter => {
                        self.input.trim_end();
                        self.file.push_string(self.input.to_string());
                        self.input.clear();
                    }
                    KeyCode::Char(c) => {
                        self.input.put(c);
                    }
                    KeyCode::Left => self.input.left(),
                    KeyCode::Right => self.input.right(),
                    KeyCode::Backspace => self.input.back(),
                    KeyCode::Delete => self.input.delete(),
                    KeyCode::Esc | KeyCode::BackTab => {
                        self.change_mode(EditorMode::Normal);
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
                            self.change_mode(EditorMode::Normal);
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
                            self.change_mode(EditorMode::Normal);
                            self.edit = None;
                        }
                        _ => {}
                    }
                } else {
                    // if we somehow are in editing mode without any information about what
                    // to edit, just quit back to write mode.
                    self.change_mode(EditorMode::Writing);
                    self.unselect();
                }
            }
            EditorMode::Saving => {
                match key.code {
                    KeyCode::Char(c) => self.temp_input.push(c),
                    KeyCode::Backspace => {
                        self.temp_input.pop();
                    }
                    KeyCode::Esc => self.change_mode(EditorMode::Normal),
                    KeyCode::Enter => {
                        if self.file.name.is_empty() {
                            if !self.file.messages.is_empty() {
                                let name: String = self.temp_input.drain(..).collect();
                                self.file.write_to_path(name.clone());
                                self.file.name = name.clone();

                                // save to mru
                                // TODO: extract the filepath from this filename by expanding '.'
                                // directory and then appending filename
                                let filename = PathBuf::from(name.clone());
                                self.config.update_mru_with(
                                    canonicalize(&filename)
                                        .expect("should've canonicalized the full path")
                                        .to_str()
                                        .expect("Why wouldn't you be able to convert this to a string?...")
                                        .to_string(),
                                );
                            } else {
                                // if file is empty... don't write anything.
                                self.set_status(String::from("Nothing to write!..."));
                                // unreachable!();
                            }
                        } else {
                            // write to filename
                            self.file.write_to_path(self.file.name.clone());
                        }
                        self.change_mode(EditorMode::Normal);
                    }
                    _ => (),
                }
            }
        }
    }
    fn run_prequit(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('n') => self.route = AppRoute::Edit,
            KeyCode::Char('y') => self.route = AppRoute::Quit,
            KeyCode::Enter => self.route = AppRoute::Quit,
            _ => (),
        }
    }
    fn change_mode(&mut self, mode: EditorMode) {
        self.status_msg = String::new();
        self.mode = mode;
    }
    fn set_status(&mut self, msg: String) {
        self.status_msg = msg;
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
        // TODO locate mru.  if mru not found, then default.
        let config: DiaryConfig = if let Ok(config) = confy::load(crate::util::APP_NAME, None) {
            config
        } else {
            DiaryConfig::default()
        };
        let app = App {
            config,
            route: AppRoute::Start,
            file: commit::Diary::new(),
            input: GapBuffer::default(),
            edit: None,
            mode: EditorMode::Normal,
            status_msg: String::default(),
            temp_input: String::default(),
            select_state: ListState::default(),
        };

        app
    }
}
