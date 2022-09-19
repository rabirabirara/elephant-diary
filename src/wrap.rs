use unicode_width::UnicodeWidthStr;
use unicode_linebreak::{linebreaks, BreakOpportunity::{Mandatory, Allowed}};

pub fn greedy_wrap(text: String, width: u32) {
    let v = linebreaks(&text);
    
}
