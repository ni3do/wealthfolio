// Placeholder for Phase 3: IBKR Data Parser
// This will parse XML/CSV responses from IBKR Flex Query into ActivityImport objects

use crate::Result;

pub struct FlexQueryParser;

impl FlexQueryParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlexQueryParser {
    fn default() -> Self {
        Self::new()
    }
}
