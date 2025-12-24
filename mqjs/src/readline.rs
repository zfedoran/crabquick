//! Readline integration

#[cfg(feature = "repl")]
use rustyline::Editor;

/// Readline wrapper
pub struct Readline {
    // TODO: Wrap rustyline editor
    #[cfg(feature = "repl")]
    _editor: Editor<()>,
}

impl Readline {
    /// Creates a new readline instance
    pub fn new() -> Self {
        #[cfg(feature = "repl")]
        {
            Readline {
                _editor: Editor::<()>::new().unwrap(),
            }
        }
        #[cfg(not(feature = "repl"))]
        {
            Readline {}
        }
    }

    /// Reads a line from stdin
    pub fn read_line(&mut self, _prompt: &str) -> Option<String> {
        // TODO: Use rustyline if feature enabled
        None
    }
}

impl Default for Readline {
    fn default() -> Self {
        Self::new()
    }
}
