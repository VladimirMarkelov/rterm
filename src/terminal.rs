use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

use cellbuf::*;
use common::*;
use unicode_width::*;
use intf::*;
#[cfg(windows)] use term_windows::*;

/// Virtual terminal that can print strings on real terminal, emit terminal
/// events, return the current terminal data, and move cursor
pub struct Terminal {
    buffer: CellBuf,
    fg: Attribute,
    bg: Attribute,
    #[cfg(windows)] terminal: WinTerminal,
    event_chan_rx: SyncSender<Event>,
    event_chan_tx: Receiver<Event>,
}

impl Terminal {
    /// Creates a new virtual terminal
    /// At the time of creation a real terminal's properties may be modified.
    /// The function starts event loop to monitor keyboard, mouse and real
    /// terminal events
    #[cfg(windows)]
    pub fn new() -> Terminal {
        let (rx, tx) = sync_channel::<Event>(100);
        let rx_clone = rx.clone();
        //let term = WinTerminal::new(INPUT_MOUSE | INPUT_ALT, rx_clone);
        let term = WinTerminal::new(INPUT_MOUSE | INPUT_ESC, rx_clone);
        let res = term.size();
        match res {
            Err(er) => {
                println!("Failed to initialize console: {}", er);
                panic!("Console intialization failed");
            },
            Ok(pt) => Terminal{
                            buffer: CellBuf::new(pt.x, pt.y),
                            fg: COLOR_DEFAULT,
                            bg: COLOR_DEFAULT,
                            event_chan_tx: tx,
                            event_chan_rx: rx,
                            terminal: term,
                        },
        }
    }

    /// Stops the main event loop and cleans up everything
    #[cfg(windows)]
    pub fn stop(&mut self) {
        self.terminal.tx.send(1).unwrap();
        self.terminal.wait_for_stdin();
    }

    /// Checks if there is any event in main event queue. The function does not
    /// block the execution and returns immediately. If the queue is empty then
    /// the result is `None`, otherwise the function deletes the event from
    /// queue and returns it as `Option<Event>`.
    pub fn peek_event(&mut self) -> Option<Event> {
        //self.terminal.peek_event()
        // TODO: squash the same events like Refresh/MouseMove into one
        let res = self.event_chan_tx.try_recv();
        match res {
            Ok(ev) => Some(ev),
            _ => None,
        }
    }

    /// Blocking call. If event queue contains any event then it works in the
    /// same way as peek_event does. But if the queue is empty then the function
    /// waits until a new event comes then returns the event
    pub fn get_event(&mut self) -> Option<Event> {
        //self.terminal.get_event()
        // TODO: squash the same events like Refresh/MouseMove into one
        let res = self.event_chan_tx.recv();
        match res {
            Ok(ev) => Some(ev),
            _ => None,
        }
    }

    /// Add a new event to event queue. Maybe be useful to manipulate `Terminal`,
    /// e.g., to make it refresh the real terminal immediately without waiting
    /// the next main loop cycle
    pub fn put_event(&self, ev: Event) {
        // TODO: may block, in other cases it never errs, so unwrap is OK
        self.event_chan_rx.send(ev).unwrap();
    }

    /// Writes all detected changes from internal buffer to real terminal
    pub fn flush(&mut self) {
        let res = self.terminal.write(&self.buffer);
        match res {
            Err(e) => panic!(e),
            _ => { self.buffer.dirty = false; self.buffer.dirty_rect = CellRect::new(); },
        };
    }

    /// Clears the buffer with space character and default attributes
    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    /// Retuns if the internal buffer dirty. If it retruns `true` it means that
    /// the internal buffer contains some text that is not shows on the real
    /// terminal yet(not displayed for a user yet)
    pub fn dirty(&self) -> bool {
        self.buffer.dirty
    }

    /// Returns the value of internal buffer. It is public for debug purposes
    pub fn cells(&self) -> &[Cell] {
        &self.buffer.cells[..]
    }

    /// Resize the internal buffer. Used when the real terminal is resized
    pub fn resize(&mut self, width: i32, height: i32) {
        self.buffer.resize(width, height);
    }

    /// Returns the current terminal size
    pub fn get_size(&self) -> (i32, i32) {
        (self.buffer.width, self.buffer.height)
    }

    /// Sets foreground(text) color for all following put calls
    pub fn set_foreground(&mut self, c: Attribute) {
        self.fg = c;
    }

    /// Sets background color for all folowwing put calls
    pub fn set_background(&mut self, c: Attribute) {
        self.bg = c;
    }

    /// Retuns the current foreground(text) color
    pub fn get_foreground(&self) -> Attribute {
        self.fg
    }

    /// Retuns the current background color
    pub fn get_background(&self) -> Attribute {
        self.bg
    }

    /// Sets the value of a single terminal cell.
    /// Retuns `false` if coordinates are outside terminal window
    pub fn set_cell(&mut self, x: i32, y: i32, c: Cell) -> bool {
        self.buffer.set_cell(x, y, c)
    }

    /// Retunrs the current value of a single terminal cell. Note: if the internal
    /// buffer is dirty then get_cell may return a value different from what a
    /// user sees in the terminal (in case of x and y are inside dirty region).
    /// Returns `None` if coordinates are outside terminal window or `Option<Cell>`
    pub fn get_cell(&self, x: i32, y: i32) -> Option<Cell> {
        self.buffer.get_cell(x, y)
    }

    /// Sets an UTF8 character of a terminal cell with current attributes.
    /// Retuns `false` if coordinates are outside terminal window
    pub fn put_char(&mut self, x: i32, y: i32, c: char) -> bool {
        let f = self.fg;
        let b = self.bg;
        self.set_cell(x, y, Cell{ch: c, fg: f, bg: b})
    }

    /// Sets temporarily attributes and purs a character to given coordinates
    /// Retuns `false` if coordinates are outside terminal window
    pub fn put_char_with_attrs(&mut self, x: i32, y: i32, c: char, fg: Attribute, bg: Attribute) -> bool {
        self.set_cell(x, y, Cell{ch: c, fg: fg, bg: bg})
    }

    // The only function that uses unicode width information
    // TODO: how to get CJK context?
    /// Puts a string to given coordinates using the current attributes.
    /// Retuns `false` if the entire string is outside terminal window.
    /// Retunrs `true` if at least one character of the string was printed on the screen
    pub fn put_string<S: Into<String> >(&mut self, x: i32, y: i32, s: S) -> bool {
        if y < 0 || y >= self.buffer.height || x >= self.buffer.width {
            return false
        }

        let string = s.into();
        if x + (string.chars().count() as i32) < 0 {
            return false;
        }

        let mut pos = x;
        for c in string.chars() {
            if let Some(w) = c.width() {
                if w == 0 {
                    continue;
                }
                let wi = w as i32;

                if pos >= 0 {
                    if pos + wi <= self.buffer.width {
                        self.buffer.set_cell(pos, y, Cell{ch: c, fg: self.fg, bg: self.bg});
                    } else if w == 2 && pos == self.buffer.width - 1 {
                        self.buffer.set_cell(pos, y, Cell{ch: ' ', fg: self.fg, bg: self.bg});
                    }
                }
                pos += wi;

                if pos >= self.buffer.width {
                    break;
                }
            }
        }

        true
    }

    /// Sets new attributes temporarily and puts a string to given coordinates
    /// Retuns `false` if the entire string is outside terminal window.
    /// Retunrs `true` if at least one character of the string was printed on the screen
    pub fn put_string_with_attrs<S: Into<String> >(&mut self, x: i32, y: i32, s: S, fg: Attribute, bg: Attribute) -> bool {
        let (f_save, b_save) = (self.fg, self.bg);
        self.fg = fg;
        self.bg = bg;
        let res = self.put_string(x, y, s);
        self.bg = b_save;
        self.fg = f_save;
        res
    }

    /// Puts a string from top to bottom starting from given coordinates using
    /// the current attributes.
    /// Retuns `false` if the entire string is outside terminal window.
    /// Retunrs `true` if at least one character of the string was printed on the screen
    pub fn put_string_vertical<S: Into<String> >(&mut self, x: i32, y: i32, s: S) -> bool {
        if x < 0 || x >= self.buffer.width || y >= self.buffer.height {
            return false
        }

        let string = s.into();
        if y + (string.chars().count() as i32) < 0 {
            return false;
        }

        let mut pos = y;
        for c in string.chars() {
            if pos >= 0 {
                self.buffer.set_cell(x, pos, Cell{ch: c, fg: self.fg, bg: self.bg});
            }
            pos += 1;

            if pos >= self.buffer.height {
                break;
            }
        }

        true
    }

    /// Temporarily changes the current attributes and puts a string from top to
    /// bottom starting from given coordinates.
    /// Retuns `false` if the entire string is outside terminal window.
    /// Retunrs `true` if at least one character of the string was printed on the screen
    pub fn put_string_vertical_with_attrs<S: Into<String> >(&mut self, x: i32, y: i32, s: S, fg: Attribute, bg: Attribute) -> bool {
        let (f_save, b_save) = (self.fg, self.bg);
        self.fg = fg;
        self.bg = bg;
        let res = self.put_string_vertical(x, y, s);
        self.bg = b_save;
        self.fg = f_save;
        res
    }

    /// Puts a horizontal line of character `c` starting from given coordinates.
    /// The current attributes are used.
    /// Retuns `false` if the entire string is outside terminal window.
    /// Retunrs `true` if at least one character of the string was printed on the screen
    pub fn put_horizontal_line(&mut self, x: i32, y: i32, length: i32, c: char) -> bool {
        if y < 0 || y >= self.buffer.height || x >= self.buffer.width || x + length < 0 {
            return false;
        }

        let mut xw = length;
        let mut xs = x;
        if x < 0 {
            xw += x;
            xs = 0;
        }
        if x + xw > self.buffer.width {
            xw = self.buffer.width - x
        }

        if xw <= 0 {
            return false
        }

        for xx in 0..xw {
            self.buffer.set_cell(xs + xx, y, Cell{ch: c, fg: self.fg, bg: self.bg});
        }

        true
    }

    /// Puts a horizontal line of character `c` starting from given coordinates.
    /// The attributes are changed temporarily and restored after the function finishes.
    /// Retuns `false` if the entire string is outside terminal window.
    /// Retunrs `true` if at least one character of the string was printed on the screen
    pub fn put_horizontal_line_with_attrs(&mut self, x: i32, y: i32, length: i32,
                                c: char, fg: Attribute, bg: Attribute) -> bool {
        let (f_save, b_save) = (self.fg, self.bg);
        self.fg = fg;
        self.bg = bg;
        let res = self.put_horizontal_line(x, y, length, c);
        self.bg = b_save;
        self.fg = f_save;
        res
    }

    /// Puts a vertical line of character `c` starting from given coordinates.
    /// The current attributes are used.
    /// Retuns `false` if the entire string is outside terminal window.
    /// Retunrs `true` if at least one character of the string was printed on the screen
    pub fn put_vertical_line(&mut self, x: i32, y: i32, length: i32, c: char) -> bool {
        if x < 0 || x >= self.buffer.width || y >= self.buffer.height || y + length < 0 {
            return false;
        }

        let mut yw = length;
        let mut ys = y;
        if y < 0 {
            yw += y;
            ys = 0;
        }
        if y + yw > self.buffer.height {
            yw = self.buffer.height - y
        }

        if yw <= 0 {
            return false
        }

        for yy in 0..yw {
            self.buffer.set_cell(x, ys + yy, Cell{ch: c, fg: self.fg, bg: self.bg});
        }

        true
    }

    /// Puts a vertical line of character `c` starting from given coordinates.
    /// The attributes are changed temporarily and restored after the function finishes.
    /// Retuns `false` if the entire string is outside terminal window.
    /// Retunrs `true` if at least one character of the string was printed on the screen
    pub fn put_vertical_line_with_attrs(&mut self, x: i32, y: i32, length: i32,
                                c: char, fg: Attribute, bg: Attribute) -> bool {
        let (f_save, b_save) = (self.fg, self.bg);
        self.fg = fg;
        self.bg = bg;
        let res = self.put_vertical_line(x, y, length, c);
        self.bg = b_save;
        self.fg = f_save;
        res
    }

    /// Moves terminal cursor.
    /// Returns `OK(())` if the cursor has moved, or `Err(string)` if anything
    /// failed, e.g. API call
    pub fn set_cursor_pos(&self, x: i16, y: i16) -> Result<(), String> {
        self.terminal.set_cursor_pos(x, y)
    }

    /// Returns current terminal cursor position.
    /// Returns `OK(CursorInfo)` if the cursor has moved, or `Err(string)` if anything
    /// failed, e.g. API call
    pub fn get_cursor_pos(&self) -> Result<CursorInfo, String> {
        self.terminal.get_cursor_pos()
    }
}
