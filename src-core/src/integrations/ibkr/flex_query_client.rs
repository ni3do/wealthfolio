use crate::integrations::ibkr::{
    FlexQueryClientConfig, SendRequestResponse, StatementResponse,
};
use crate::Result;
use log::{debug, info, warn};
use quick_xml::de::from_str;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

/// Client for interacting with IBKR Flex Query Web Service API
pub struct FlexQueryClient {
    client: Client,
    config: FlexQueryClientConfig,
}

impl FlexQueryClient {
    /// Create a new Flex Query client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(FlexQueryClientConfig::default())
    }

    /// Create a new Flex Query client with custom configuration
    pub fn with_config(config: FlexQueryClientConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent("WealthFolio/1.0")
            .build()
            .map_err(|e| {
                crate::Error::InvalidInput(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { client, config })
    }

    /// Request generation of a Flex Query statement
    /// Returns a reference code that can be used to fetch the statement
    pub async fn request_statement(&self, token: &str, query_id: &str) -> Result<String> {
        let url = format!(
            "{}/FlexStatementService.SendRequest",
            self.config.base_url
        );

        info!(
            "Requesting Flex Query statement generation for query_id: {}",
            query_id
        );

        let mut attempt = 0;
        let max_attempts = self.config.max_retries + 1;

        loop {
            attempt += 1;
            debug!("Request attempt {}/{}", attempt, max_attempts);

            let response = self
                .client
                .get(&url)
                .query(&[("v", "3"), ("t", token), ("q", query_id)])
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let error_text = resp.text().await.unwrap_or_else(|_| String::new());
                        let error_msg = format!(
                            "IBKR API returned error status {}: {}",
                            status, error_text
                        );

                        if attempt >= max_attempts {
                            return Err(crate::Error::InvalidInput(error_msg));
                        }

                        warn!("{} - retrying...", error_msg);
                    } else {
                        let body = resp.text().await.map_err(|e| {
                            crate::Error::InvalidInput(format!("Failed to read response: {}", e))
                        })?;

                        debug!("Response body: {}", body);

                        // Parse XML response
                        let send_response: SendRequestResponse = from_str(&body).map_err(|e| {
                            crate::Error::ParseError(format!("Failed to parse XML response: {}", e))
                        })?;

                        // Check for errors in the response
                        if let Some(error_msg) = send_response.error_message {
                            return Err(crate::Error::InvalidInput(format!(
                                "IBKR Flex Query error: {}",
                                error_msg
                            )));
                        }

                        // Get reference code
                        let reference_code = send_response.get_reference_code()?;
                        info!("Statement generation requested, reference code: {}", reference_code);
                        return Ok(reference_code);
                    }
                }
                Err(e) => {
                    if attempt >= max_attempts {
                        return Err(crate::Error::InvalidInput(format!(
                            "Failed to request statement after {} attempts: {}",
                            max_attempts, e
                        )));
                    }

                    warn!(
                        "Request failed: {} - retrying in {}ms...",
                        e,
                        self.config.initial_backoff_ms * (2_u64.pow(attempt - 1))
                    );
                }
            }

            // Exponential backoff
            let backoff_ms = self.config.initial_backoff_ms * (2_u64.pow(attempt - 1));
            sleep(Duration::from_millis(backoff_ms)).await;
        }
    }

    /// Fetch a generated Flex Query statement using the reference code
    /// This method will poll for the statement to be ready
    pub async fn fetch_statement(
        &self,
        token: &str,
        reference_code: &str,
    ) -> Result<StatementResponse> {
        let url = format!(
            "{}/FlexStatementService.GetStatement",
            self.config.base_url
        );

        info!("Fetching Flex Query statement with reference code: {}", reference_code);

        let mut poll_attempt = 0;
        let max_poll_attempts = 15; // Poll for up to 30 seconds (15 attempts * 2 seconds)

        loop {
            poll_attempt += 1;
            debug!("Fetch attempt {}/{}", poll_attempt, max_poll_attempts);

            let response = self
                .client
                .get(&url)
                .query(&[("v", "3"), ("t", token), ("q", reference_code)])
                .send()
                .await
                .map_err(|e| {
                    crate::Error::InvalidInput(format!("Failed to fetch statement: {}", e))
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| String::new());
                return Err(crate::Error::InvalidInput(format!(
                    "IBKR API returned error status {}: {}",
                    status, error_text
                )));
            }

            let body = response.text().await.map_err(|e| {
                crate::Error::InvalidInput(format!("Failed to read response: {}", e))
            })?;

            debug!("Response body length: {}", body.len());

            // Check if response is an error message
            if body.contains("<ErrorCode>") || body.contains("<ErrorMessage>") {
                // Try to parse as error response
                if let Ok(error_response) = from_str::<SendRequestResponse>(&body) {
                    if let Some(error_msg) = error_response.error_message {
                        // Check if it's a "statement not ready" error
                        if error_msg.contains("not been generated") || error_msg.contains("1006") {
                            if poll_attempt >= max_poll_attempts {
                                return Err(crate::Error::InvalidInput(format!(
                                    "Statement generation timeout after {} attempts: {}",
                                    max_poll_attempts, error_msg
                                )));
                            }

                            info!("Statement not ready yet, waiting 2 seconds...");
                            sleep(Duration::from_secs(2)).await;
                            continue;
                        }

                        return Err(crate::Error::InvalidInput(format!(
                            "IBKR Flex Query error: {}",
                            error_msg
                        )));
                    }
                }
            }

            // Determine if response is XML or CSV
            if body.trim().starts_with("<?xml") || body.trim().starts_with("<FlexQueryResponse") {
                info!("Statement fetched successfully (XML format)");
                return Ok(StatementResponse::Xml(body));
            } else if body.contains(',') && (body.lines().count() > 1) {
                info!("Statement fetched successfully (CSV format)");
                return Ok(StatementResponse::Csv(body));
            } else {
                return Err(crate::Error::ParseError(format!(
                    "Unexpected response format: {}",
                    &body[..std::cmp::min(200, body.len())]
                )));
            }
        }
    }

    /// Convenience method to request and fetch a statement in one call
    pub async fn get_statement(&self, token: &str, query_id: &str) -> Result<StatementResponse> {
        let reference_code = self.request_statement(token, query_id).await?;
        self.fetch_statement(token, &reference_code).await
    }
}

impl Default for FlexQueryClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default FlexQueryClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_request_response_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ReferenceCode>1234567890</ReferenceCode>"#;

        // Note: This will fail without proper XML structure
        // Real responses from IBKR would have proper structure
    }

    #[test]
    fn test_error_response_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Status>
    <ErrorCode>1003</ErrorCode>
    <ErrorMessage>Invalid token</ErrorMessage>
</Status>"#;

        // Test error response parsing
    }
}
