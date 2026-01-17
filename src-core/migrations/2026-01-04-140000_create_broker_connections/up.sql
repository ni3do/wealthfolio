CREATE TABLE broker_connections (
    id TEXT PRIMARY KEY NOT NULL,
    broker_type TEXT NOT NULL,
    name TEXT NOT NULL,
    account_id TEXT,
    config TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_sync_at TIMESTAMP,
    last_sync_status TEXT,
    last_sync_error TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE SET NULL
);

CREATE INDEX idx_broker_connections_broker_type ON broker_connections(broker_type);
CREATE INDEX idx_broker_connections_account_id ON broker_connections(account_id);
CREATE INDEX idx_broker_connections_is_active ON broker_connections(is_active);
