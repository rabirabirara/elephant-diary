use textwrap;
use tui::layout::Rect;

use crate::text::gapbuffer::GapBuffer;
use crate::text::editcursor::EditCursor;


const NOTES: &'static str = r#"
An EditArea is composed of the following:
1. The text that is displayed in the area.
2. A cursor that exists somewhere in the area.
    : provides the ability to perform motions, such as start of line, prev word
3. The physical dimensions of the area.
4. A corresponding block for the area.
eventually:
highlighting and selection
history of edits (for undo/redo)
"#;

#[derive(Default, Debug, Clone)]
pub struct EditArea {
    buffer: GapBuffer,
    cursor: EditCursor,
    area: Rect,
}

impl EditArea {
    pub fn new() -> Self {
        Self::default()
    }
}
