//! A library for terminal/console-based applications
//!
//! The library contains a set of basic functions that makes possible creation
//! of full-featured terminal applications with mouse and keyboard support.
//!
//! Terminal management includes
//! * output to terminal
//! * reading the current terminal content
//! * moving curosr
//! * events from mouse, keyboard, and terminal
//!
//! Output introduce the following functions:
//! * print a character
//! * print a string horintally or vertically
//! * print a horizontal or vertical line of the same character
//! All functoion above come in two flavors: printing with current colors or
//! using temporary one
//!
//! Reading functions:
//! * read value of one cell of the terminal (character with its attributes)
//! * read the entire terminal content
//!
//! Cursor functions:
//! * set cursor position
//! * get cursor position
//!
//! Events:
//! * key press event (bohe key down and key release)
//! * mouse click event
//! * mouse move event (generated only if any mouse button is pressed. In other words, only dragging with mouse generates mouse move event
//! * mouse wheel event
//! * terminal resize event
//! * terminal exit event
//!
//! ### A minimal example
//! ```
//! extern crate rterm;
//!
//! fn main() {
//!     let mut cb = rterm::Terminal::new();
//!
//!     cb.clear();
//!
//!     loop {
//!         cb.put_string(5, 3, "Hello, ");
//!         cb.put_string_with_attrs(12, 3, "World!", rterm::COLOR_GREEN, rterm::COLOR_DEFAULT);
//!         cb.flush();
//!
//!         if let Some(ev) = cb.get_event() {
//!             match ev {
//!                 rterm::Event::Key(key, _, _) =>
//!                     if key == rterm::KEY_ESC {
//!                         break;
//!                     },
//!                 _ => {}
//!             }
//!         }
//!     }
//!
//!     cb.stop();
//! }
//!
//! ```
#[macro_use]
extern crate iota;
#[macro_use]
extern crate lazy_static;
extern crate unicode_width;

pub mod common;
pub mod cellbuf;
pub mod terminal;
pub mod intf;

#[cfg(windows)] mod term_windows;

pub use common::*;
pub use cellbuf::*;
pub use terminal::*;
pub use intf::*;
