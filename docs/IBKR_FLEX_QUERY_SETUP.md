# Interactive Brokers Flex Query Setup Guide

This guide will walk you through setting up a Flex Query in your Interactive Brokers account so you can automatically sync your trading activities with Wealthfolio.

## What is a Flex Query?

A Flex Query is Interactive Brokers' customizable reporting tool that allows you to retrieve account data programmatically via API. This enables automatic synchronization of your trades, dividends, deposits, and other activities without manual CSV downloads.

## Prerequisites

- Active Interactive Brokers account
- Access to IBKR Client Portal (https://www.interactivebrokers.com)

---

## Step 1: Log into Client Portal

1. Go to [IBKR Client Portal](https://www.interactivebrokers.com/sso/Login)
2. Enter your username and password
3. Complete two-factor authentication if enabled

---

## Step 2: Navigate to Flex Queries

1. From the main menu, hover over **Performance & Reports**
2. Click on **Flex Queries**

   ![Navigate to Flex Queries](images/ibkr-flex-menu.png)

---

## Step 3: Create a New Flex Query

1. Click the **"+"** button or **"Create"** to start a new Flex Query
2. Select **Activity Flex Query** (not Trade Confirmation)
3. Give your query a descriptive name:
   - Example: "Wealthfolio Sync"
   - Example: "Daily Activity Report"

   ![Create Flex Query](images/ibkr-create-query.png)

---

## Step 4: Configure Query Sections

You need to include specific sections to capture all your trading activities. Add the following sections:

### Required Sections:

#### A. **Trades** Section
- Click **Add Section** → Select **Trades**
- Include these fields:
  - Account ID
  - Symbol
  - Date/Time
  - Quantity
  - Trade Price
  - Trade Money
  - IB Commission
  - Currency
  - Description (optional but recommended)
  - Asset Category

#### B. **Cash Transactions** Section
- Click **Add Section** → Select **Cash Transactions**
- Include these fields:
  - Account ID
  - Type (Dividends, Deposits, Withdrawals, etc.)
  - Symbol
  - Date/Time
  - Amount
  - Currency
  - Description (optional but recommended)

#### C. **Corporate Actions** Section *(Optional but recommended)*
- Click **Add Section** → Select **Corporate Actions**
- Include these fields:
  - Account ID
  - Symbol
  - Date/Time
  - Type
  - Quantity
  - Currency
  - Description

---

## Step 5: Configure Date Range

Set how far back you want to retrieve data:

**Option 1: Fixed Period** (Recommended for initial sync)
- Select **Fixed Period**
- Choose: Last 365 days (or as needed)

**Option 2: Specific Date Range**
- Select **Between** and specify start/end dates
- Useful for one-time historical imports

**Option 3: Dynamic Period** (Recommended for recurring syncs)
- Select **Last 7 days** or **Last 30 days**
- Query updates automatically each time you sync

---

## Step 6: Configure Output Format

**IMPORTANT:** Select **XML** format (not CSV)

- Format: **XML**
- Include Column Headers: **Yes**
- Date Format: **YYYY-MM-DD**

Wealthfolio currently supports XML format for best compatibility and data accuracy.

---

## Step 7: Save the Flex Query

1. Click **Save** or **Create**
2. Your query will be assigned a **Query ID**
3. **IMPORTANT:** Copy and save this Query ID - you'll need it later!
   - Example format: `123456`

   ![Query ID](images/ibkr-query-id.png)

---

## Step 8: Activate Flex Web Service

To enable API access, you must activate the Flex Web Service:

1. Go back to **Performance & Reports** → **Flex Queries**
2. Look for **"Flex Web Service"** section at the bottom
3. Check the box: **"Activate Flex Web Service"**
4. Click **Configure** or **Generate Token**

   ![Activate Flex Web Service](images/ibkr-activate-service.png)

5. A **Token** will be generated for you
6. **CRITICAL:** Copy this token immediately and store it securely!
   - Format: 64-character alphanumeric string
   - Example: `1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef`
   - This token is shown only once - if you lose it, you'll need to regenerate

---

## Step 9: Configure in Wealthfolio

Now that you have your Query ID and Token, add them to Wealthfolio:

1. Open Wealthfolio
2. Go to **Settings** → **Broker Connections**
3. Click **"+ Add Connection"**
4. Select **Interactive Brokers**
5. Enter the following:
   - **Connection Name**: "My IBKR Account" (or any name you prefer)
   - **IBKR Token**: Paste the 64-character token
   - **Query ID**: Enter your Flex Query ID
   - **Linked Account**: Select which Wealthfolio account to sync to (optional)
   - **Sync Interval**: Choose Manual, Daily, or Weekly
   - **Auto-import**: Enable if you want activities imported automatically (recommended: leave off for first sync)

6. Click **Test Connection** to verify your credentials
7. Click **Save**

---

## Step 10: Perform First Sync

1. In Wealthfolio, go to **Broker Connections**
2. Find your IBKR connection
3. Click **Sync Now**
4. Review the imported activities
5. Verify everything looks correct
6. Click **Import** to add them to your portfolio

---

## Troubleshooting

### Error: "Invalid Token"
- Token may have been regenerated or expired
- Go back to IBKR Client Portal → Flex Queries → Flex Web Service
- Generate a new token and update it in Wealthfolio

### Error: "Invalid Query ID"
- Double-check the Query ID in IBKR Client Portal
- Ensure you copied the correct ID (not the query name)

### Error: "Statement not ready"
- IBKR is still generating the report
- Wait 30 seconds and try again
- If it persists, check that your query isn't too complex

### No activities found
- Check your date range in the Flex Query settings
- Ensure you have actual activities in that period
- Verify all required sections (Trades, Cash Transactions) are included

### Activities are imported but amounts are wrong
- Verify currency settings in your query
- Check that "Include Column Headers" is enabled
- Ensure format is set to XML

---

## Best Practices

### Security
- **Never share your Flex Web Service token** - it provides full read access to your account data
- Store it securely in Wealthfolio's encrypted credential storage
- If compromised, regenerate immediately in IBKR Client Portal

### Sync Frequency
- **First time**: Use a fixed period (e.g., last 365 days) to import historical data
- **Ongoing**: Switch to "Last 30 days" or "Last 7 days" for automatic syncs
- **Manual sync**: Best for infrequent traders or when you want full control

### Data Management
- **Enable draft mode** for first sync to review activities before final import
- **Check for duplicates** if you've already manually imported some activities
- **Verify symbols** - IBKR sometimes uses different tickers than other providers

---

## Advanced Configuration

### Multi-Account Setup
If you have multiple IBKR accounts:

1. Create separate Flex Queries for each account
2. Each query gets its own Query ID
3. Add multiple broker connections in Wealthfolio, one per account
4. Link each connection to the corresponding Wealthfolio account

### Custom Date Ranges
For tax reporting or specific analysis:

1. Create a dedicated Flex Query with fixed dates
2. Example: "2024 Tax Year" (Jan 1 - Dec 31, 2024)
3. Sync once to get that specific period
4. Don't use for recurring syncs

---

## FAQ

**Q: How often should I sync?**
A: For active traders, daily or weekly. For buy-and-hold investors, manual syncs monthly or quarterly.

**Q: Will this download my entire history?**
A: Only the date range you specify in the Flex Query settings.

**Q: Can I use the same query for multiple accounts?**
A: Yes, if your query doesn't filter by account. However, it's cleaner to have separate queries per account.

**Q: What if I already imported activities manually?**
A: Wealthfolio will detect duplicates. Review carefully before importing to avoid double-counting.

**Q: Is my data secure?**
A: Yes. Your token is stored encrypted in your system's keyring. The API is read-only and cannot execute trades.

**Q: Does this work with IBKR Lite?**
A: Yes! Flex Queries work with all IBKR account types.

---

## Need Help?

- **IBKR Support**: [Contact Interactive Brokers](https://www.interactivebrokers.com/en/support/contact.php)
- **Flex Query Documentation**: [IBKR Flex Queries Guide](https://www.interactivebrokers.com/en/software/am/am/reports/activityflexqueries.htm)
- **Wealthfolio Issues**: [GitHub Issues](https://github.com/afadil/wealthfolio/issues)

---

## Summary Checklist

- [ ] Log into IBKR Client Portal
- [ ] Create new Activity Flex Query
- [ ] Add Trades, Cash Transactions, and Corporate Actions sections
- [ ] Configure date range (Last 365 days for first sync)
- [ ] Set output format to XML
- [ ] Save query and copy Query ID
- [ ] Activate Flex Web Service
- [ ] Generate and securely save Token
- [ ] Add connection in Wealthfolio with Token and Query ID
- [ ] Test connection
- [ ] Perform first sync and review activities
- [ ] Import activities and verify portfolio data

**You're all set!** Your IBKR account is now connected and ready to sync automatically. 🎉
