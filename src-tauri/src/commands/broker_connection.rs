use crate::context::ServiceContext;
use crate::events::{emit_resource_changed, ResourceEventPayload};
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State};
use wealthfolio_core::broker_connections::{
    BrokerConnection, BrokerConnectionConfig, DataFormat, SyncInterval, SyncResult,
};

/// Request payload for creating a new broker connection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBrokerConnectionRequest {
    pub broker_type: String,
    pub name: String,
    pub account_id: Option<String>,
    pub query_id: String,
    pub token: String,
    pub sync_interval: Option<String>,
    pub format: Option<String>,
    pub auto_import: bool,
}

/// Request payload for updating a broker connection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBrokerConnectionRequest {
    pub name: Option<String>,
    pub account_id: Option<String>,
    pub query_id: Option<String>,
    pub token: Option<String>,
    pub sync_interval: Option<String>,
    pub format: Option<String>,
    pub auto_import: Option<bool>,
}

#[tauri::command]
pub async fn get_broker_connections(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<BrokerConnection>, String> {
    debug!("Fetching all broker connections...");
    Ok(state
        .broker_connection_service()
        .get_all_connections()?)
}

#[tauri::command]
pub async fn get_broker_connection(
    connection_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Option<BrokerConnection>, String> {
    debug!("Fetching broker connection: {}", connection_id);
    Ok(state
        .broker_connection_service()
        .get_connection(&connection_id)?)
}

#[tauri::command]
pub async fn create_broker_connection(
    request: CreateBrokerConnectionRequest,
    state: State<'_, Arc<ServiceContext>>,
    handle: AppHandle,
) -> Result<BrokerConnection, String> {
    debug!("Creating broker connection: {}", request.name);

    // Parse sync interval
    let sync_interval = match request.sync_interval.as_deref() {
        Some("daily") => SyncInterval::Daily,
        Some("weekly") => SyncInterval::Weekly,
        _ => SyncInterval::Manual,
    };

    // Parse format
    let format = match request.format.as_deref() {
        Some("csv") => DataFormat::Csv,
        _ => DataFormat::Xml,
    };

    // Create config
    let mut config = BrokerConnectionConfig::new(request.query_id);
    config.sync_interval = sync_interval;
    config.format = format;
    config.auto_import = request.auto_import;

    // Create connection
    let connection = state
        .broker_connection_service()
        .create_connection(
            request.broker_type,
            request.name,
            request.account_id,
            config,
            request.token,
        )?;

    // Emit event
    emit_resource_changed(
        &handle,
        ResourceEventPayload {
            resource: "broker_connections".to_string(),
            action: "created".to_string(),
            data: serde_json::to_value(&connection).ok(),
        },
    );

    Ok(connection)
}

#[tauri::command]
pub async fn update_broker_connection(
    connection_id: String,
    request: UpdateBrokerConnectionRequest,
    state: State<'_, Arc<ServiceContext>>,
    handle: AppHandle,
) -> Result<BrokerConnection, String> {
    debug!("Updating broker connection: {}", connection_id);

    // Get current connection to update config
    let current = state
        .broker_connection_service()
        .get_connection(&connection_id)?
        .ok_or_else(|| format!("Connection {} not found", connection_id))?;

    let mut config = current.config.clone();

    // Update config if provided
    if let Some(query_id) = request.query_id {
        config.query_id = query_id;
    }

    if let Some(sync_interval) = request.sync_interval {
        config.sync_interval = match sync_interval.as_str() {
            "daily" => SyncInterval::Daily,
            "weekly" => SyncInterval::Weekly,
            _ => SyncInterval::Manual,
        };
    }

    if let Some(format) = request.format {
        config.format = match format.as_str() {
            "csv" => DataFormat::Csv,
            _ => DataFormat::Xml,
        };
    }

    if let Some(auto_import) = request.auto_import {
        config.auto_import = auto_import;
    }

    // Update connection
    let updated = state
        .broker_connection_service()
        .update_connection(
            &connection_id,
            request.name,
            request.account_id,
            Some(config),
            request.token,
        )?;

    // Emit event
    emit_resource_changed(
        &handle,
        ResourceEventPayload {
            resource: "broker_connections".to_string(),
            action: "updated".to_string(),
            data: serde_json::to_value(&updated).ok(),
        },
    );

    Ok(updated)
}

#[tauri::command]
pub async fn delete_broker_connection(
    connection_id: String,
    state: State<'_, Arc<ServiceContext>>,
    handle: AppHandle,
) -> Result<(), String> {
    debug!("Deleting broker connection: {}", connection_id);

    state
        .broker_connection_service()
        .delete_connection(&connection_id)?;

    // Emit event
    emit_resource_changed(
        &handle,
        ResourceEventPayload {
            resource: "broker_connections".to_string(),
            action: "deleted".to_string(),
            data: serde_json::json!({ "id": connection_id }),
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn toggle_broker_connection(
    connection_id: String,
    is_active: bool,
    state: State<'_, Arc<ServiceContext>>,
    handle: AppHandle,
) -> Result<BrokerConnection, String> {
    debug!(
        "Toggling broker connection: {} to {}",
        connection_id, is_active
    );

    let updated = state
        .broker_connection_service()
        .toggle_connection(&connection_id, is_active)?;

    // Emit event
    emit_resource_changed(
        &handle,
        ResourceEventPayload {
            resource: "broker_connections".to_string(),
            action: "updated".to_string(),
            data: serde_json::to_value(&updated).ok(),
        },
    );

    Ok(updated)
}

#[tauri::command]
pub async fn test_broker_connection(
    connection_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<bool, String> {
    debug!("Testing broker connection: {}", connection_id);

    // Test the connection
    let result = state
        .ibkr_sync_service()
        .test_connection(&connection_id)
        .await?;

    Ok(result)
}

#[tauri::command]
pub async fn sync_broker_connection(
    connection_id: String,
    state: State<'_, Arc<ServiceContext>>,
    handle: AppHandle,
) -> Result<SyncResult, String> {
    debug!("Syncing broker connection: {}", connection_id);

    // Execute sync
    let result = state
        .ibkr_sync_service()
        .sync_connection(&connection_id)
        .await?;

    // Emit event for activities update
    emit_resource_changed(
        &handle,
        ResourceEventPayload {
            resource: "activities".to_string(),
            action: "synced".to_string(),
            data: serde_json::to_value(&result).ok(),
        },
    );

    Ok(result)
}

#[tauri::command]
pub async fn sync_broker_connection_for_review(
    connection_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<wealthfolio_core::activities::ActivityImport>, String> {
    debug!("Syncing broker connection for review: {}", connection_id);

    // Execute sync and return activities for review
    let activities = state
        .ibkr_sync_service()
        .sync_for_review(&connection_id)
        .await?;

    Ok(activities)
}
