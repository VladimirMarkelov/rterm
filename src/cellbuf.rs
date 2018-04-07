use common::*;
use std::cmp;
use std::mem;

const DEFAULT_FG: Attribute = COLOR_DEFAULT;
const DEFAULT_BG: Attribute = COLOR_DEFAULT;

/// Structure `CellRect` is a simple structure to keep information about
/// arbitrary rectange. Used internally in `CellBuf`
#[derive(Debug,Clone)]
pub struct CellRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
impl CellRect {
    pub fn new() -> Self {
        CellRect {
            left: -1,
            right: -1,
            top: -1,
            bottom: -1,
        }
    }
}

/// `CellBuf` provides a temporary buffer to generate a picture before
/// flushing all the printed data to terminal. Flushing is smart: between two
/// consecutive flush calls `CellBuf` detects damaged area and send a minimal
/// set of data to real terminal for quicker screen refresh.
#[derive(Debug)]
pub struct CellBuf {
    /// Buffer width in characters
    pub width: i32,
    /// Buffer height in characters
    pub height: i32,
    /// The current terminal state
    pub cells : Vec<Cell>,
    /// Detects if terminal must be refreshed. Dirty is set to `true` after
    /// successful execution of any putting function in case the function changes
    /// at least one `Cell` of the internal buffer.
    /// The only exception: `Clear` makes all buffer invalid not depending on
    /// how many cells were changed
    pub dirty: bool,
    /// The current 'dirty' rectange tat is sent to real terminal after calling `flush`
    pub dirty_rect: CellRect,
}

impl CellBuf {
    /// Creates a new buffer
    pub fn new(width: i32, height: i32) -> CellBuf {
        CellBuf{
            width: width,
            height: height,
            cells : vec![
                Cell{
                    ch: ' ',
                    bg: DEFAULT_BG,
                    fg: DEFAULT_FG
                };
                (height * width) as usize
            ],
            dirty: false,
            dirty_rect: CellRect::new(),
        }
    }

    /// Clears the bufffer by filling it with default colors and space character.
    /// Makes the entire buffer dirty
    pub fn clear(&mut self) {
        for c in &mut self.cells {
            c.ch = ' ';
            c.bg = COLOR_DEFAULT;
            c.fg = COLOR_DEFAULT;
        }
        self.dirty_rect = CellRect{
            left: 0,
            top: 0,
            right: self.width - 1,
            bottom: self.height - 1,
        };
        self.dirty = true;
    }

    /// Changes buffer dimensions. Used when terminal is resized.
    /// It does not clears the buffer. If new size is less than old one then
    /// the buffer content is cropped. Otherwise only new area is filled
    /// with default attributes
    pub fn resize(&mut self, width: i32, height: i32) {
        if self.width == width && self.height == height {
            return
        }

        let mut newvec = vec![
            Cell{
                ch: ' ',
                bg: COLOR_DEFAULT,
                fg: COLOR_DEFAULT
            };
            (height * width) as usize
        ];

        let minw = cmp::min(self.width, width);
        let minh = cmp::min(self.height, height);

        for y in 0..minh {
            for x in 0..minw {
                let new_idx: usize = (y * width + x) as usize;
                let old_idx: usize = (y * self.width + x) as usize;
                newvec[new_idx] = self.cells[old_idx].clone();
            }
        }

        mem::swap(&mut self.cells, &mut newvec);
        self.width = width;
        self.height = height;
        self.dirty = true;
        self.dirty_rect = CellRect{
            left: 0,
            top: 0,
            right: width - 1,
            bottom: height - 1,
        };
    }

    /// Returns the buffer `Some<Cell>` value: character and its attributes.
    /// If the coordinates are outside the current buffer it returns `None`
    pub fn get_cell(&self, x: i32, y: i32) -> Option<Cell> {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return None
        }

        let idx = (x + y * self.width) as usize;
        Some(self.cells[idx].clone())
    }

    /// Sets the new value for a buffer cell
    /// Returns false is cell coordinates are outside the current buffer
    pub fn set_cell(&mut self, x: i32, y: i32, c: Cell) -> bool {
        if let Some(old) = self.get_cell(x, y) {
            if old != c {
                let idx = (x + y * self.width) as usize;
                self.cells[idx] = c;
                self.dirty = true;

                if self.dirty_rect.left == -1 {
                    self.dirty_rect = CellRect {
                        left: x,
                        right: x,
                        top: y,
                        bottom: y,
                    };
                } else {
                    if x < self.dirty_rect.left {
                        self.dirty_rect.left = x;
                    } else if x > self.dirty_rect.right {
                        self.dirty_rect.right = x;
                    }
                    if y < self.dirty_rect.top {
                        self.dirty_rect.top = y;
                    } else if y > self.dirty_rect.bottom {
                        self.dirty_rect.bottom = y;
                    }
                }
            }
            true
        } else {
            false
        }
    }
}

