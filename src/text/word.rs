
// from tui-textarea.rs
#[derive(PartialEq)]
enum CharKind {
    Whitespace,
    Punctuation,
    Other,
}

trait HasCharKind {
    fn kind(&self) -> CharKind;
}

impl HasCharKind for char {
    fn kind(&self) -> CharKind {
        if self.is_whitespace() {
            CharKind::Whitespace
        } else if self.is_ascii_punctuation() {
            CharKind::Punctuation
        } else {
            CharKind::Other
        }
    }
}

// given a line and a position of the cursor in it, find the next word.
pub fn next_word(line: &str, current: usize) -> Option<usize> {
    // a_bcd cursor at 1
    // skip 1 element
    let mut line_itr = line.chars().enumerate().skip(current);
    let mut prev_ch_kind = line_itr.next()?.1.kind();
    for (pos, ch) in line_itr {
        // if cursor is on whitespace and next thing is whitespace, keep moving
        let ch_kind = ch.kind();
        if ch_kind != CharKind::Other && prev_ch_kind == ch_kind {
            return Some(pos);
        }
        prev_ch_kind = ch_kind;
    }
    None
}

// TODO check if this works.
// pub fn prev_word(line: &str, current: usize) -> Option<usize> {
//     // a_bcd cursor at 1
//     // skip 1 element
//     let mut line_itr = line.chars().rev().enumerate().skip(line.len() - current);
//     let mut prev_ch_kind = line_itr.next()?.1.kind();
//     for (pos, ch) in line_itr {
//         // if cursor is on whitespace and next thing is whitespace, keep moving
//         let ch_kind = ch.kind();
//         if ch_kind != CharKind::Other && prev_ch_kind == ch_kind {
//             return Some(pos);
//         }
//         prev_ch_kind = ch_kind;
//     }
//     None
// }
