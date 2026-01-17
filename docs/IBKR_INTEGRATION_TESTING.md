# IBKR Flex Query Integration - Testing Guide

This document provides comprehensive testing instructions for the Interactive Brokers Flex Query integration in WealthFolio.

## Table of Contents

1. [Test Environment Setup](#test-environment-setup)
2. [IBKR Flex Query Configuration](#ibkr-flex-query-configuration)
3. [Manual Testing Procedures](#manual-testing-procedures)
4. [Automated Testing](#automated-testing)
5. [Test Scenarios & Cases](#test-scenarios--cases)
6. [Troubleshooting](#troubleshooting)

---

## Test Environment Setup

### Prerequisites

- **Development Environment**: Node.js 18+, Rust 1.70+, pnpm
- **IBKR Account**: Active Interactive Brokers account with trading history
- **Test Data**: Sample activities (trades, dividends, deposits) in your IBKR account

### Repository Test Setup

WealthFolio uses multiple testing frameworks:

#### 1. **Frontend Tests (Vitest)**

```bash
# Run all unit tests
pnpm test

# Run tests in watch mode
pnpm test:watch

# Run tests with UI
pnpm test:ui

# Run tests with coverage
pnpm test:coverage
```

Test files are located in:
- `src/**/*.test.ts` - Unit tests for utilities
- `src/**/*.test.tsx` - Component tests

#### 2. **Backend Tests (Rust/Cargo)**

```bash
# Run all Rust tests
cd src-core
cargo test

# Run specific test
cargo test ibkr

# Run tests with output
cargo test -- --nocapture

# Run tests with logging
RUST_LOG=debug cargo test
```

Test files are located in:
- `src-core/src/**/*_tests.rs` - Unit tests
- `src-core/src/**/tests.rs` - Module tests

#### 3. **E2E Tests (Playwright)**

```bash
# Run E2E tests
pnpm test:e2e

# Run E2E tests with UI
pnpm test:e2e:ui
```

E2E tests are in `e2e-tests/` directory.

---

## IBKR Flex Query Configuration

### Required Configuration

Before testing, you must set up an IBKR Flex Query. Follow the complete guide: [IBKR_FLEX_QUERY_SETUP.md](./IBKR_FLEX_QUERY_SETUP.md)

### Quick Setup Summary

**Step 1: Create Flex Query**
- Log into IBKR Client Portal
- Navigate to **Performance & Reports** → **Flex Queries**
- Create new **Activity Flex Query**

**Step 2: Add Required Sections**

Must include these sections with all fields:

1. **Trades** section:
   - Account ID, Symbol, Date/Time
   - Quantity, Trade Price, Trade Money
   - IB Commission, Currency
   - Asset Category, Description

2. **Cash Transactions** section:
   - Account ID, Type, Symbol
   - Date/Time, Amount, Currency
   - Description

3. **Corporate Actions** section (recommended):
   - Account ID, Symbol, Date/Time
   - Type, Quantity, Currency
   - Description

**Step 3: Configure Settings**
- **Format**: XML (REQUIRED)
- **Date Format**: YYYY-MM-DD
- **Date Range**: Last 365 days (for testing), Last 30 days (for production)
- **Include Headers**: Yes

**Step 4: Activate Web Service**
- Go to Flex Queries → Flex Web Service
- Check "Activate Flex Web Service"
- Generate token (64-character string)
- **SAVE BOTH**: Query ID and Token

### Test Flex Query Configuration

For testing purposes, configure with:
- **Date Range**: Last 90 days (ensures you have data)
- **Format**: XML
- **All Sections**: Trades, Cash Transactions, Corporate Actions
- **All Fields**: Include every available field for comprehensive testing

---

## Manual Testing Procedures

### Phase 1: Connection Setup Tests

#### Test 1.1: Create IBKR Connection

**Steps:**
1. Launch WealthFolio desktop application
2. Navigate to **Settings** → **Broker Connections**
3. Click **"+ Add IBKR Connection"**
4. Fill in the form:
   - **Connection Name**: "Test IBKR Account"
   - **Flex Query ID**: Your Query ID from IBKR
   - **Flex Query Token**: Your 64-character token
   - **Linked Account**: Select an existing account or leave blank
   - **Sync Interval**: Manual
   - **Auto Import**: Disabled (for testing)
5. Click **"Create Connection"**

**Expected Results:**
- ✅ Success toast: "IBKR connection created successfully"
- ✅ Connection appears in list with "Never synced" status
- ✅ Connection is active (no "Disabled" badge)

**Error Cases to Test:**
- ❌ Empty name → Validation error
- ❌ Empty Query ID → Validation error
- ❌ Empty token → Validation error
- ❌ Invalid Query ID → API error on first sync
- ❌ Invalid token → API error on first sync

---

#### Test 1.2: Test Connection Credentials

**Steps:**
1. Locate your connection in the list
2. Click the three-dot menu → **Edit**
3. Click **"Test Connection"** button

**Expected Results:**
- ✅ Success toast: "Connection test successful. Your IBKR credentials are valid."
- ✅ No errors in console

**Error Cases:**
- ❌ Invalid token → "Failed to test broker connection" error
- ❌ Invalid Query ID → "Failed to test broker connection" error
- ❌ IBKR API down → Network error with retry message

---

#### Test 1.3: Edit Connection

**Steps:**
1. Click three-dot menu → **Edit**
2. Change connection name to "Updated Test Account"
3. Change sync interval to "Daily"
4. Enable "Auto Import"
5. Click **"Update Connection"**

**Expected Results:**
- ✅ Success toast: "Connection updated successfully"
- ✅ Changes reflected immediately in list
- ✅ Token field is empty (for security, not pre-filled)

---

#### Test 1.4: Toggle Connection Active/Inactive

**Steps:**
1. Click three-dot menu → **Disable**
2. Verify "Disabled" badge appears
3. Click three-dot menu → **Enable**
4. Verify "Disabled" badge disappears

**Expected Results:**
- ✅ Success toasts for enable/disable
- ✅ Badge updates immediately
- ✅ Sync button disabled when connection is disabled

---

#### Test 1.5: Delete Connection

**Steps:**
1. Click three-dot menu → **Delete**
2. Confirm deletion in dialog
3. Verify connection removed from list

**Expected Results:**
- ✅ Confirmation dialog appears
- ✅ Connection removed after confirmation
- ✅ Token removed from system keyring
- ✅ Success toast displayed

---

### Phase 2: Sync & Data Import Tests

#### Test 2.1: Manual Sync - First Sync

**Steps:**
1. Ensure connection is created and active
2. Click **"Sync Now"** button
3. Wait for sync to complete (may take 10-30 seconds)

**Expected Results:**
- ✅ Button shows "Syncing..." with spinner
- ✅ Success toast: "Sync completed successfully. Imported X activities."
- ✅ Last sync timestamp updates
- ✅ Status badge shows "Synced" (green)
- ✅ Activity count shown in toast

**Monitor:**
- Browser console for errors
- Network tab for API calls
- Backend logs for processing details

---

#### Test 2.2: Verify Imported Activities

**Steps:**
1. After successful sync, navigate to **Activities** page
2. Filter by the linked account
3. Verify activities are imported

**Expected Results:**
- ✅ Activities appear in list
- ✅ Activity types are correct:
  - Trades: BUY/SELL
  - Cash transactions: DIVIDEND, INTEREST, TAX, DEPOSIT, WITHDRAWAL, FEE
  - Corporate actions: SPLIT
- ✅ Dates formatted correctly
- ✅ Amounts are accurate
- ✅ Currency matches IBKR data
- ✅ Symbols are normalized correctly

**Data Validation Checks:**
- Trade quantities match IBKR
- Prices match IBKR
- Fees/commissions imported correctly
- Dividend amounts correct
- Tax withholding captured
- Stock splits reflected properly

---

#### Test 2.3: Duplicate Detection

**Steps:**
1. Perform a sync
2. Note the number of activities imported
3. Immediately sync again without new IBKR activities

**Expected Results:**
- ✅ Second sync completes successfully
- ✅ No duplicate activities created
- ✅ Activity count is 0 or minimal (only genuinely new activities)
- ✅ Existing activities not modified

---

#### Test 2.4: Auto-Import vs Manual Review

**Test Auto-Import (Enabled):**
1. Edit connection, enable "Auto Import"
2. Link to a specific account
3. Click "Sync Now"
4. Activities automatically imported to portfolio

**Expected Results:**
- ✅ Activities imported immediately
- ✅ No review step
- ✅ Success toast shows count

**Test Manual Review (Disabled):**
1. Edit connection, disable "Auto Import"
2. Click "Sync Now"
3. Should show preview of activities for review

**Expected Results:**
- ✅ Activities fetched but not imported
- ✅ Preview shown for review
- ✅ User can approve/reject before import

---

#### Test 2.5: Multi-Account Sync

If you have multiple IBKR accounts:

**Steps:**
1. Create second broker connection
2. Link to different WealthFolio account
3. Sync both connections
4. Verify activities go to correct accounts

**Expected Results:**
- ✅ Each connection syncs independently
- ✅ Activities imported to correct linked accounts
- ✅ No cross-contamination between accounts

---

### Phase 3: Error Handling Tests

#### Test 3.1: Invalid Credentials

**Steps:**
1. Create connection with invalid token
2. Click "Sync Now"

**Expected Results:**
- ❌ Error toast: "Failed to sync broker connection"
- ❌ Status badge shows "Error"
- ❌ Last sync error displayed in card
- ✅ App doesn't crash
- ✅ User can edit and fix credentials

---

#### Test 3.2: Network Failure

**Steps:**
1. Disconnect from internet
2. Click "Sync Now"

**Expected Results:**
- ❌ Error toast with network error message
- ❌ Status shows "Error"
- ✅ App remains responsive
- ✅ User can retry when network restored

---

#### Test 3.3: IBKR API Rate Limiting

**Steps:**
1. Perform multiple syncs rapidly (5+ within 1 minute)
2. Observe behavior

**Expected Results:**
- ✅ Retry logic handles rate limits
- ✅ Exponential backoff implemented
- ✅ Eventually succeeds or shows clear error

---

#### Test 3.4: Malformed IBKR Data

This requires manual testing with mock data:

**Test Cases:**
- Missing required fields (symbol, date, amount)
- Invalid date formats
- Non-numeric amounts
- Unknown activity types
- Missing currency

**Expected Results:**
- ✅ Parser handles gracefully
- ✅ Invalid records skipped with warning
- ✅ Valid records still imported
- ✅ Error details shown to user

---

### Phase 4: UI/UX Tests

#### Test 4.1: Settings Page Navigation

**Steps:**
1. From dashboard, navigate to Settings
2. Click "Broker Connections" in sidebar
3. Verify page loads correctly

**Expected Results:**
- ✅ Page loads without errors
- ✅ Sidebar item highlighted
- ✅ Proper breadcrumbs (if any)

---

#### Test 4.2: Empty State

**Steps:**
1. With no connections created
2. Navigate to Broker Connections

**Expected Results:**
- ✅ Empty state card displayed
- ✅ Cloud icon shown
- ✅ Helpful message: "No broker connections!"
- ✅ "Add IBKR Connection" button prominent

---

#### Test 4.3: Loading States

**Steps:**
1. Observe loading states during:
   - Page load
   - Connection creation
   - Sync operation
   - Testing credentials

**Expected Results:**
- ✅ Skeleton loaders shown during fetch
- ✅ Spinners on buttons during async operations
- ✅ Disabled state on buttons during operations
- ✅ No layout shift or flickering

---

#### Test 4.4: Form Validation

**Steps:**
1. Open "Add Connection" dialog
2. Try to submit with empty fields
3. Try to submit with invalid data

**Expected Results:**
- ✅ Inline validation messages
- ✅ Submit button disabled when invalid
- ✅ Clear error messages
- ✅ Form cannot submit until valid

---

#### Test 4.5: Responsive Design

**Steps:**
1. Test on desktop (1920px, 1366px)
2. Test on tablet (768px)
3. Test on mobile (375px, 414px)

**Expected Results:**
- ✅ Layout adapts to screen size
- ✅ Buttons remain accessible
- ✅ Dialogs/modals fit screen
- ✅ No horizontal scroll
- ✅ Touch targets adequate size

---

#### Test 4.6: Activity Import Integration

**Steps:**
1. Navigate to **Import** page
2. Observe "Sync from Interactive Brokers" card

**Expected Results:**
- ✅ Card visible below file upload
- ✅ Cloud icon displayed
- ✅ "Manage Connections" button present
- ✅ Clicking button navigates to Broker Connections page

---

### Phase 5: Security Tests

#### Test 5.1: Token Storage

**Steps:**
1. Create connection with token
2. Close app completely
3. Reopen app
4. Navigate to Broker Connections
5. Edit connection

**Expected Results:**
- ✅ Token field is empty (not displayed)
- ✅ Connection still works (token retrieved from keyring)
- ✅ Can update other fields without re-entering token
- ✅ Token not visible in DevTools/Network tab

---

#### Test 5.2: Token Update

**Steps:**
1. Edit connection
2. Enter new token
3. Save
4. Verify sync works with new token

**Expected Results:**
- ✅ Old token replaced in keyring
- ✅ Sync works with new token
- ✅ No errors about old token

---

#### Test 5.3: Connection Deletion

**Steps:**
1. Create connection
2. Delete connection
3. Check system keyring (if possible)

**Expected Results:**
- ✅ Token removed from keyring
- ✅ No orphaned credentials
- ✅ Connection fully cleaned up

---

## Automated Testing

### Unit Tests (Frontend)

Create test file: `src/hooks/use-broker-connections.test.ts`

```typescript
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useBrokerConnections } from './use-broker-connections';

describe('useBrokerConnections', () => {
  it('fetches broker connections', async () => {
    const queryClient = new QueryClient();
    const wrapper = ({ children }) => (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    );

    const { result } = renderHook(() => useBrokerConnections(), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.connections).toBeDefined();
  });
});
```

### Unit Tests (Backend)

Add to `src-core/src/integrations/ibkr/flex_query_parser.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trade_buy() {
        let parser = FlexQueryParser::new();
        let trade = Trade {
            symbol: "AAPL".to_string(),
            date_time: "2024-01-15;090000".to_string(),
            quantity: rust_decimal::Decimal::new(10, 0),
            trade_price: rust_decimal::Decimal::new(15000, 2), // 150.00
            proceeds: rust_decimal::Decimal::new(-150000, 2), // -1500.00
            comm_currency: rust_decimal::Decimal::new(-100, 2), // -1.00
            currency: "USD".to_string(),
            description: Some("APPLE INC".to_string()),
            asset_category: Some("STK".to_string()),
        };

        let result = parser.parse_trade(&trade);
        assert!(result.is_ok());

        let activity = result.unwrap();
        assert_eq!(activity.activity_type, "BUY");
        assert_eq!(activity.quantity, 10.0);
        assert_eq!(activity.unit_price, 150.0);
        assert_eq!(activity.fee, 1.0);
    }

    #[test]
    fn test_parse_trade_sell() {
        // Negative quantity = SELL
        let trade = Trade {
            quantity: rust_decimal::Decimal::new(-10, 0),
            // ... other fields
        };
        // Assert SELL activity created
    }

    #[test]
    fn test_parse_dividend() {
        let cash_transaction = CashTransaction {
            type_field: "Dividends".to_string(),
            symbol: Some("AAPL".to_string()),
            amount: rust_decimal::Decimal::new(2550, 2), // 25.50
            // ... other fields
        };

        let result = parser.parse_cash_transaction(&cash_transaction);
        assert!(result.is_ok());

        let activity = result.unwrap();
        assert_eq!(activity.activity_type, "DIVIDEND");
        assert_eq!(activity.quantity, 25.5);
    }

    #[test]
    fn test_invalid_date_format() {
        // Test error handling for malformed dates
    }

    #[test]
    fn test_missing_required_fields() {
        // Test error handling for missing data
    }
}
```

### Integration Tests

Add to `src-core/src/integrations/ibkr/mod.rs`:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_sync_workflow() {
        // This requires test IBKR credentials
        // Consider using mocks or test data
    }

    #[tokio::test]
    async fn test_sync_with_invalid_credentials() {
        // Test error handling
    }
}
```

---

## Test Scenarios & Cases

### Scenario 1: First-Time User Setup

**User Story:** As a new user, I want to connect my IBKR account and import my trading history.

**Steps:**
1. User opens WealthFolio for the first time
2. Completes onboarding
3. Creates account(s)
4. Navigates to Settings → Broker Connections
5. Clicks "Add IBKR Connection"
6. Follows IBKR setup guide to get credentials
7. Enters credentials and saves
8. Tests connection
9. Performs first sync
10. Reviews imported activities
11. Confirms import

**Success Criteria:**
- All activities imported correctly
- No duplicates
- Data matches IBKR portal
- User understands the process

---

### Scenario 2: Regular Sync Cadence

**User Story:** As an active trader, I want my activities to sync automatically daily.

**Steps:**
1. User sets sync interval to "Daily"
2. Enables auto-import
3. Links to account
4. Saves connection
5. Activities sync automatically each day

**Success Criteria:**
- New activities appear without manual intervention
- No duplicates created
- Sync errors are surfaced clearly
- User can monitor sync status

---

### Scenario 3: Multi-Account Management

**User Story:** As a user with multiple IBKR accounts, I want to track each separately.

**Steps:**
1. User creates first connection for Account A
2. Creates second connection for Account B
3. Links each to different WealthFolio accounts
4. Syncs both
5. Verifies activities in correct accounts

**Success Criteria:**
- Activities don't mix between accounts
- Each connection syncs independently
- Status tracked separately

---

### Scenario 4: Error Recovery

**User Story:** As a user, I want clear feedback when sync fails and ability to retry.

**Steps:**
1. Sync fails (network issue, invalid token, etc.)
2. User sees error message
3. User fixes issue (updates token, reconnects network)
4. User retries sync
5. Sync succeeds

**Success Criteria:**
- Error message is clear and actionable
- User can identify and fix issue
- Retry is straightforward
- No data loss or corruption

---

## Troubleshooting

### Common Issues During Testing

#### Issue: "Statement not ready" error
**Cause:** IBKR is still generating the Flex Query report
**Solution:**
- Wait 30 seconds and retry
- Check query complexity (too many sections can slow generation)
- Verify date range isn't excessive

#### Issue: No activities imported
**Cause:** Various reasons
**Solutions:**
- Check date range covers period with actual trades
- Verify all required sections added to Flex Query
- Confirm XML format selected (not CSV)
- Check "Include Column Headers" is enabled

#### Issue: Wrong activity amounts
**Cause:** Data mapping or currency issues
**Solutions:**
- Verify currency field in Flex Query
- Check commission/fee calculation
- Review trade_money vs quantity * price calculation
- Examine IBKR data directly to compare

#### Issue: Keyring access errors
**Cause:** System keyring not available or permissions issue
**Solutions:**
- On Linux: Install libsecret
- On macOS: Grant Keychain access
- On Windows: No action usually needed
- Check application has necessary permissions

#### Issue: Duplicate activities
**Cause:** Multiple syncs without proper duplicate detection
**Solutions:**
- Check activity ID generation logic
- Verify duplicate detection using date+symbol+amount
- Review last_reference_code tracking
- May need to manually remove duplicates

---

## Test Data Requirements

For comprehensive testing, ensure your IBKR account has:

- ✅ **Buy trades** (at least 3)
- ✅ **Sell trades** (at least 2)
- ✅ **Dividends** (at least 1)
- ✅ **Interest payments**
- ✅ **Deposits/Withdrawals**
- ✅ **Tax withholding**
- ✅ **Fees/Commissions**
- ✅ **Stock splits** (if available)
- ✅ **Multiple currencies** (if applicable)
- ✅ **Different asset types** (stocks, ETFs, options if supported)

If your account lacks certain types, consider:
- Using IBKR paper trading account for testing
- Creating test data in a separate environment
- Coordinating with users who have diverse activity history

---

## Test Checklist

Before marking the feature as production-ready:

### Backend
- [ ] Unit tests for FlexQueryClient
- [ ] Unit tests for FlexQueryParser
- [ ] Unit tests for IBKRSyncService
- [ ] Integration test for full sync workflow
- [ ] Error handling tests for all edge cases
- [ ] Performance test with large datasets (1000+ activities)

### Frontend
- [ ] Unit tests for hooks
- [ ] Unit tests for mutations
- [ ] Component tests for forms
- [ ] Component tests for connection cards
- [ ] E2E test for connection creation
- [ ] E2E test for sync operation

### Manual Testing
- [ ] All manual test procedures completed
- [ ] All test scenarios passed
- [ ] Tested on all supported platforms (Windows, macOS, Linux)
- [ ] Tested on various screen sizes
- [ ] Security review completed
- [ ] Performance acceptable on low-end hardware

### Documentation
- [ ] IBKR setup guide reviewed and accurate
- [ ] Testing guide complete
- [ ] Troubleshooting section comprehensive
- [ ] User-facing help text clear

---

## Reporting Issues

When reporting test failures or bugs:

1. **Environment**: OS, WealthFolio version, IBKR account type
2. **Steps to Reproduce**: Exact sequence of actions
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happened
5. **Logs**: Console output, backend logs, error messages
6. **Screenshots**: If UI-related
7. **IBKR Data**: Sample of the XML/data causing issues (sanitize sensitive info)

File issues at: https://github.com/ni3do/wealthfolio/issues

---

## Success Criteria

The IBKR integration is considered production-ready when:

- ✅ All manual test procedures pass
- ✅ Automated test coverage >80% for new code
- ✅ No critical or high-severity bugs
- ✅ Documentation complete and accurate
- ✅ Performance meets requirements (sync <30 seconds for typical dataset)
- ✅ Security review passed
- ✅ User testing feedback incorporated
- ✅ Works on all supported platforms

---

## Additional Resources

- [IBKR Flex Query Setup Guide](./IBKR_FLEX_QUERY_SETUP.md)
- [IBKR Flex Queries Documentation](https://www.interactivebrokers.com/en/software/am/am/reports/activityflexqueries.htm)
- [WealthFolio GitHub Repository](https://github.com/ni3do/wealthfolio)
- [Vitest Documentation](https://vitest.dev/)
- [Playwright Documentation](https://playwright.dev/)

---

**Last Updated**: January 2026
**Version**: 1.0
**Status**: Complete implementation, ready for testing
