

// provide a library that calculates the position of a cursor in a string.


// TODO: provide C-Left and C-Right by searching from cursor position left/right for a space or a
// punctuation breakpoint.

use std::collections::VecDeque;

const GAP_GROWTH: usize = 256;
const GAP_FILL: &'static str = "                                                                                                                                                                                                                                                                ";

struct Cursor {
    input: GapBuffer,
}

struct GapBuffer {
    buffer: Vec<char>,
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
        GapBuffer {
            buffer: Vec::new(),
            gap_start: 0,
            gap_end: 0,
        }
    }
    pub fn left(&mut self) {
        if self.gap_start > 0 {
            self.buffer.swap(self.gap_start-1, self.gap_end);
            self.gap_start -= 1;
            self.gap_end -= 1;
        }
    }
    pub fn right(&mut self) {
        if self.gap_end < self.buffer.len() - 1 {
            self.buffer.swap(self.gap_start, self.gap_end+1);
            self.gap_start += 1;
            self.gap_end += 1;
        }
    }
    // * A grow_by(len) function is possible, but &[char] is a constant; making the length variable
    // is pointless and more complex.
    fn grow(&mut self) {
        self.buffer.reserve(GAP_GROWTH);
        let mut v = self.buffer.split_off(self.gap_start);
        self.buffer.extend_from_slice(&[' '; GAP_GROWTH]);
        self.buffer.append(&mut v);
    }

}


// struct GapBuffer<T> {
//     offset: usize,      // tells us the start of the input
//     buf: VecDeque<T>,
// }
//
//
// impl<T> GapBuffer<T> {
//     pub fn new() -> Self {
//         Self {
//             offset: 0,
//             buf: VecDeque::new(),
//         }
//     }
//     // abcde|f
//     // |fabcde cursor: 5, offset: 1, len: 6, cursor = len-offset
//     pub fn pos(&self) -> usize {
//         self.buf.len() - self.offset
//     }
//     pub fn append(&mut self, ch: T) {
//         self.buf.push_back(ch);
//     }
//     pub fn backspace(&mut self) {
//         // if offset is already len - 1, then set offset to 0
//         if self.offset == self.buf.len() - 1 {
//             self.offset = 0;
//         }
//         self.buf.pop_back();
//     }
//     pub fn delete(&mut self) {
//         if 
//     }
//     // ab|cdef
//     // agb|cdef
//     pub fn insert(&mut self, pos: usize, ch: T) {
//
//     }
//     pub fn left(&mut self) {
//         if let Some(back) = self.buf.pop_back() {
//             self.buf.push_front(back);
//             if self.offset == self.buf.len() - 1 {
//                 self.offset = 0;
//             } else {
//                 self.offset += 1;
//             }
//         }
//     }
//     pub fn right(&mut self) {
//         if let Some(front) = self.buf.pop_front() {
//             self.buf.push_back(front);
//             if self.offset == 0 {
//                 self.offset = self.buf.len() - 1;
//             } else {
//                 self.offset -= 1;
//             }
//         }
//     }
// }
