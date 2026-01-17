use crate::activities::activities_service::ActivityService;
use crate::activities::activities_traits::ActivityServiceTrait;
use crate::broker_connections::{BrokerConnectionService, SyncResult};
use crate::integrations::ibkr::{FlexQueryClient, FlexQueryParser};
use crate::Result;
use log::{debug, info, warn};
use std::sync::Arc;

/// Service for orchestrating IBKR Flex Query synchronization
pub struct IBKRSyncService {
    connection_service: Arc<BrokerConnectionService>,
    activity_service: Arc<ActivityService>,
    flex_client: FlexQueryClient,
    parser: FlexQueryParser,
}

impl IBKRSyncService {
    /// Create a new IBKR sync service
    pub fn new(
        connection_service: Arc<BrokerConnectionService>,
        activity_service: Arc<ActivityService>,
    ) -> Result<Self> {
        Ok(Self {
            connection_service,
            activity_service,
            flex_client: FlexQueryClient::new()?,
            parser: FlexQueryParser::new(),
        })
    }

    /// Synchronize activities from IBKR for a specific broker connection
    pub async fn sync_connection(&self, connection_id: &str) -> Result<SyncResult> {
        info!("Starting IBKR sync for connection: {}", connection_id);

        // Mark sync as in progress
        self.connection_service
            .mark_sync_in_progress(connection_id)?;

        // Execute sync and handle result
        match self.execute_sync(connection_id).await {
            Ok(result) => {
                info!(
                    "Sync completed successfully: {} activities",
                    result.activities_count
                );
                self.connection_service.mark_sync_success(connection_id)?;
                Ok(result)
            }
            Err(e) => {
                warn!("Sync failed: {}", e);
                self.connection_service
                    .mark_sync_error(connection_id, e.to_string())?;
                Err(e)
            }
        }
    }

    /// Execute the sync workflow
    async fn execute_sync(&self, connection_id: &str) -> Result<SyncResult> {
        // 1. Load connection from database
        let connection = self
            .connection_service
            .get_connection(connection_id)?
            .ok_or_else(|| {
                crate::Error::NotFound(format!("Connection {} not found", connection_id))
            })?;

        debug!("Loaded connection: {}", connection.name);

        // Verify connection is active
        if !connection.is_active {
            return Err(crate::Error::InvalidInput(format!(
                "Connection {} is not active",
                connection.name
            )));
        }

        // 2. Get token from secure storage
        let token = self
            .connection_service
            .get_connection_token(connection_id)?
            .ok_or_else(|| {
                crate::Error::Secret(format!(
                    "Token not found for connection {}",
                    connection_id
                ))
            })?;

        debug!("Retrieved token from secure storage");

        // 3. Fetch data from IBKR Flex Query API
        info!("Fetching data from IBKR Flex Query API...");
        let statement = self
            .flex_client
            .get_statement(&token, &connection.config.query_id)
            .await?;

        debug!("Statement retrieved successfully");

        // 4. Parse response into activities
        info!("Parsing IBKR response...");
        let mut activities = self.parser.parse(&statement)?;

        debug!("Parsed {} activities", activities.len());

        // 5. Get account ID - required for validation
        let account_id = connection.account_id.clone().ok_or_else(|| {
            crate::Error::InvalidInput(format!(
                "Connection {} has no linked account. Please link an account before syncing.",
                connection.name
            ))
        })?;

        // Set account ID for all activities
        for activity in &mut activities {
            activity.account_id = Some(account_id.clone());
        }

        // 6. Validate activities using existing activity service
        info!("Validating activities...");
        let validated_activities = self
            .activity_service
            .check_activities_import(account_id.clone(), activities)
            .await?;

        let activities_count = validated_activities.len();
        debug!("Validated {} activities", activities_count);

        // 7. Import activities if auto_import is enabled
        if connection.config.auto_import {
            info!("Auto-import enabled, importing {} activities", activities_count);

            // Filter out invalid activities
            let valid_activities: Vec<_> = validated_activities
                .into_iter()
                .filter(|a| a.is_valid)
                .collect();

            if valid_activities.is_empty() {
                warn!("No valid activities to import");
                return Ok(SyncResult::success(connection_id.to_string(), 0));
            }

            debug!("Importing {} valid activities", valid_activities.len());
            self.activity_service
                .import_activities(account_id, valid_activities)
                .await?;

            info!("Activities imported successfully");

            Ok(SyncResult::success(
                connection_id.to_string(),
                activities_count,
            ))
        } else {
            info!("Auto-import disabled, returning activities for review");

            // Return success but don't import - activities will be reviewed by user
            Ok(SyncResult::success(
                connection_id.to_string(),
                activities_count,
            ))
        }
    }

    /// Test a connection by attempting to fetch a statement
    pub async fn test_connection(&self, connection_id: &str) -> Result<bool> {
        info!("Testing IBKR connection: {}", connection_id);

        // Load connection
        let connection = self
            .connection_service
            .get_connection(connection_id)?
            .ok_or_else(|| {
                crate::Error::NotFound(format!("Connection {} not found", connection_id))
            })?;

        // Get token
        let token = self
            .connection_service
            .get_connection_token(connection_id)?
            .ok_or_else(|| crate::Error::Secret("Token not found".to_string()))?;

        // Try to request a statement (don't wait for it to complete)
        let reference_code = self
            .flex_client
            .request_statement(&token, &connection.config.query_id)
            .await?;

        info!(
            "Connection test successful, reference code: {}",
            reference_code
        );

        Ok(true)
    }

    /// Sync and return activities for review (don't import automatically)
    pub async fn sync_for_review(&self, connection_id: &str) -> Result<Vec<crate::activities::ActivityImport>> {
        info!("Syncing for review: {}", connection_id);

        // Temporarily disable auto-import
        let connection = self
            .connection_service
            .get_connection(connection_id)?
            .ok_or_else(|| {
                crate::Error::NotFound(format!("Connection {} not found", connection_id))
            })?;

        let original_auto_import = connection.config.auto_import;

        if original_auto_import {
            // Disable auto-import temporarily
            let mut new_config = connection.config.clone();
            new_config.auto_import = false;

            self.connection_service
                .update_connection_config(connection_id, new_config)?;
        }

        // Execute sync
        let result = self.execute_sync(connection_id).await;

        // Restore original auto-import setting
        if original_auto_import {
            let mut restore_config = connection.config.clone();
            restore_config.auto_import = true;

            let _ = self
                .connection_service
                .update_connection_config(connection_id, restore_config);
        }

        // If sync was successful, fetch and return the activities
        match result {
            Ok(_) => {
                // Re-fetch and parse to return activities
                let connection = self
                    .connection_service
                    .get_connection(connection_id)?
                    .ok_or_else(|| {
                        crate::Error::NotFound(format!("Connection {} not found", connection_id))
                    })?;

                let token = self
                    .connection_service
                    .get_connection_token(connection_id)?
                    .ok_or_else(|| crate::Error::Secret("Token not found".to_string()))?;

                let statement = self
                    .flex_client
                    .get_statement(&token, &connection.config.query_id)
                    .await?;

                let mut activities = self.parser.parse(&statement)?;

                // Get account ID
                let account_id = connection.account_id.clone().ok_or_else(|| {
                    crate::Error::InvalidInput("Connection has no linked account".to_string())
                })?;

                // Set account ID for all activities
                for activity in &mut activities {
                    activity.account_id = Some(account_id.clone());
                }

                // Validate
                let validated = self
                    .activity_service
                    .check_activities_import(account_id, activities)
                    .await?;

                Ok(validated)
            }
            Err(e) => Err(e),
        }
    }
}

impl Default for IBKRSyncService {
    fn default() -> Self {
        panic!("IBKRSyncService requires connection_service and activity_service to be initialized")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        // Test service creation requires proper dependencies
        // This is a placeholder for integration tests
    }
}
