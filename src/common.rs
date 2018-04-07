pub type InputMode = i32;
pub type OutputMode = i32;
pub type EventType = u8;
pub type Modifier = u8;
pub type Key = u16;
pub type Attribute = u16;

/// Terminal cursor information
#[derive(Debug)]
pub struct CursorInfo {
    pub visible: bool,
    pub x: i16,
    pub y: i16,
}

/// A structure to keep a pair of coordinates
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Events emitted by `Terminal`
#[derive(Debug,Clone)]
pub enum Event {
    /// No event
    None,
    /// Terminal was resized - value is new terminal size
    Resize(i32, i32),
    /// Mouse event: button pressed, released, wheel scrolled, mouse moved
    /// event coordinates, button pressed, modifier used as mouse move indicator(MOD_MOTION)
    Mouse(i32, i32, Key, Modifier),
    /// Key pressed
    /// Virtual code, UTF8 character if available, key modifier: Alt, Shift, Control
    Key(Key, char, Modifier),
}

/// Internal terminal cell representation
#[derive(Debug,Clone,PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Attribute,
    pub bg: Attribute
}

iota! {
    pub const KEY_F1: u16 = 0xFFFF - iota;
        | KEY_F2
        | KEY_F3
        | KEY_F4
        | KEY_F5
        | KEY_F6
        | KEY_F7
        | KEY_F8
        | KEY_F9
        | KEY_F10
        | KEY_F11
        | KEY_F12
        | KEY_INSERT
        | KEY_DELETE
        | KEY_HOME
        | KEY_END
        | KEY_PGUP
        | KEY_PGDN
        | KEY_ARROW_UP
        | KEY_ARROW_DOWN
        | KEY_ARROW_LEFT
        | KEY_ARROW_RIGHT
        | KEY_MIN
        | MOUSE_LEFT
        | MOUSE_MIDDLE
        | MOUSE_RIGHT
        | MOUSE_RELEASE
        | MOUSE_WHEEL_UP
        | MOUSE_WHEEL_DOWN
}

pub const KEY_CTRL_TILDE      :Key = 0x00;
pub const KEY_CTRL_2          :Key = 0x00;
pub const KEY_CTRL_SPACE      :Key = 0x00;
pub const KEY_CTRL_A          :Key = 0x01;
pub const KEY_CTRL_B          :Key = 0x02;
pub const KEY_CTRL_C          :Key = 0x03;
pub const KEY_CTRL_D          :Key = 0x04;
pub const KEY_CTRL_E          :Key = 0x05;
pub const KEY_CTRL_F          :Key = 0x06;
pub const KEY_CTRL_G          :Key = 0x07;
pub const KEY_BACKSPACE      :Key = 0x08;
pub const KEY_CTRL_H          :Key = 0x08;
pub const KEY_TAB            :Key = 0x09;
pub const KEY_CTRL_I          :Key = 0x09;
pub const KEY_CTRL_J          :Key = 0x0A;
pub const KEY_CTRL_K          :Key = 0x0B;
pub const KEY_CTRL_L          :Key = 0x0C;
pub const KEY_ENTER          :Key = 0x0D;
pub const KEY_CTRL_M          :Key = 0x0D;
pub const KEY_CTRL_N          :Key = 0x0E;
pub const KEY_CTRL_O          :Key = 0x0F;
pub const KEY_CTRL_P          :Key = 0x10;
pub const KEY_CTRL_Q          :Key = 0x11;
pub const KEY_CTRL_R          :Key = 0x12;
pub const KEY_CTRL_S          :Key = 0x13;
pub const KEY_CTRL_T          :Key = 0x14;
pub const KEY_CTRL_U          :Key = 0x15;
pub const KEY_CTRL_V          :Key = 0x16;
pub const KEY_CTRL_W          :Key = 0x17;
pub const KEY_CTRL_X          :Key = 0x18;
pub const KEY_CTRL_Y          :Key = 0x19;
pub const KEY_CTRL_Z          :Key = 0x1A;
pub const KEY_ESC            :Key = 0x1B;
pub const KEY_CTRL_LSQ_BRACKET :Key = 0x1B;
pub const KEY_CTRL_3          :Key = 0x1B;
pub const KEY_CTRL_4          :Key = 0x1C;
pub const KEY_CTRL_BACKSLASH  :Key = 0x1C;
pub const KEY_CTRL_5          :Key = 0x1D;
pub const KEY_CTRL_RSQ_BRACKET :Key = 0x1D;
pub const KEY_CTRL_6          :Key = 0x1E;
pub const KEY_CTRL_7          :Key = 0x1F;
pub const KEY_CTRL_SLASH      :Key = 0x1F;
pub const KEY_CTRL_UNDERSCORE :Key = 0x1F;
pub const KEY_SPACE          :Key = 0x20;
pub const KEY_BACKSPACE_2     :Key = 0x7F;
pub const KEY_CTRL_8          :Key = 0x7F;

iota! {
    pub const MOD_ALT: Modifier = 1 << iota;
        | MOD_MOTION
}

iota! {
    pub const COLOR_DEFAULT: Attribute = iota;
        | COLOR_BLACK
        | COLOR_RED
        | COLOR_GREEN
        | COLOR_YELLOW
        | COLOR_BLUE
        | COLOR_MAGENTA
        | COLOR_CYAN
        | COLOR_WHITE
}

iota! {
    pub const INPUT_ESC: InputMode = 1 << iota;
        | INPUT_ALT
        | INPUT_MOUSE
}
pub const INPUT_CURRENT: InputMode = 0;

iota! {
    pub const OUTPUT_CURRENT: OutputMode = iota;
        | OUTPUT_NORMAL
        | OUTPUT_256
        | OUTPUT_216
        | OUTPUT_GRAYSCALE
}

iota! {
    pub const EVENT_KEY: EventType = iota;
        | EVENT_RESIZE
        | EVENT_MOUSE
        | EVENT_ERROR
        | EVENT_INTERRUPT
        | EVENT_RAW
        | EVENT_NONE
}

iota! {
    pub const ATTR_BOLD: Attribute = 1 << (iota + 9);
        | ATTR_UNDERLINE
        | ATTR_REVERSE
}


pub const CURSOR_HIDDEN: i32 = -1;
