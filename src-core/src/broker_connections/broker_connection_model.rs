use crate::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

/// Configuration for a broker connection stored as JSON
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrokerConnectionConfig {
    pub query_id: String,
    pub sync_interval: SyncInterval,
    pub format: DataFormat,
    pub auto_import: bool,
    pub last_reference_code: Option<String>,
}

impl BrokerConnectionConfig {
    pub fn new(query_id: String) -> Self {
        Self {
            query_id,
            sync_interval: SyncInterval::Manual,
            format: DataFormat::Xml,
            auto_import: false,
            last_reference_code: None,
        }
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

/// Sync interval options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncInterval {
    Manual,
    Daily,
    Weekly,
}

/// Data format options for broker API responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataFormat {
    Xml,
    Csv,
}

/// Domain model representing a broker connection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerConnection {
    pub id: String,
    pub broker_type: String,
    pub name: String,
    pub account_id: Option<String>,
    pub config: BrokerConnectionConfig,
    pub is_active: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database model for broker connections
#[derive(
    Queryable,
    Identifiable,
    Selectable,
    AsChangeset,
    PartialEq,
    Serialize,
    Deserialize,
    Debug,
    Clone,
)]
#[diesel(table_name = crate::schema::broker_connections)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(treat_none_as_null = true)]
pub struct BrokerConnectionDB {
    pub id: String,
    pub broker_type: String,
    pub name: String,
    pub account_id: Option<String>,
    pub config: String, // JSON string
    pub is_active: bool,
    pub last_sync_at: Option<String>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl BrokerConnectionDB {
    /// Convert database model to domain model
    pub fn to_domain(&self) -> Result<BrokerConnection> {
        Ok(BrokerConnection {
            id: self.id.clone(),
            broker_type: self.broker_type.clone(),
            name: self.name.clone(),
            account_id: self.account_id.clone(),
            config: BrokerConnectionConfig::from_json(&self.config)?,
            is_active: self.is_active,
            last_sync_at: self
                .last_sync_at
                .as_ref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            last_sync_status: self.last_sync_status.clone(),
            last_sync_error: self.last_sync_error.clone(),
            created_at: DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| crate::Error::ParseError(e.to_string()))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&self.updated_at)
                .map_err(|e| crate::Error::ParseError(e.to_string()))?
                .with_timezone(&Utc),
        })
    }
}

/// Model for creating a new broker connection
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::broker_connections)]
#[serde(rename_all = "camelCase")]
pub struct NewBrokerConnection {
    pub id: String,
    pub broker_type: String,
    pub name: String,
    pub account_id: Option<String>,
    pub config: String, // JSON string
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl NewBrokerConnection {
    pub fn new(
        broker_type: String,
        name: String,
        account_id: Option<String>,
        config: BrokerConnectionConfig,
    ) -> Result<Self> {
        let now = Utc::now().to_rfc3339();
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            broker_type,
            name,
            account_id,
            config: config.to_json()?,
            is_active: true,
            created_at: now.clone(),
            updated_at: now,
        })
    }
}

/// Model for updating broker connection
#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::broker_connections)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBrokerConnection {
    pub name: Option<String>,
    pub account_id: Option<String>,
    pub config: Option<String>, // JSON string
    pub is_active: Option<bool>,
    pub updated_at: String,
}

impl UpdateBrokerConnection {
    pub fn new() -> Self {
        Self {
            name: None,
            account_id: None,
            config: None,
            is_active: None,
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_account_id(mut self, account_id: Option<String>) -> Self {
        self.account_id = account_id;
        self
    }

    pub fn with_config(mut self, config: BrokerConnectionConfig) -> Result<Self> {
        self.config = Some(config.to_json()?);
        Ok(self)
    }

    pub fn with_is_active(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }
}

impl Default for UpdateBrokerConnection {
    fn default() -> Self {
        Self::new()
    }
}

/// Model for updating sync status
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::broker_connections)]
pub struct UpdateSyncStatus {
    pub last_sync_at: Option<String>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub updated_at: String,
}

impl UpdateSyncStatus {
    pub fn success() -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            last_sync_at: Some(now.clone()),
            last_sync_status: Some("success".to_string()),
            last_sync_error: None,
            updated_at: now,
        }
    }

    pub fn in_progress() -> Self {
        Self {
            last_sync_at: None,
            last_sync_status: Some("in_progress".to_string()),
            last_sync_error: None,
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn error(error_message: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            last_sync_at: Some(now.clone()),
            last_sync_status: Some("error".to_string()),
            last_sync_error: Some(error_message),
            updated_at: now,
        }
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub connection_id: String,
    pub status: String,
    pub activities_count: usize,
    pub error_message: Option<String>,
    pub synced_at: DateTime<Utc>,
}

impl SyncResult {
    pub fn success(connection_id: String, activities_count: usize) -> Self {
        Self {
            connection_id,
            status: "success".to_string(),
            activities_count,
            error_message: None,
            synced_at: Utc::now(),
        }
    }

    pub fn error(connection_id: String, error_message: String) -> Self {
        Self {
            connection_id,
            status: "error".to_string(),
            activities_count: 0,
            error_message: Some(error_message),
            synced_at: Utc::now(),
        }
    }
}
