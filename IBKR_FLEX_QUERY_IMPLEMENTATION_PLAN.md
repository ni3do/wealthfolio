# Interactive Brokers Flex Query Integration - Implementation Plan

## Overview
Add full automated integration with IBKR Flex Query API to enable users to sync their trading activities directly from Interactive Brokers without manual CSV downloads.

## Architecture Decision: Use Existing Rust Libraries or Build Custom?

### Available Rust Crates Analysis

**Option 1: Use [`ibflex`](https://crates.io/crates/ibflex)**
- ✅ CLI and library for downloading Flex Reports
- ✅ Handles API token and query ID
- ✅ Parses XML responses
- ⚠️ May have more features than needed (CLI tools)
- ⚠️ Need to verify compatibility with latest IBKR API

**Option 2: Use [`ibkr-flex-statement`](https://lib.rs/crates/ibkr-flex-statement)**
- ✅ Recently updated (May 2025)
- ✅ Focused on parsing flex query results
- ⚠️ May not include API client (just parser)

**Option 3: Build Custom Implementation**
- ✅ Full control over implementation
- ✅ Minimal dependencies
- ✅ We already have `reqwest` and `tokio`
- ✅ IBKR Flex API is very simple (2 HTTP GET requests)
- ⚠️ Need to implement XML/CSV parsing ourselves

**RECOMMENDATION: Option 3 (Custom Implementation) with Option 2 for parsing if needed**

**Rationale:**
- The Flex Query API is extremely simple (just 2 GET requests)
- We already have all dependencies (`reqwest`, `tokio`)
- Full control over error handling and retry logic
- Can use `ibkr-flex-statement` for XML parsing if we choose XML format
- Lighter weight solution

## Technical Stack

### Infrastructure Already Available ✅
- **HTTP Client**: `reqwest` (already in Cargo.toml)
- **Async Runtime**: `tokio` (already in Cargo.toml)
- **Secret Storage**: `keyring` + `SecretStore` trait (already implemented)
- **Database**: SQLite with Diesel ORM
- **Migrations**: Diesel migrations system in place
- **Frontend**: React + TypeScript with React Query

### New Dependencies Needed
```toml
# src-core/Cargo.toml
[dependencies]
# For XML parsing if we choose XML format over CSV
quick-xml = "0.31"  # or serde-xml-rs = "0.6"
```

## Database Schema Changes

### New Table: `broker_connections`

```sql
CREATE TABLE broker_connections (
    id TEXT PRIMARY KEY,
    broker_type TEXT NOT NULL,              -- 'IBKR', 'SCHWAB', etc.
    name TEXT NOT NULL,                     -- User-friendly name
    account_id TEXT,                        -- Link to existing account (optional)
    config TEXT NOT NULL,                   -- JSON config (query_id, sync_interval, etc.)
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_sync_at TIMESTAMP,
    last_sync_status TEXT,                  -- 'success', 'error', 'in_progress'
    last_sync_error TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

CREATE INDEX idx_broker_connections_broker_type ON broker_connections(broker_type);
CREATE INDEX idx_broker_connections_account_id ON broker_connections(account_id);
```

**Config JSON structure:**
```json
{
  "query_id": "123456",
  "sync_interval": "daily",  // 'manual', 'daily', 'weekly'
  "format": "xml",           // 'xml' or 'csv'
  "auto_import": true,       // Auto-import or review first
  "last_reference_code": "789012"
}
```

**Token stored in keyring:**
- Service ID: `wealthfolio_broker_connection_{connection_id}`
- Value: IBKR API token

## Component Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (React)                        │
├─────────────────────────────────────────────────────────────────┤
│  Settings Page                    Activity Import Page          │
│  └─ Broker Connections            └─ "Sync from IBKR" Button    │
│     ├─ Add IBKR Connection                                      │
│     ├─ Test Connection                                          │
│     ├─ Manual Sync Trigger                                      │
│     └─ Connection Status                                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Tauri Commands (Rust)                        │
├─────────────────────────────────────────────────────────────────┤
│  • create_broker_connection(broker_type, config, token)         │
│  • update_broker_connection(id, config, token?)                 │
│  • delete_broker_connection(id)                                 │
│  • list_broker_connections()                                    │
│  • test_broker_connection(id)                                   │
│  • sync_broker_connection(id)                                   │
│  • get_sync_status(id)                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│               Core Business Logic (src-core)                    │
├─────────────────────────────────────────────────────────────────┤
│  BrokerConnectionService                                        │
│  ├─ CRUD operations for connections                             │
│  ├─ Credential management (via SecretStore)                     │
│  └─ Sync orchestration                                          │
│                                                                  │
│  IBKR Module (integrations/ibkr/)                               │
│  ├─ FlexQueryClient                                             │
│  │  ├─ request_statement(token, query_id) -> reference_code     │
│  │  └─ fetch_statement(token, reference_code) -> XML/CSV        │
│  ├─ FlexQueryParser                                             │
│  │  └─ parse_response(xml/csv) -> Vec<ActivityImport>           │
│  └─ IBKRSyncService                                             │
│     ├─ sync_connection(connection_id)                           │
│     └─ handle_sync_result(activities)                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  External IBKR API                              │
├─────────────────────────────────────────────────────────────────┤
│  1. SendRequest → reference_code                                │
│  2. GetStatement → XML/CSV data                                 │
└─────────────────────────────────────────────────────────────────┘
```

## File Structure

```
src-core/src/
├── broker_connections/
│   ├── mod.rs
│   ├── broker_connection_model.rs     # Data models
│   ├── broker_connection_repository.rs # Database layer
│   ├── broker_connection_service.rs    # Business logic
│   └── broker_connection_traits.rs     # Trait definitions
│
├── integrations/
│   └── ibkr/
│       ├── mod.rs
│       ├── flex_query_client.rs       # HTTP API client
│       ├── flex_query_parser.rs       # XML/CSV parser
│       ├── flex_query_models.rs       # IBKR-specific models
│       └── ibkr_sync_service.rs       # Sync orchestration
│
└── schema.rs                           # Add broker_connections table

src-core/migrations/
└── 2026-01-XX-XXXXXX_create_broker_connections/
    ├── up.sql
    └── down.sql

src-tauri/src/commands/
└── broker_connections.rs               # Tauri command handlers

src/
├── commands/
│   └── broker-connections.ts           # Frontend bridge
│
├── pages/settings/
│   └── broker-connections/
│       ├── broker-connections-page.tsx
│       ├── add-ibkr-connection-dialog.tsx
│       ├── connection-card.tsx
│       └── sync-status-indicator.tsx
│
└── pages/activity/import/
    └── hooks/
        └── use-ibkr-sync.ts            # Hook for IBKR sync
```

## Implementation Phases

### Phase 1: Core Infrastructure (Foundation)

**Goal:** Set up database, models, and basic CRUD operations

**Tasks:**
1. Create database migration for `broker_connections` table
2. Run migration and update `schema.rs`
3. Create data models:
   - `BrokerConnection`
   - `NewBrokerConnection`
   - `BrokerConnectionConfig`
4. Implement repository layer (CRUD operations)
5. Create service layer with SecretStore integration

**Files:**
- `src-core/migrations/2026-01-XX-XXXXXX_create_broker_connections/up.sql`
- `src-core/src/broker_connections/broker_connection_model.rs`
- `src-core/src/broker_connections/broker_connection_repository.rs`
- `src-core/src/broker_connections/broker_connection_service.rs`
- `src-core/src/broker_connections/mod.rs`

**Acceptance Criteria:**
- ✅ Migration runs successfully
- ✅ Can create, read, update, delete broker connections
- ✅ Tokens stored securely in keyring
- ✅ Unit tests for repository and service

---

### Phase 2: IBKR Flex Query API Client

**Goal:** Implement HTTP client to interact with IBKR API

**Tasks:**
1. Create `FlexQueryClient` struct with `reqwest`
2. Implement `request_statement()` method
   - Endpoint: `https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.SendRequest`
   - Parameters: `v=3`, `t={token}`, `q={query_id}`
   - Response: Parse XML for reference code
3. Implement `fetch_statement()` method
   - Endpoint: `https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.GetStatement`
   - Parameters: `v=3`, `t={token}`, `q={reference_code}`
   - Response: XML or CSV data
4. Add retry logic with exponential backoff
5. Add proper error handling and logging

**Files:**
- `src-core/src/integrations/ibkr/flex_query_client.rs`
- `src-core/src/integrations/ibkr/flex_query_models.rs`
- `src-core/src/integrations/ibkr/mod.rs`

**API Flow:**
```rust
// Step 1: Request statement generation
let reference_code = client.request_statement(token, query_id).await?;

// Step 2: Poll for completion and fetch (may need to wait)
let response = client.fetch_statement(token, &reference_code).await?;
```

**Acceptance Criteria:**
- ✅ Successfully requests statement from IBKR API
- ✅ Successfully fetches generated statement
- ✅ Proper error handling for API failures
- ✅ Retry logic works correctly
- ✅ Integration test with test credentials

---

### Phase 3: IBKR Data Parser

**Goal:** Parse IBKR Flex Query responses into `ActivityImport` objects

**Tasks:**
1. Implement XML parser using `quick-xml` or `serde-xml-rs`
2. Map IBKR activity types to app activity types:
   - Trades → BUY/SELL
   - Dividends → DIVIDEND
   - Withholding Tax → TAX
   - Broker Interest → INTEREST
   - Deposits/Withdrawals → DEPOSIT/WITHDRAWAL
   - Corporate Actions → SPLIT (or other)
3. Handle IBKR-specific fields:
   - Symbol mapping (e.g., IBKR uses different symbols)
   - Currency conversion
   - Fee calculation
4. Create validation logic for parsed data

**Files:**
- `src-core/src/integrations/ibkr/flex_query_parser.rs`

**IBKR XML Structure (example):**
```xml
<FlexQueryResponse>
  <FlexStatements>
    <FlexStatement>
      <Trades>
        <Trade accountId="U1234567" symbol="AAPL" dateTime="2024-01-15"
               quantity="100" tradePrice="150.00" tradeMoney="-15000.00"
               ibCommission="-1.00" currency="USD" />
      </Trades>
      <CashTransactions>
        <CashTransaction accountId="U1234567" type="Dividends"
                         symbol="MSFT" dateTime="2024-01-20"
                         amount="23.00" currency="USD" />
      </CashTransactions>
    </FlexStatement>
  </FlexStatements>
</FlexQueryResponse>
```

**Mapping Logic:**
```rust
impl FlexQueryParser {
    fn parse_trade(&self, trade: &Trade) -> Result<ActivityImport> {
        ActivityImport {
            activity_type: if trade.quantity > 0 { "BUY" } else { "SELL" },
            symbol: self.normalize_symbol(&trade.symbol),
            quantity: trade.quantity.abs(),
            unit_price: trade.trade_price,
            fee: trade.ib_commission.abs(),
            currency: trade.currency,
            date: self.parse_date(&trade.date_time)?,
            // ... other fields
        }
    }
}
```

**Acceptance Criteria:**
- ✅ Parses trades correctly
- ✅ Parses dividends, interest, fees, taxes
- ✅ Handles symbol mapping
- ✅ Handles multi-currency transactions
- ✅ Unit tests with sample IBKR XML

---

### Phase 4: Sync Orchestration Service

**Goal:** Coordinate the full sync workflow

**Tasks:**
1. Create `IBKRSyncService` to orchestrate:
   - Fetch connection config from database
   - Retrieve token from SecretStore
   - Call FlexQueryClient to get data
   - Parse response with FlexQueryParser
   - Validate activities
   - Import activities (auto or review mode)
   - Update sync status and timestamp
2. Implement error handling and rollback
3. Add logging for debugging
4. Store last sync metadata

**Files:**
- `src-core/src/integrations/ibkr/ibkr_sync_service.rs`

**Sync Flow:**
```rust
pub async fn sync_connection(
    &self,
    connection_id: &str,
) -> Result<SyncResult> {
    // 1. Load connection
    let connection = self.repo.get_connection(connection_id)?;

    // 2. Get token from keyring
    let token = self.secret_store.get_secret(&format!("broker_connection_{}", connection_id))?;

    // 3. Fetch data from IBKR
    let reference_code = self.client.request_statement(&token, &connection.config.query_id).await?;
    let data = self.client.fetch_statement(&token, &reference_code).await?;

    // 4. Parse into activities
    let activities = self.parser.parse(&data)?;

    // 5. Validate
    let validated = self.activity_service.check_activities_import(activities).await?;

    // 6. Import (if auto_import enabled)
    if connection.config.auto_import {
        self.activity_service.import_activities(validated).await?;
    }

    // 7. Update sync status
    self.repo.update_sync_status(connection_id, "success", None)?;

    Ok(SyncResult { activities_count: validated.len() })
}
```

**Acceptance Criteria:**
- ✅ Full end-to-end sync works
- ✅ Auto-import mode works
- ✅ Review mode returns activities for user approval
- ✅ Error states properly handled and logged
- ✅ Sync status updated correctly

---

### Phase 5: Tauri Commands & Frontend Bridge

**Goal:** Expose backend functionality to frontend

**Tasks:**
1. Create Tauri command handlers:
   - `create_broker_connection`
   - `update_broker_connection`
   - `delete_broker_connection`
   - `list_broker_connections`
   - `test_broker_connection`
   - `sync_broker_connection`
   - `get_sync_status`
2. Create TypeScript bindings
3. Add React Query hooks for data fetching

**Files:**
- `src-tauri/src/commands/broker_connections.rs`
- `src/commands/broker-connections.ts`
- `src/lib/types.ts` (add TypeScript types)

**Example Command:**
```rust
#[tauri::command]
pub async fn sync_broker_connection(
    connection_id: String,
    app_state: tauri::State<'_, AppState>,
) -> Result<SyncResult, String> {
    let service = app_state.ibkr_sync_service.lock().await;
    service.sync_connection(&connection_id)
        .await
        .map_err(|e| e.to_string())
}
```

**TypeScript Hook:**
```typescript
export const useSyncBrokerConnection = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (connectionId: string) =>
      commands.syncBrokerConnection(connectionId),
    onSuccess: () => {
      queryClient.invalidateQueries(['broker-connections']);
      queryClient.invalidateQueries(['activities']);
    },
  });
};
```

**Acceptance Criteria:**
- ✅ All commands work correctly
- ✅ TypeScript types match Rust structs
- ✅ React Query integration works
- ✅ Error messages propagate to frontend

---

### Phase 6: Frontend UI - Settings Page

**Goal:** UI for managing IBKR connections

**Tasks:**
1. Create "Broker Connections" section in Settings
2. Implement "Add IBKR Connection" dialog:
   - Input: Connection name
   - Input: IBKR Token
   - Input: IBKR Query ID
   - Select: Linked account (optional)
   - Select: Sync interval (manual/daily/weekly)
   - Toggle: Auto-import
3. Display connection cards with:
   - Connection name and status
   - Last sync timestamp
   - Sync now button
   - Edit/Delete buttons
4. Show sync status indicator
5. Add test connection button

**Files:**
- `src/pages/settings/broker-connections/broker-connections-page.tsx`
- `src/pages/settings/broker-connections/add-ibkr-connection-dialog.tsx`
- `src/pages/settings/broker-connections/connection-card.tsx`
- `src/pages/settings/broker-connections/sync-status-indicator.tsx`

**UI Mockup:**
```
┌─────────────────────────────────────────────────────────┐
│ Settings > Broker Connections                  [+ Add]  │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │ 🏦 Interactive Brokers - Main Account    ✓     │    │
│  │ Last synced: 2 hours ago                       │    │
│  │ 45 activities imported                         │    │
│  │                                                 │    │
│  │ [Sync Now]  [Edit]  [Delete]                   │    │
│  └────────────────────────────────────────────────┘    │
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │ 🏦 Interactive Brokers - IRA          ⚠️       │    │
│  │ Last sync failed: Invalid token                │    │
│  │                                                 │    │
│  │ [Retry]  [Edit]  [Delete]                      │    │
│  └────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Add Connection Dialog:**
```
┌─────────────────────────────────────────────────┐
│ Add IBKR Connection                      [X]    │
├─────────────────────────────────────────────────┤
│                                                  │
│ Connection Name                                 │
│ [_____________________________________]          │
│                                                  │
│ IBKR Flex Query Token                           │
│ [_____________________________________]          │
│ ℹ️ Get this from IBKR portal                    │
│                                                  │
│ Query ID                                        │
│ [_____________________________________]          │
│                                                  │
│ Linked Account (optional)                       │
│ [Select account... ▼]                           │
│                                                  │
│ Sync Interval                                   │
│ ○ Manual  ● Daily  ○ Weekly                     │
│                                                  │
│ ☑ Auto-import activities                        │
│                                                  │
│          [Test Connection]  [Cancel]  [Save]    │
└─────────────────────────────────────────────────┘
```

**Acceptance Criteria:**
- ✅ Can add new IBKR connection
- ✅ Can edit existing connection
- ✅ Can delete connection (with confirmation)
- ✅ Can test connection
- ✅ Can manually trigger sync
- ✅ Sync status updates in real-time
- ✅ Error messages displayed clearly

---

### Phase 7: Frontend UI - Activity Import Integration

**Goal:** Add "Sync from IBKR" option to activity import flow

**Tasks:**
1. Add "Sync from Broker" button to activity import page
2. Show list of configured IBKR connections
3. Allow user to select connection and sync
4. Display synced activities in preview table
5. Allow user to review and confirm import
6. Handle sync in progress state

**Files:**
- `src/pages/activity/import/activity-import-page.tsx`
- `src/pages/activity/import/hooks/use-ibkr-sync.ts`

**UI Flow:**
```
Activity Import Page
├─ Step 1: Choose Import Method
│  ├─ Upload CSV File
│  └─ Sync from Broker ← NEW
│     └─ Select IBKR Connection
├─ Step 2: Review Activities (same as CSV import)
├─ Step 3: Confirm Import (same as CSV import)
└─ Step 4: Results (same as CSV import)
```

**Acceptance Criteria:**
- ✅ "Sync from Broker" option visible
- ✅ Can select IBKR connection
- ✅ Synced activities appear in preview
- ✅ Can review and edit before import
- ✅ Import confirmation works
- ✅ Success/error feedback shown

---

### Phase 8: Background Sync (Future Enhancement)

**Goal:** Automated background syncing on schedule

**Tasks:**
1. Create background job scheduler
2. Implement daily/weekly sync jobs
3. Add notification system for sync results
4. Handle conflicts (duplicate activities)

**Note:** This phase can be deferred to a future iteration.

---

## Testing Strategy

### Unit Tests
- Repository CRUD operations
- FlexQueryClient HTTP requests (with mocked responses)
- FlexQueryParser with sample XML
- Activity mapping logic

### Integration Tests
- Full sync flow with test IBKR credentials
- Database operations with test database
- SecretStore integration

### E2E Tests
- Complete user workflow:
  1. Add IBKR connection
  2. Test connection
  3. Trigger sync
  4. Review activities
  5. Import activities
  6. Verify in database

### Manual Testing Checklist
- [ ] Create IBKR connection with valid credentials
- [ ] Create IBKR connection with invalid credentials (verify error handling)
- [ ] Sync activities from IBKR
- [ ] Auto-import activities
- [ ] Review mode before import
- [ ] Edit connection settings
- [ ] Delete connection (verify token removed from keyring)
- [ ] Test with multiple IBKR connections
- [ ] Verify duplicate detection
- [ ] Test network failures and retries

---

## Security Considerations

1. **Token Storage**
   - ✅ Use system keyring (already implemented)
   - ✅ Never store tokens in database
   - ✅ Never log tokens

2. **API Communication**
   - ✅ Use HTTPS only (IBKR API uses HTTPS)
   - ✅ Validate SSL certificates
   - ✅ Implement request timeout

3. **Error Messages**
   - ✅ Don't expose sensitive info in error messages
   - ✅ Log detailed errors server-side only

4. **User Data**
   - ✅ Connection config stored per-user (local SQLite)
   - ✅ No data sent to external servers (except IBKR)

---

## User Documentation Needed

1. **How to get IBKR Flex Query credentials:**
   - Create Flex Query in IBKR portal
   - Activate Flex Web Service
   - Copy token and query ID

2. **What data is synced:**
   - Trades (buy/sell)
   - Dividends
   - Interest
   - Fees and taxes
   - Deposits/withdrawals

3. **Sync intervals:**
   - Manual: User triggers sync
   - Daily: Auto-sync once per day
   - Weekly: Auto-sync once per week

4. **Troubleshooting:**
   - Invalid token error
   - Query ID not found
   - Network errors
   - Duplicate activities

---

## Open Questions / Decisions Needed

1. **XML vs CSV format for Flex Query?**
   - XML: More structured, easier to parse, includes more metadata
   - CSV: Simpler, but we already dealt with CSV multi-header issue
   - **Recommendation:** XML (use `quick-xml` or `serde-xml-rs`)

2. **Duplicate activity detection:**
   - How to handle activities that were already imported manually?
   - Should we check for duplicates based on (date, symbol, quantity, price)?
   - **Recommendation:** Add duplicate detection logic, show warning to user

3. **Symbol mapping:**
   - IBKR may use different symbols than other providers
   - Should we create a symbol mapping table?
   - **Recommendation:** Reuse existing symbol mapping from CSV import

4. **Polling vs Webhook:**
   - IBKR API requires polling for statement generation
   - Should we implement polling with timeout?
   - **Recommendation:** Poll every 2 seconds, max 30 seconds timeout

5. **Multi-account support:**
   - IBKR Flex Query can include multiple account IDs
   - Should we map each IBKR account to a Wealthfolio account?
   - **Recommendation:** Yes, add account mapping logic in parser

---

## Timeline Estimation

| Phase | Description | Estimated Effort |
|-------|-------------|------------------|
| Phase 1 | Core Infrastructure | 2-3 days |
| Phase 2 | IBKR API Client | 2-3 days |
| Phase 3 | Data Parser | 3-4 days |
| Phase 4 | Sync Service | 2-3 days |
| Phase 5 | Tauri Commands | 1-2 days |
| Phase 6 | Settings UI | 3-4 days |
| Phase 7 | Import Integration | 2-3 days |
| **Total** | | **15-22 days** |

**Note:** This is development time only. Add time for:
- Code review
- Testing
- Documentation
- Bug fixes

---

## Success Metrics

- ✅ Users can connect IBKR account in < 5 minutes
- ✅ Sync completes in < 30 seconds for typical dataset
- ✅ 95%+ accuracy in activity parsing
- ✅ Zero token leaks or security issues
- ✅ Clear error messages for all failure modes
- ✅ Positive user feedback on usability

---

## Future Enhancements

1. **Additional Brokers:**
   - Schwab
   - Fidelity
   - TD Ameritrade
   - E*TRADE

2. **Advanced Features:**
   - Background sync scheduler
   - Push notifications for sync completion
   - Sync history and audit log
   - Conflict resolution UI
   - Bulk connection management

3. **Performance:**
   - Incremental sync (only new activities)
   - Parallel syncing for multiple connections
   - Caching of reference codes

---

## References

- [IBKR Flex Web Service API Documentation](https://www.interactivebrokers.com/campus/ibkr-api-page/flex-web-service/)
- [ibflex Rust Crate](https://crates.io/crates/ibflex)
- [ibkr-flex-statement Rust Crate](https://lib.rs/crates/ibkr-flex-statement)
- [Existing CSV Import Implementation](src/pages/activity/import/)
