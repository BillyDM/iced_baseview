//! Access the clipboard.

use std::cell::RefCell;

use copypasta::{ClipboardContext, ClipboardProvider};

use crate::command::{self, Command};

pub use iced_native::clipboard::Action;

/// A `copypasta` wrapper for iced's clipboard abstraction.
#[allow(missing_debug_implementations)]
pub struct Clipboard {
    clipboard: Option<RefCell<ClipboardContext>>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            clipboard: match copypasta::ClipboardContext::new() {
                Ok(clipboard) => Some(RefCell::new(clipboard)),
                Err(e) => {
                    eprintln!("Failed to initialize clipboard: {}", e);
                    None
                }
            },
        }
    }
}

impl iced_native::Clipboard for Clipboard {
    fn read(&self) -> Option<String> {
        match &self.clipboard {
            Some(clipboard) => clipboard.borrow_mut().get_contents().ok(),
            None => None,
        }
    }

    fn write(&mut self, contents: String) {
        if let Some(clipboard) = &self.clipboard {
            clipboard
                .borrow_mut()
                .set_contents(contents)
                .unwrap_or_else(|err| eprintln!("Error while writing to the clipboard: {err:?}"));
        }
    }
}

/// Read the current contents of the clipboard.
pub fn read<Message>(f: impl Fn(Option<String>) -> Message + 'static) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Read(Box::new(f))))
}

/// Write the given contents to the clipboard.
pub fn write<Message>(contents: String) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Write(contents)))
}
