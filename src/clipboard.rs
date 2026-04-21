//! Access the clipboard.

use std::cell::RefCell;

use crate::core::clipboard::Kind as ClipboardKind;

pub use crate::runtime::clipboard::{read, read_primary, write, write_primary};

/// A buffer for short-term storage and transfer within and between
/// applications.
#[allow(missing_debug_implementations)]
pub struct Clipboard {
    state: State,
}

enum State {
    Connected(RefCell<window_clipboard::Clipboard>),
    Unavailable,
}

impl Clipboard {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn new(window: raw_window_handle_06::RawDisplayHandle) -> Self {
        let clipboard_res = unsafe {
            let window = raw_window_handle_06::DisplayHandle::borrow_raw(window);
            window_clipboard::Clipboard::connect(&window)
        };

        let state = match clipboard_res {
            Ok(c) => State::Connected(RefCell::new(c)),
            Err(e) => {
                log::error!("Could not connect to clipboard: {}", e);
                State::Unavailable
            }
        };

        Clipboard { state }
    }

    /// Creates a new [`Clipboard`] that isn't associated with a window.
    /// This clipboard will never contain a copied value.
    pub fn unconnected() -> Clipboard {
        Clipboard {
            state: State::Unavailable,
        }
    }

    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read(&self, kind: ClipboardKind) -> Option<String> {
        match &self.state {
            State::Connected(clipboard) => match kind {
                ClipboardKind::Primary => match clipboard.borrow_mut().read_primary() {
                    Some(Ok(s)) => Some(s),
                    Some(Err(e)) => {
                        log::error!("Failed to read from primary clipboard: {}", e);
                        None
                    }
                    None => None,
                },
                ClipboardKind::Standard => match clipboard.borrow_mut().read() {
                    Ok(s) => Some(s),
                    Err(e) => {
                        log::error!("Failed to read from clipboard: {}", e);
                        None
                    }
                },
            },
            State::Unavailable => None,
        }
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write(&mut self, kind: ClipboardKind, contents: String) {
        match &mut self.state {
            State::Connected(clipboard) => match kind {
                ClipboardKind::Primary => {
                    if let Some(Err(e)) = clipboard.borrow_mut().write_primary(contents) {
                        log::warn!("Failed to write to clipboard: {}", e);
                    }
                }
                ClipboardKind::Standard => {
                    if let Err(e) = clipboard.borrow_mut().write(contents) {
                        log::warn!("Failed to write to clipboard: {}", e);
                    }
                }
            },
            State::Unavailable => {}
        }
    }
}

impl crate::core::Clipboard for Clipboard {
    fn read(&self, kind: ClipboardKind) -> Option<String> {
        self.read(kind)
    }

    fn write(&mut self, kind: ClipboardKind, contents: String) {
        self.write(kind, contents)
    }
}
