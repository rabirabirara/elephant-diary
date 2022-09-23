use chrono::prelude::*;
// use chrono::serde::ts_seconds;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::str::FromStr;
// use serde::{Deserialize, Serialize};

const TIME_FORMAT_STRING: &str = "%Y %b %d %H:%M:%S %z";

#[derive(Debug, Clone)]
pub struct FileParseError;

// A file is a list of messages.
#[derive(Debug, Clone)]
pub struct File {
    file_name: String,
    pub messages: Vec<Message>,
}

impl File {
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            messages: Vec::new(),
        }
    }
    pub fn name(&self) -> &str {
        &self.file_name
    }
    pub fn set_name(&mut self, file_name: String) {
        self.file_name = file_name;
    }
    pub fn push_string(&mut self, s: String) {
        self.messages
            .push(Message::from_commit(Commit::from_data(s)));
    }
    pub fn push_msg(&mut self, msg: Message) {
        self.messages.push(msg);
    }
    pub fn write_to_path(&self, path: String) {
        fs::write(path, self.to_string()).expect("unable to write to file!");
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}\n", self.file_name)?;
        for message in self.messages.iter() {
            writeln!(f, "\n{}", message)?;
        }
        Ok(())
    }
}

impl FromStr for File {
    type Err = FileParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut file = File::new();

        Ok(File::new())
    }
}

// A message is a stack of commits, where the most recent commit is at the top.
#[derive(Debug, Clone)]
pub struct Message {
    commits: Vec<Commit>,
}

impl Message {
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
        }
    }
    pub fn from_commit(c: Commit) -> Self {
        Self { commits: vec![c] }
    }
    pub fn empty(&self) -> bool {
        self.commits.is_empty()
    }
    pub fn created(&self) -> Option<DateTime<Local>> {
        if let Some(first) = self.commits.first() {
            Some(first.time)
        } else {
            None
        }
    }
    pub fn modified(&self) -> Option<DateTime<Local>> {
        if let Some(last) = self.commits.last() {
            Some(last.time)
        } else {
            None
        }
    }
    pub fn most_recent(&self) -> Option<&Commit> {
        self.commits.last()
    }
    pub fn most_recent_mut(&mut self) -> Option<&mut Commit> {
        self.commits.last_mut()
    }
    pub fn push_commit(&mut self, commit: Commit) {
        self.commits.push(commit);
    }
}

// Message::to_string() simply writes all commits line by line.  Commits cannot have empty trailing
// spaces.
impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for c in self.commits.iter() {
            write!(f, "{}\n", c)?;
        }
        Ok(())
    }
}

// A commit is a string message, as well as a time.
// You CANNOT edit a commit, so each commit merely has a time it was created.
#[derive(Debug, Clone)]
pub struct Commit {
    time: DateTime<Local>,
    data: String,
}

// * Check the more concise way of printing the time.
impl Commit {
    pub fn new() -> Self {
        Self {
            time: Local::now(),
            data: String::new(),
        }
    }
    pub fn data(&self) -> &str {
        &self.data
    }
    pub fn from_data(data: String) -> Self {
        Self {
            time: Local::now(),
            data,
        }
    }
}

impl Display for Commit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {}",
            self.time.format(TIME_FORMAT_STRING),
            // if let Some(mtime) = self.modified { format!("{}", mtime.format(TIME_FORMAT_STRING)) } else { "".to_string() },
            self.data
        )
    }
}
