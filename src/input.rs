use crate::gapbuffer::GapBuffer;

#[derive(Clone, Default)]
pub struct Input {
    pub buffer: GapBuffer,
    pub scroll: (usize, usize),
}
