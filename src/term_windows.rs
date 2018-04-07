extern crate kernel32;
extern crate winapi;

use std::thread;
use std::sync::mpsc::{channel, Sender, SyncSender};

use self::winapi::{HANDLE, WCHAR};
use self::winapi::{FALSE, DWORD, SHORT, BOOL};
use self::winapi::{COORD, SMALL_RECT, CHAR_INFO, CONSOLE_SCREEN_BUFFER_INFO};
use self::winapi::{FOREGROUND_RED, FOREGROUND_GREEN, FOREGROUND_BLUE};
use self::winapi::{BACKGROUND_RED, BACKGROUND_GREEN, BACKGROUND_BLUE};
use self::winapi::{FOREGROUND_INTENSITY, BACKGROUND_INTENSITY};

use common::*;
use cellbuf::*;
use intf::*;

const MOUSE_LMB: DWORD = 0x1;
const MOUSE_RMB: DWORD = 0x2;
const MOUSE_MMB: DWORD = 0x4 | 0x8 | 0x10;

/// Implemetation of Windows terminal
#[derive(Debug)]
pub struct WinTerminal {
    /// Channel to send keyboard, mouse and terminal events
    pub tx: Sender<i32>,
    stdin_worker: Option<thread::JoinHandle<()>>,
    input_mode: InputMode,
    event_chan: SyncSender<Event>,
}

/// Internal state of keyboard and mouse processor
struct ThreadState {
    last_state: DWORD,
    last_x: SHORT,
    last_y: SHORT,
    last_button: Key,
    last_button_pressed: Key,
    alt_mode_esc: bool,
    input_mode: InputMode,
    repeat_count: u16,
}

lazy_static! {
    static ref COLOR_TABLE_BG: Vec<DWORD> = {
        vec![
            0, // Default = black
            0,
            BACKGROUND_RED,
            BACKGROUND_GREEN,
            BACKGROUND_RED | BACKGROUND_GREEN,
            BACKGROUND_BLUE,
            BACKGROUND_RED | BACKGROUND_BLUE,
            BACKGROUND_BLUE | BACKGROUND_GREEN,
            BACKGROUND_BLUE | BACKGROUND_GREEN | BACKGROUND_RED,
        ]
    };
    static ref COLOR_TABLE_FG: Vec<DWORD> = {
        vec![
            FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_RED, // Default = White
            0,
            FOREGROUND_RED,
            FOREGROUND_GREEN,
            FOREGROUND_RED | FOREGROUND_GREEN,
            FOREGROUND_BLUE,
            FOREGROUND_RED | FOREGROUND_BLUE,
            FOREGROUND_BLUE | FOREGROUND_GREEN,
            FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_RED,
        ]
    };
}

fn get_ct(table: &Vec<DWORD>, idx: u16) -> u16 {
    let mut idx: usize = (idx as usize) & 0x0F;
    if idx >= table.len() {
        idx = table.len() - 1
    }
    return table[idx] as u16
}

fn cell_to_char_info(c: &Cell) -> (u16, Vec<WCHAR>) {
    let mut attr = get_ct(&*COLOR_TABLE_FG, c.fg) | get_ct(&*COLOR_TABLE_BG, c.bg);

    if c.fg & ATTR_REVERSE | c.bg & ATTR_REVERSE != 0 {
        attr = (attr&0xF0)>>4 | (attr & 0x0F)<<4;
    }
    if c.fg & ATTR_BOLD != 0 {
        attr |= FOREGROUND_INTENSITY as u16
    }
    if c.bg & ATTR_BOLD != 0 {
        attr |= BACKGROUND_INTENSITY as u16
    }

    let mut v: Vec<WCHAR> = vec![];
    let c = c.ch;
    let mut arr: [u16; 2] = [0; 2];

    let res = c.encode_utf16(&mut arr);
    if res.len() == 1 {
        v.push(res[0]);
    } else {
        v.push(res[0]);
        v.push(res[1]);
    }

    (attr, v)
}

impl WinTerminal {
    pub fn stdout_handle() -> winapi::HANDLE {
        let hout: HANDLE;

        unsafe {
            hout = kernel32::GetStdHandle(self::winapi::STD_OUTPUT_HANDLE);
        }
        if hout == winapi::INVALID_HANDLE_VALUE {
            panic!("NO STDOUT");
        }

        hout
    }

    pub fn wait_for_stdin(&mut self) {
        if self.stdin_worker.is_none() {
            return;
        }

        self.stdin_worker.take().unwrap().join().expect("Could not join thread");
    }

    fn input_record_to_event(irec: winapi::INPUT_RECORD, state: &mut ThreadState) -> Event {
        state.repeat_count = 1;
        match irec.EventType {
            winapi::WINDOW_BUFFER_SIZE_EVENT => {
                let ws: &winapi::WINDOW_BUFFER_SIZE_RECORD;
                unsafe { ws = irec.WindowBufferSizeEvent(); }
                Event::Resize(
                    ws.dwSize.X as i32,
                    ws.dwSize.Y as i32,
                )
            },
            winapi::MOUSE_EVENT => {
                let ms: &winapi::MOUSE_EVENT_RECORD;
                unsafe { ms = irec.MouseEvent(); }
                match ms.dwEventFlags {
                    // single or double click
                    0 | 2 => {
                        let cs = ms.dwButtonState;
                        if state.last_state & MOUSE_LMB == 0 &&
                            cs & MOUSE_LMB != 0 {
                            state.last_button = MOUSE_LEFT;
                            state.last_button_pressed = MOUSE_LEFT;
                        } else if state.last_state & MOUSE_LMB != 0 &&
                            cs & MOUSE_LMB == 0 {
                            state.last_button = MOUSE_RELEASE;
                        } else if state.last_state & MOUSE_RMB == 0 &&
                            cs & MOUSE_RMB != 0 {
                            state.last_button = MOUSE_RIGHT;
                            state.last_button_pressed = MOUSE_RIGHT;
                        } else if state.last_state & MOUSE_RMB != 0 &&
                            cs & MOUSE_RMB == 0 {
                            state.last_button = MOUSE_RELEASE;
                        } else if state.last_state & MOUSE_MMB == 0 &&
                            cs & MOUSE_MMB != 0 {
                            state.last_button = MOUSE_MIDDLE;
                            state.last_button_pressed = MOUSE_MIDDLE;
                        } else if state.last_state & MOUSE_MMB != 0 &&
                            cs & MOUSE_MMB == 0 {
                            state.last_button = MOUSE_RELEASE;
                        } else {
                            state.last_state = cs;
                            return Event::None;
                        }

                        state.last_state = cs;
                        Event::Mouse(
                            ms.dwMousePosition.X as i32,
                            ms.dwMousePosition.Y as i32,
                            state.last_button,
                            0,
                        )
                    },
                    // mouse motion
                    1 => {
                        let x = ms.dwMousePosition.X;
                        let y = ms.dwMousePosition.Y;
                        if state.last_state != 0
                           && (state.last_x != x || state.last_y != y) {
                            state.last_x = x;
                            state.last_y = y;
                            Event::Mouse(x as i32, y as i32, state.last_button, MOD_MOTION)
                        } else {
                            Event::None
                        }
                    },
                    // mouse wheel
                    4 => {
                        let n = (ms.dwButtonState >> 16) as i16;
                        let k = if n > 0 { MOUSE_WHEEL_UP } else { MOUSE_WHEEL_DOWN };
                        state.last_x = ms.dwMousePosition.X;
                        state.last_y = ms.dwMousePosition.Y;

                        Event::Mouse(state.last_x as i32, state.last_y as i32, k, 0)
                    },
                    _ => Event::None,
                }
            },
            winapi::KEY_EVENT => {
                let ks: &winapi::KEY_EVENT_RECORD;
                unsafe { ks = irec.KeyEvent(); }
                state.repeat_count = ks.wRepeatCount;
                if ks.bKeyDown == 0 {
                    return Event::None;
                }

                let mut modif = 0;
                if state.input_mode & INPUT_ALT != 0 {
                    if state.alt_mode_esc {
                        modif = MOD_ALT;
                        state.alt_mode_esc = false;
                    }
                    if ks.dwControlKeyState & (winapi::LEFT_ALT_PRESSED | winapi::RIGHT_ALT_PRESSED) != 0 {
                        modif = MOD_ALT;
                    }
                }
                let ctrl_pressed = ks.dwControlKeyState & (winapi::LEFT_ALT_PRESSED | winapi::RIGHT_ALT_PRESSED) != 0;

                let mut key = 0;
                let key_code = ks.wVirtualKeyCode as i32;
                if key_code >= winapi::VK_F1 && key_code <= winapi::VK_F12 {
                    key = match key_code {
                        winapi::VK_F1 => KEY_F1,
                        winapi::VK_F2 => KEY_F2,
                        winapi::VK_F3 => KEY_F3,
                        winapi::VK_F4 => KEY_F4,
                        winapi::VK_F5 => KEY_F5,
                        winapi::VK_F6 => KEY_F6,
                        winapi::VK_F7 => KEY_F7,
                        winapi::VK_F8 => KEY_F8,
                        winapi::VK_F9 => KEY_F9,
                        winapi::VK_F10 => KEY_F10,
                        winapi::VK_F11 => KEY_F11,
                        _ => KEY_F12,
                    };
                    return Event::Key(key, 0 as char, modif);
                } else if key_code <= winapi::VK_DELETE {
                    key = match key_code {
                        winapi::VK_INSERT => KEY_INSERT,
                        winapi::VK_DELETE => KEY_DELETE,
                        winapi::VK_HOME => KEY_HOME,
                        winapi::VK_END => KEY_END,
                        winapi::VK_PRIOR => KEY_PGUP,
                        winapi::VK_NEXT => KEY_PGDN,
                        winapi::VK_UP => KEY_ARROW_UP,
                        winapi::VK_DOWN => KEY_ARROW_DOWN,
                        winapi::VK_LEFT => KEY_ARROW_LEFT,
                        winapi::VK_RIGHT => KEY_ARROW_RIGHT,
                        winapi::VK_BACK => if ctrl_pressed { KEY_BACKSPACE_2} else { KEY_BACKSPACE },
                        winapi::VK_TAB => KEY_TAB,
                        winapi::VK_RETURN=> KEY_ENTER,
                        winapi::VK_ESCAPE => {
                            if state.input_mode & INPUT_ESC != 0 {
                                KEY_ESC
                            } else if state.input_mode & INPUT_ALT != 0 {
                                state.alt_mode_esc = true;
                                return Event::None;
                            } else {
                                0
                            }
                        },
                        winapi::VK_SPACE => if ctrl_pressed {
                                                KEY_CTRL_SPACE
                                            } else {
                                                KEY_SPACE
                                            },
                        _ => 0,
                    };

                    if key != 0 {
                        return Event::Key(key, 0 as char, modif);
                    }
                }

                if ctrl_pressed {
                    key = ks.UnicodeChar as Key;
                    if key >= KEY_CTRL_A && key <= KEY_CTRL_RSQ_BRACKET {
                        if state.input_mode & INPUT_ALT != 0 && key == KEY_ESC {
                            state.alt_mode_esc = false;
                            return Event::None
                        }
                        return Event::Key(key, 0 as char, modif);
                    }

                    key = match ks.wVirtualKeyCode {
                        192 | 50 => KEY_CTRL_2,
                        51 => if state.input_mode & INPUT_ALT != 0 {
                                    state.alt_mode_esc = true;
                                    0
                              } else {
                                  KEY_CTRL_3
                              },
                        52 => KEY_CTRL_4,
                        53 => KEY_CTRL_5,
                        54 => KEY_CTRL_6,
                        55 | 189 | 191 => KEY_CTRL_7,
                        56 | 8 => KEY_CTRL_8,
                        _ => 0,
                    };

                    if key != 0 {
                        return Event::Key(key, 0 as char, modif);
                    }
                }

                if ks.UnicodeChar != 0 {
                    let v = &[ks.UnicodeChar];
                    let s = String::from_utf16_lossy(v);
                    let c = s.chars().next().unwrap();
                    return Event::Key(key, c, modif);
                }

                Event::None
            },
            _ => Event::None,
        }

    }

    pub fn new(mode: InputMode, sender: SyncSender<Event>) -> Self {
        let (t, recv) = channel();

        let mut wt = WinTerminal{
            tx: t,
            stdin_worker: None,
            input_mode: mode,
            event_chan: sender,
        };

        {
            let md = wt.input_mode;
            let chan_clone = wt.event_chan.clone();

            wt.stdin_worker = Some(thread::spawn(move || {
                let mut state = ThreadState {
                    last_state: 0,
                    last_x: 0xFF,
                    last_y: 0xFF,
                    last_button: MOUSE_RELEASE,
                    last_button_pressed: 0,
                    alt_mode_esc: false,
                    input_mode: md,
                    repeat_count: 0,
                };

                unsafe {
                    let hin = kernel32::GetStdHandle(self::winapi::STD_INPUT_HANDLE);
                    if hin == winapi::INVALID_HANDLE_VALUE {
                        panic!("NO STDIN");
                    }
                    // TODO: save and restore old console MODE
                    kernel32::SetConsoleMode(hin,
                                             winapi::ENABLE_WINDOW_INPUT
                                             | winapi::ENABLE_MOUSE_INPUT
                                             | winapi::ENABLE_EXTENDED_FLAGS);
                    let h1: Vec<winapi::HANDLE> = vec![hin];

                    loop {
                        let riter = recv.try_iter().next();
                        if riter.is_some() {
                            break;
                        }

                        let res = kernel32::WaitForMultipleObjects(1, h1.as_ptr(), 0, 50);
                        match res {
                            winapi::WAIT_TIMEOUT => { continue },
                            winapi::WAIT_FAILED => panic!("WaitForMultipleObject failed"),
                            _ => {
                                let mut ir: winapi::INPUT_RECORD = winapi::INPUT_RECORD {
                                    EventType: winapi::MENU_EVENT,
                                    Event: [0u32, 0u32, 0u32, 0u32],
                                };
                                let mut read: winapi::DWORD = 1;
                                let res = kernel32::ReadConsoleInputW(hin, &mut ir, 1, &mut read);
                                if res == 0 {
                                    panic!("Failed to read console input");
                                }

                                // TODO: if the last and new events are MOUSE_MOVE
                                // replace the event on top of the vector
                                let ev = WinTerminal::input_record_to_event(ir, &mut state);

                                match ev {
                                    Event::None => {},
                                    _ => {
                                        // TODO: unwrap
                                        chan_clone.send(ev.clone()).unwrap();
                                    }
                                }
                            },

                        }
                    }
                }
            }));
        }

        wt
    }
}

impl TerminalManager for WinTerminal {
    fn write(self: &Self, buf: &CellBuf) -> Result<(), String> {
        let rect = &buf.dirty_rect;
        if rect.left == -1 {
            return Ok(());
        }

        let height = rect.bottom - rect.top + 1;
        let width = rect.right - rect.left + 1;

        let mut v: Vec<CHAR_INFO> = vec![];

        for y in 0..height {
            for x in 0..width {
                if let Some(cl) = buf.get_cell(rect.left + x, rect.top + y) {
                    let (attr, vec) = cell_to_char_info(&cl);
                    v.push(CHAR_INFO{Attributes: attr, UnicodeChar: vec[0]});
                    //if v.len() > 1 {
                    //    v.push(CHAR_INFO{Attributes: attr, UnicodeChar: v[0]});
                    //}
                } else {
                    let c = Cell{ch: ' ', bg: COLOR_BLACK, fg: COLOR_WHITE};
                    let (attr, vec) = cell_to_char_info(&c);
                    v.push(CHAR_INFO{Attributes: attr, UnicodeChar: vec[0]});
                }
            }
        }

        let res: i32;
        let size: COORD = COORD{
            X: width as i16,
            Y: height as i16,
        };
        let coord: COORD = COORD{X: 0, Y: 0};
        let errcode: DWORD;
        let mut region: SMALL_RECT =
            SMALL_RECT{
                Left: rect.left as i16,
                Top: rect.top as i16,
                Right: rect.right as i16,
                Bottom: rect.bottom as i16,
            };
        unsafe {
            let h = WinTerminal::stdout_handle();
            res = kernel32::WriteConsoleOutputW(h, v.as_ptr(), size, coord, &mut region);
            errcode = kernel32::GetLastError();
        }

        match res {
            0 => Err(format!("Failed to output: {}", errcode)),
            _ => Ok(()),
        }
    }

    fn set_cursor_pos(&self, x: i16, y: i16) -> Result<(), String> {
        let coord: COORD = COORD{
            X: x,
            Y: y,
        };
        let res: BOOL;
        let errcode: DWORD;

        unsafe {
            let h = WinTerminal::stdout_handle();
            res = kernel32::SetConsoleCursorPosition(h, coord);
            errcode = kernel32::GetLastError();
        }
        match res {
            FALSE => Err(format!("Failed to set cursor position: {}", errcode)),
            _ => Ok(())
        }
    }

    fn get_cursor_pos(&self) -> Result<CursorInfo, String> {
        let mut cinfo: CONSOLE_SCREEN_BUFFER_INFO = CONSOLE_SCREEN_BUFFER_INFO{
            dwSize: COORD { X: 0, Y: 0},
            dwCursorPosition: COORD { X: 0, Y: 0},
            wAttributes: 0,
            srWindow: SMALL_RECT {
                Left: 0,
                Top: 0,
                Right: 0,
                Bottom: 0,
            },
            dwMaximumWindowSize: COORD { X: 0, Y: 0 }
        };
        let res: BOOL;
        let errcode: DWORD;

        unsafe {
            let h = WinTerminal::stdout_handle();
            res = kernel32::GetConsoleScreenBufferInfo(h, &mut cinfo);
            errcode = kernel32::GetLastError();
        }

        match res {
            FALSE => Err(format!("Failed to get cursor position: {}", errcode)),
            _ => Ok(CursorInfo{
                    // TODO: add function to show/hide cursor
                    visible: true,
                    x: cinfo.dwCursorPosition.X,
                    y: cinfo.dwCursorPosition.Y,
            })
        }
    }

    fn size(&self) -> Result<Point, String> {
        let mut cinfo: CONSOLE_SCREEN_BUFFER_INFO = CONSOLE_SCREEN_BUFFER_INFO{
            dwSize: COORD { X: 0, Y: 0},
            dwCursorPosition: COORD { X: 0, Y: 0},
            wAttributes: 0,
            srWindow: SMALL_RECT {
                Left: 0,
                Top: 0,
                Right: 0,
                Bottom: 0,
            },
            dwMaximumWindowSize: COORD { X: 0, Y: 0 }
        };
        let res: BOOL;
        let errcode: DWORD;

        unsafe {
            let h = WinTerminal::stdout_handle();
            res = kernel32::GetConsoleScreenBufferInfo(h, &mut cinfo);
            errcode = kernel32::GetLastError();
        }
        match res {
            FALSE => Err(format!("Failed to get console size: {}", errcode)),
            _ => Ok( Point{
                    x: cinfo.dwSize.X as i32,
                    y: cinfo.dwSize.Y as i32,
                 })
        }
    }
}
