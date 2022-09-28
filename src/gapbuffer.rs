use std::fmt::{Display, Formatter};

// provide a library that calculates the position of a cursor in a string.

// TODO: word motions/actions, text selection
// punctuation breakpoint.

const GAP_GROWTH: usize = 256;
const GAP_FILL: &'static str = "                                                                                                                                                                                                                                                                ";

// #[derive(Default, Debug, Clone)]
// pub struct Input {
//     gb: GapBuffer,
//     // cursor
//     // selected text
// }

#[derive(Default, Debug, Clone)]
pub struct GapBuffer {
    buffer: Vec<char>, // yes, Vec<char> is fine for our purposes.  you do NOT want to use String; it's a hassle.
    gap_start: usize,
    gap_end: usize,
}

// ____
// a___
// ab__
// a__b

// [a, b, c, -, -, -, -, -, -, d, e, f, g]
// cursor is at 3; gap_start = 3, gap_end = 8
// don't have to erase the gap parts - the "gap" is considered unused memory, so it doesn't have to
// be rewritten; it just has to be ignored
impl GapBuffer {
    pub fn new() -> Self {
        let mut gb = GapBuffer {
            buffer: vec![' '],
            gap_start: 0,
            gap_end: 0,
        };
        gb.grow();
        gb
    }
    pub fn from_string(s: String) -> Self {
        let buffer: Vec<char> = s.chars().collect();
        let mut gb = Self {
            gap_start: buffer.len(),
            gap_end: buffer.len() - 1,
            buffer,
        };
        gb.grow();
        gb
    }
    // should ONLY BE USED to check the length of the gap.  NOTHING ELSE.
    fn gap_len(&self) -> usize {
        1 + self.gap_end - self.gap_start
    }
    pub fn gap_start(&self) -> usize {
        self.gap_start
    }
    pub fn gap_end(&self) -> usize {
        self.gap_end
    }
    pub fn c_home(&mut self) {
        if self.gap_start < 100 {
            for _ in 0..self.gap_start {
                self.left();
            }
        } else {
            // ab__cd
            // __abcd
            // delete the gap, append the string to a gap
            if self.gap_end < self.buffer.len() - 1 {
                let mut v = self.buffer.split_off(self.gap_end + 1);
                let mut b: Vec<char> = self.buffer.drain(self.gap_start..).collect();
                b.append(&mut self.buffer);
                b.append(&mut v);
                self.buffer = b;
            } else {
                // drain the gap, then append the rest
                let mut gap: Vec<char> = self.buffer.drain(self.gap_start..).collect();
                gap.append(&mut self.buffer);
                self.buffer = gap;
            }
        }
    }
    pub fn c_end(&mut self) {
        // ab__cd
        // ab__cd__
        // extend with gap, split at gap_end, drain gap length, reset pointers
        todo!()
    }
    pub fn left(&mut self) {
        if self.gap_start > 0 {
            // ? Performance of this?
            // let mut v = self.buffer.chars().collect::<Vec<char>>();
            // v.swap(self.gap_start-1, self.gap_end);
            // self.buffer = v.into_iter().collect();
            self.buffer.swap(self.gap_start - 1, self.gap_end);
            self.gap_start -= 1;
            self.gap_end -= 1;
        }
    }
    pub fn right(&mut self) {
        if self.gap_end < self.buffer.len() - 1 {
            self.buffer.swap(self.gap_start, self.gap_end + 1);
            self.gap_start += 1;
            self.gap_end += 1;
        }
    }
    pub fn put(&mut self, ch: char) {
        // guaranteed to be in bounds, since the gaps are always in-bounds.
        if self.buffer.is_empty() {
            self.buffer.push(' ');
            self.grow();
        }
        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;

        if self.gap_start > self.gap_end {
            self.grow();
        }
    }
    pub fn back(&mut self) {
        if self.gap_start > 0 {
            self.gap_start -= 1;
        }
    }
    pub fn delete(&mut self) {
        if self.gap_end < self.buffer.len() - 1 {
            self.gap_end += 1;
        }
    }
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.buffer.push('_');
        self.gap_start = 0;
        self.gap_end = 0;
        self.grow();
    }
    pub fn trim_end(&mut self) {
        for i in (0..self.buffer.len() - 1).rev() {
            if self.buffer[i].is_whitespace() {
                self.buffer.pop();
            } else {
                break;
            }
        }
    }
    // * A grow_by(len) function is possible, but &[char] is a constant; making the length variable
    // * is pointless and more complex.
    // growing the gapbuffer takes at most 2 microseconds and usually around 175 ns.
    fn grow(&mut self) {
        self.buffer.reserve(GAP_GROWTH);
        let mut v = self.buffer.split_off(self.gap_start);
        self.buffer.extend_from_slice(&[' '; GAP_GROWTH]);
        self.buffer.append(&mut v);
        self.gap_end += GAP_GROWTH;
    }
    // should only be used sparingly (if at all), when the buffer is not being edited.
    fn shrink(&mut self) {
        if self.gap_len() > 4 * GAP_GROWTH {
            let mut v = self.buffer.split_off(self.gap_end);
            self.buffer
                .drain(self.gap_start..self.gap_start + (3 * GAP_GROWTH));
            self.buffer.append(&mut v);
        }
    }
    // abc_
    // abcd, s = 4, e = 3
    // abcd__, s = 4, e = 5 (3 + GAP_GROWTH)
    // ab_d, s=2, e=2
    // abcd, s=3, e=2
    // abc__d, s=3, e=4 (2 + GAP__GROWTH)
}

impl Display for GapBuffer {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut v = String::new();
        for i in 0..self.buffer.len() {
            if !(i >= self.gap_start) || !(i <= self.gap_end) {
                v.push(self.buffer[i]);
            }
        }
        write!(f, "{}", v)
    }
}
