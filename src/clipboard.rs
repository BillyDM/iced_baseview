//! Access the clipboard.

use std::cell::RefCell;

use copypasta::ClipboardProvider;

/// A buffer for short-term storage and transfer within and between
/// applications.
#[allow(missing_debug_implementations)]
pub struct Clipboard {
    state: State,
}

enum State {
    Connected(RefCell<copypasta::ClipboardContext>),
    Unavailable,
}

impl Clipboard {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn new() -> Self {
        let state = copypasta::ClipboardContext::new()
            .ok()
            .map(|c| State::Connected(RefCell::new(c)))
            .unwrap_or(State::Unavailable);

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
    pub fn read(&self) -> Option<String> {
        match &self.state {
            State::Connected(clipboard) => clipboard.borrow_mut().get_contents().ok(),
            State::Unavailable => None,
        }
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write(&mut self, contents: String) {
        match &mut self.state {
            State::Connected(clipboard) => match clipboard.borrow_mut().set_contents(contents) {
                Ok(()) => {}
                Err(error) => {
                    log::warn!("error writing to clipboard: {}", error)
                }
            },
            State::Unavailable => {}
        }
    }
}

impl crate::core::Clipboard for Clipboard {
    fn read(&self) -> Option<String> {
        self.read()
    }

    fn write(&mut self, contents: String) {
        self.write(contents)
    }
}
