use crate::broker_connections::{
    BrokerConnection, BrokerConnectionConfig, BrokerConnectionRepository, NewBrokerConnection,
    UpdateBrokerConnection, UpdateSyncStatus,
};
use crate::errors::{Error, Result};
use crate::secrets::SecretStore;
use std::sync::Arc;

pub struct BrokerConnectionService {
    repository: Arc<dyn BrokerConnectionRepository>,
    secret_store: Arc<dyn SecretStore>,
}

impl BrokerConnectionService {
    pub fn new(
        repository: Arc<dyn BrokerConnectionRepository>,
        secret_store: Arc<dyn SecretStore>,
    ) -> Self {
        Self {
            repository,
            secret_store,
        }
    }

    /// Create a new broker connection with secure token storage
    pub fn create_connection(
        &self,
        broker_type: String,
        name: String,
        account_id: Option<String>,
        config: BrokerConnectionConfig,
        token: String,
    ) -> Result<BrokerConnection> {
        // Create the connection in database
        let new_connection = NewBrokerConnection::new(broker_type, name, account_id, config)?;
        let connection = self.repository.create(new_connection)?;

        // Store token in secure keyring
        let service_id = format!("broker_connection_{}", connection.id);
        self.secret_store.set_secret(&service_id, &token)?;

        Ok(connection)
    }

    /// Get a broker connection by ID
    pub fn get_connection(&self, id: &str) -> Result<Option<BrokerConnection>> {
        self.repository.get_by_id(id)
    }

    /// Get all broker connections
    pub fn get_all_connections(&self) -> Result<Vec<BrokerConnection>> {
        self.repository.get_all()
    }

    /// Get active broker connections
    pub fn get_active_connections(&self) -> Result<Vec<BrokerConnection>> {
        self.repository.get_active()
    }

    /// Get connections by broker type
    pub fn get_connections_by_type(&self, broker_type: &str) -> Result<Vec<BrokerConnection>> {
        self.repository.get_by_broker_type(broker_type)
    }

    /// Get connections for a specific account
    pub fn get_connections_by_account(&self, account_id: &str) -> Result<Vec<BrokerConnection>> {
        self.repository.get_by_account_id(account_id)
    }

    /// Update a broker connection
    pub fn update_connection(
        &self,
        id: &str,
        name: Option<String>,
        account_id: Option<String>,
        config: Option<BrokerConnectionConfig>,
        token: Option<String>,
    ) -> Result<BrokerConnection> {
        let mut update = UpdateBrokerConnection::new();

        if let Some(n) = name {
            update = update.with_name(n);
        }

        if account_id.is_some() {
            update = update.with_account_id(account_id);
        }

        if let Some(c) = config {
            update = update.with_config(c)?;
        }

        // Update token if provided
        if let Some(t) = token {
            let service_id = format!("broker_connection_{}", id);
            self.secret_store.set_secret(&service_id, &t)?;
        }

        self.repository.update(id, update)
    }

    /// Toggle connection active status
    pub fn toggle_connection(&self, id: &str, is_active: bool) -> Result<BrokerConnection> {
        let update = UpdateBrokerConnection::new().with_is_active(is_active);
        self.repository.update(id, update)
    }

    /// Delete a broker connection and its stored token
    pub fn delete_connection(&self, id: &str) -> Result<()> {
        // Delete token from keyring first
        let service_id = format!("broker_connection_{}", id);
        self.secret_store.delete_secret(&service_id)?;

        // Delete connection from database
        self.repository.delete(id)?;

        Ok(())
    }

    /// Get the stored token for a connection
    pub fn get_connection_token(&self, id: &str) -> Result<Option<String>> {
        let service_id = format!("broker_connection_{}", id);
        self.secret_store.get_secret(&service_id)
    }

    /// Update sync status to "in progress"
    pub fn mark_sync_in_progress(&self, id: &str) -> Result<()> {
        let status = UpdateSyncStatus::in_progress();
        self.repository.update_sync_status(id, status)
    }

    /// Update sync status to "success"
    pub fn mark_sync_success(&self, id: &str) -> Result<()> {
        let status = UpdateSyncStatus::success();
        self.repository.update_sync_status(id, status)
    }

    /// Update sync status to "error"
    pub fn mark_sync_error(&self, id: &str, error_message: String) -> Result<()> {
        let status = UpdateSyncStatus::error(error_message);
        self.repository.update_sync_status(id, status)
    }

    /// Test a connection by verifying credentials exist
    pub fn test_connection(&self, id: &str) -> Result<bool> {
        // Verify connection exists
        let connection = self
            .repository
            .get_by_id(id)?
            .ok_or_else(|| Error::NotFound(format!("Connection {} not found", id)))?;

        // Verify token exists in keyring
        let token = self.get_connection_token(id)?;
        if token.is_none() {
            return Err(Error::Secret(
                "Token not found in secure storage".to_string(),
            ));
        }

        // Verify connection is active
        if !connection.is_active {
            return Err(Error::InvalidInput(
                "Connection is not active".to_string(),
            ));
        }

        Ok(true)
    }

    /// Update connection config (e.g., to store last reference code after sync)
    pub fn update_connection_config(
        &self,
        id: &str,
        config: BrokerConnectionConfig,
    ) -> Result<BrokerConnection> {
        let update = UpdateBrokerConnection::new().with_config(config)?;
        self.repository.update(id, update)
    }
}
