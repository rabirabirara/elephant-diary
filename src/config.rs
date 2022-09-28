use std::collections::VecDeque;
use serde_derive::{Serialize, Deserialize};
use confy;

#[derive(Default, Serialize, Deserialize)]
pub struct DiaryConfig {
    pub mru: VecDeque<String>,
}

impl DiaryConfig {
    pub fn update_mru_with(&mut self, filepath: String) {
        if self.mru.len() > 30 {
            self.mru.pop_front();
        }
        if !self.mru.contains(&filepath) {
            self.mru.push_back(filepath);
        }
        confy::store(crate::util::APP_NAME, None, self).expect("failed to write config");
    }
}
