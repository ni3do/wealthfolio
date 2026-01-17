use serde::{Deserialize, Serialize};

/// Request to generate a Flex Query statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexQueryRequest {
    pub token: String,
    pub query_id: String,
}

/// Response from requesting a Flex Query statement generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendRequestResponse {
    pub reference_code: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "ErrorCode")]
    pub error_code: Option<String>,
    #[serde(rename = "ErrorMessage")]
    pub error_message: Option<String>,
}

/// Response status for Flex Query operations
#[derive(Debug, Clone, PartialEq)]
pub enum FlexQueryStatus {
    Success,
    Pending,
    Error(String),
}

impl SendRequestResponse {
    pub fn status(&self) -> FlexQueryStatus {
        if let Some(ref err_msg) = self.error_message {
            return FlexQueryStatus::Error(err_msg.clone());
        }
        if let Some(ref code) = self.reference_code {
            if !code.is_empty() {
                return FlexQueryStatus::Success;
            }
        }
        FlexQueryStatus::Pending
    }

    pub fn get_reference_code(&self) -> crate::Result<String> {
        match &self.reference_code {
            Some(code) if !code.is_empty() => Ok(code.clone()),
            _ => Err(crate::Error::InvalidInput(
                self.error_message.clone().unwrap_or_else(|| {
                    "No reference code received from IBKR".to_string()
                }),
            )),
        }
    }
}

/// Statement fetch response (can be XML or CSV)
#[derive(Debug, Clone)]
pub enum StatementResponse {
    Xml(String),
    Csv(String),
    Error(String),
}

impl StatementResponse {
    pub fn is_error(&self) -> bool {
        matches!(self, StatementResponse::Error(_))
    }

    pub fn error_message(&self) -> Option<String> {
        match self {
            StatementResponse::Error(msg) => Some(msg.clone()),
            _ => None,
        }
    }

    pub fn content(&self) -> Result<String, String> {
        match self {
            StatementResponse::Xml(content) => Ok(content.clone()),
            StatementResponse::Csv(content) => Ok(content.clone()),
            StatementResponse::Error(msg) => Err(msg.clone()),
        }
    }
}

/// Configuration for Flex Query API client
#[derive(Debug, Clone)]
pub struct FlexQueryClientConfig {
    pub base_url: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
}

impl Default for FlexQueryClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://gdcdyn.interactivebrokers.com/Universal/servlet".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            initial_backoff_ms: 1000,
        }
    }
}

/// Error codes from IBKR Flex Query API
#[derive(Debug, Clone, PartialEq)]
pub enum FlexQueryErrorCode {
    InvalidToken,
    InvalidQueryId,
    StatementGenerationInProgress,
    StatementNotReady,
    StatementNotFound,
    Unknown(String),
}

impl From<&str> for FlexQueryErrorCode {
    fn from(code: &str) -> Self {
        match code {
            "1003" => FlexQueryErrorCode::InvalidToken,
            "1004" => FlexQueryErrorCode::InvalidQueryId,
            "1005" => FlexQueryErrorCode::StatementGenerationInProgress,
            "1006" => FlexQueryErrorCode::StatementNotReady,
            "1007" => FlexQueryErrorCode::StatementNotFound,
            other => FlexQueryErrorCode::Unknown(other.to_string()),
        }
    }
}
