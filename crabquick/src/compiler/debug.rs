//! Debug information (pc2line mapping)

/// Debug information for mapping PC to source line/column
pub struct DebugInfo {
    // TODO: Implement fields:
    // - pc2line: Vec<u8> (compressed line number mapping)
    _placeholder: u8,
}

impl DebugInfo {
    /// Creates new debug info
    pub fn new() -> Self {
        DebugInfo {
            _placeholder: 0,
        }
    }

    /// Adds a PC to line mapping
    pub fn add_mapping(&mut self, _pc: u32, _line: u32, _column: u32) {
        // TODO: Compress and add mapping
    }

    /// Gets the line number for a given PC
    pub fn get_line(&self, _pc: u32) -> Option<(u32, u32)> {
        // TODO: Decompress and binary search
        None
    }
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self::new()
    }
}
