use chrono::prelude::*;
// use chrono::serde::ts_seconds;
use std::fmt::{self, Display, Formatter};

// use serde::{Deserialize, Serialize};

const TIME_FORMAT_STRING: &str = "%Y %b %d %H:%M:%S %z";

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
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "File: {} {{\n", self.file_name)?;
        for message in self.messages.iter() {
            writeln!(f, "{}", message)?;
        }
        writeln!(f, "}} -- EOF")
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

// * Display for Message only displays the most recent commit.
impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // writeln!(f, "===::")?;
        write!(f, "{}", self.most_recent().unwrap())
        // writeln!(f, "::===")
    }
}

// A commit is a string message, as well as a time.
#[derive(Debug, Clone)]
pub struct Commit {
    time: DateTime<Utc>,
    // modified: Option<DateTime<Utc>>,     you can't modify a commit; this should be per message.
    // * but I also want to store intermediate edits in the commit history, so...  it unfortunately
    // * takes extra space for such a small use case... hmm, I think I might reconsider this later
    // * on - nah, keeping the original commit is important.
    data: String,
}

// * Check the more concise way of printing the time.
impl Commit {
    pub fn new() -> Self {
        Self {
            time: Utc::now(),
            data: String::new(),
        }
    }
    pub fn data(&self) -> &str {
        &self.data
    }
    pub fn from_data(data: String) -> Self {
        Self {
            time: Utc::now(),
            data,
        }
    }
    pub fn edit_data(&mut self, data: String) {
        self.time = Utc::now();
        self.data = data;
    }
}

impl Display for Commit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {}",
            self.time.format(TIME_FORMAT_STRING),
            self.data
        )
    }
}
