use common::*;
use cellbuf::*;

/// Every type of virtual terminal must be able to write a buffer to real
/// terminal, return terminal size, set and get terminal cursor position.
/// All function returns `Result`, the second argument of `Result' is the
/// string - a error message or empty string if everything is OK
pub trait TerminalManager {
    fn write(&self, buf: &CellBuf) -> Result<(), String>;
    fn size(&self) -> Result<Point, String>;
    fn set_cursor_pos(&self, x: i16, y: i16) -> Result<(), String>;
    fn get_cursor_pos(&self) -> Result<CursorInfo, String>;
}
