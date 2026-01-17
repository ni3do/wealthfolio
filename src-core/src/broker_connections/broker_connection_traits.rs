use crate::broker_connections::{
    BrokerConnection, NewBrokerConnection, UpdateBrokerConnection, UpdateSyncStatus,
};
use crate::Result;

/// Repository trait for broker connection operations
pub trait BrokerConnectionRepository: Send + Sync {
    /// Create a new broker connection
    fn create(&self, new_connection: NewBrokerConnection) -> Result<BrokerConnection>;

    /// Get a broker connection by ID
    fn get_by_id(&self, id: &str) -> Result<Option<BrokerConnection>>;

    /// Get all broker connections
    fn get_all(&self) -> Result<Vec<BrokerConnection>>;

    /// Get active broker connections
    fn get_active(&self) -> Result<Vec<BrokerConnection>>;

    /// Get broker connections by type (e.g., "IBKR")
    fn get_by_broker_type(&self, broker_type: &str) -> Result<Vec<BrokerConnection>>;

    /// Get broker connections for a specific account
    fn get_by_account_id(&self, account_id: &str) -> Result<Vec<BrokerConnection>>;

    /// Update a broker connection
    fn update(&self, id: &str, update: UpdateBrokerConnection) -> Result<BrokerConnection>;

    /// Update sync status
    fn update_sync_status(&self, id: &str, status: UpdateSyncStatus) -> Result<()>;

    /// Delete a broker connection
    fn delete(&self, id: &str) -> Result<()>;
}
