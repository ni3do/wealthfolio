use crate::broker_connections::{
    BrokerConnection, BrokerConnectionDB, BrokerConnectionRepository, NewBrokerConnection,
    UpdateBrokerConnection, UpdateSyncStatus,
};
use crate::db::{get_connection, DbPool};
use crate::errors::{Error, Result};
use crate::schema::broker_connections::dsl::*;
use diesel::prelude::*;
use std::sync::Arc;

pub struct BrokerConnectionRepositoryImpl {
    pool: Arc<DbPool>,
}

impl BrokerConnectionRepositoryImpl {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

impl BrokerConnectionRepository for BrokerConnectionRepositoryImpl {
    fn create(&self, new_connection: NewBrokerConnection) -> Result<BrokerConnection> {
        let mut conn = get_connection(&self.pool)?;

        diesel::insert_into(broker_connections)
            .values(&new_connection)
            .execute(&mut conn)
            .map_err(Error::from)?;

        // Fetch the created connection and convert to domain model
        let db_connection = broker_connections
            .filter(id.eq(&new_connection.id))
            .first::<BrokerConnectionDB>(&mut conn)
            .map_err(Error::from)?;

        db_connection.to_domain()
    }

    fn get_by_id(&self, connection_id: &str) -> Result<Option<BrokerConnection>> {
        let mut conn = get_connection(&self.pool)?;

        let result = broker_connections
            .filter(id.eq(connection_id))
            .first::<BrokerConnectionDB>(&mut conn)
            .optional()
            .map_err(Error::from)?;

        match result {
            Some(db_connection) => Ok(Some(db_connection.to_domain()?)),
            None => Ok(None),
        }
    }

    fn get_all(&self) -> Result<Vec<BrokerConnection>> {
        let mut conn = get_connection(&self.pool)?;

        let db_connections = broker_connections
            .order(created_at.desc())
            .load::<BrokerConnectionDB>(&mut conn)
            .map_err(Error::from)?;

        db_connections
            .into_iter()
            .map(|db_conn| db_conn.to_domain())
            .collect()
    }

    fn get_active(&self) -> Result<Vec<BrokerConnection>> {
        let mut conn = get_connection(&self.pool)?;

        let db_connections = broker_connections
            .filter(is_active.eq(true))
            .order(created_at.desc())
            .load::<BrokerConnectionDB>(&mut conn)
            .map_err(Error::from)?;

        db_connections
            .into_iter()
            .map(|db_conn| db_conn.to_domain())
            .collect()
    }

    fn get_by_broker_type(&self, broker_type_param: &str) -> Result<Vec<BrokerConnection>> {
        let mut conn = get_connection(&self.pool)?;

        let db_connections = broker_connections
            .filter(broker_type.eq(broker_type_param))
            .order(created_at.desc())
            .load::<BrokerConnectionDB>(&mut conn)
            .map_err(Error::from)?;

        db_connections
            .into_iter()
            .map(|db_conn| db_conn.to_domain())
            .collect()
    }

    fn get_by_account_id(&self, account_id_param: &str) -> Result<Vec<BrokerConnection>> {
        let mut conn = get_connection(&self.pool)?;

        let db_connections = broker_connections
            .filter(account_id.eq(account_id_param))
            .order(created_at.desc())
            .load::<BrokerConnectionDB>(&mut conn)
            .map_err(Error::from)?;

        db_connections
            .into_iter()
            .map(|db_conn| db_conn.to_domain())
            .collect()
    }

    fn update(
        &self,
        connection_id: &str,
        update: UpdateBrokerConnection,
    ) -> Result<BrokerConnection> {
        let mut conn = get_connection(&self.pool)?;

        diesel::update(broker_connections.filter(id.eq(connection_id)))
            .set(&update)
            .execute(&mut conn)
            .map_err(Error::from)?;

        // Fetch updated connection
        let db_connection = broker_connections
            .filter(id.eq(connection_id))
            .first::<BrokerConnectionDB>(&mut conn)
            .map_err(Error::from)?;

        db_connection.to_domain()
    }

    fn update_sync_status(&self, connection_id: &str, status: UpdateSyncStatus) -> Result<()> {
        let mut conn = get_connection(&self.pool)?;

        diesel::update(broker_connections.filter(id.eq(connection_id)))
            .set(&status)
            .execute(&mut conn)
            .map_err(Error::from)?;

        Ok(())
    }

    fn delete(&self, connection_id: &str) -> Result<()> {
        let mut conn = get_connection(&self.pool)?;

        diesel::delete(broker_connections.filter(id.eq(connection_id)))
            .execute(&mut conn)
            .map_err(Error::from)?;

        Ok(())
    }
}
