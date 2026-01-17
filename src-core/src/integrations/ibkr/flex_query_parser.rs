use crate::activities::ActivityImport;
use crate::integrations::ibkr::StatementResponse;
use crate::Result;
use chrono::NaiveDate;
use log::{debug, warn};
use quick_xml::de::from_str;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// IBKR Flex Query XML response structure
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FlexQueryResponse {
    #[serde(rename = "FlexStatements")]
    pub flex_statements: FlexStatements,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FlexStatements {
    #[serde(rename = "FlexStatement")]
    pub flex_statement: FlexStatement,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FlexStatement {
    #[serde(default)]
    pub trades: Option<Trades>,
    #[serde(default)]
    pub cash_transactions: Option<CashTransactions>,
    #[serde(default)]
    pub corporate_actions: Option<CorporateActions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Trades {
    #[serde(rename = "Trade", default)]
    pub trades: Vec<Trade>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CashTransactions {
    #[serde(rename = "CashTransaction", default)]
    pub transactions: Vec<CashTransaction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorporateActions {
    #[serde(rename = "CorporateAction", default)]
    pub actions: Vec<CorporateAction>,
}

/// IBKR Trade entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub symbol: String,
    #[serde(rename = "dateTime")]
    pub date_time: String,
    pub quantity: String,
    #[serde(rename = "tradePrice")]
    pub trade_price: String,
    #[serde(rename = "tradeMoney")]
    pub trade_money: String,
    #[serde(rename = "ibCommission")]
    pub ib_commission: String,
    pub currency: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "assetCategory", default)]
    pub asset_category: Option<String>,
}

/// IBKR Cash Transaction entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CashTransaction {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub symbol: Option<String>,
    #[serde(rename = "dateTime")]
    pub date_time: String,
    pub amount: String,
    pub currency: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// IBKR Corporate Action entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateAction {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub symbol: String,
    #[serde(rename = "dateTime")]
    pub date_time: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub quantity: Option<String>,
    pub currency: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Parser for IBKR Flex Query responses
pub struct FlexQueryParser {
    /// Whether to import activities as drafts for review
    pub import_as_draft: bool,
}

impl FlexQueryParser {
    pub fn new() -> Self {
        Self {
            import_as_draft: true,
        }
    }

    pub fn with_draft_mode(mut self, draft: bool) -> Self {
        self.import_as_draft = draft;
        self
    }

    /// Parse IBKR Flex Query response into ActivityImport objects
    pub fn parse(&self, response: &StatementResponse) -> Result<Vec<ActivityImport>> {
        let xml = match response {
            StatementResponse::Xml(content) => content,
            StatementResponse::Csv(_) => {
                return Err(crate::Error::InvalidInput(
                    "CSV format not yet supported. Please use XML format.".to_string(),
                ));
            }
            StatementResponse::Error(msg) => {
                return Err(crate::Error::InvalidInput(format!(
                    "Cannot parse error response: {}",
                    msg
                )));
            }
        };

        debug!("Parsing IBKR Flex Query XML response");

        let flex_response: FlexQueryResponse = from_str(xml).map_err(|e| {
            crate::Error::ParseError(format!("Failed to parse IBKR XML: {}", e))
        })?;

        let mut activities = Vec::new();
        let statement = &flex_response.flex_statements.flex_statement;

        // Parse trades
        if let Some(trades) = &statement.trades {
            for trade in &trades.trades {
                if let Some(activity) = self.parse_trade(trade) {
                    activities.push(activity);
                }
            }
        }

        // Parse cash transactions
        if let Some(cash_txns) = &statement.cash_transactions {
            for txn in &cash_txns.transactions {
                if let Some(activity) = self.parse_cash_transaction(txn) {
                    activities.push(activity);
                }
            }
        }

        // Parse corporate actions
        if let Some(corp_actions) = &statement.corporate_actions {
            for action in &corp_actions.actions {
                if let Some(activity) = self.parse_corporate_action(action) {
                    activities.push(activity);
                }
            }
        }

        debug!("Parsed {} activities from IBKR response", activities.len());
        Ok(activities)
    }

    /// Parse an IBKR trade into an ActivityImport
    fn parse_trade(&self, trade: &Trade) -> Option<ActivityImport> {
        let quantity = Decimal::from_str(&trade.quantity).ok()?;
        let trade_price = Decimal::from_str(&trade.trade_price).ok()?;
        let commission = Decimal::from_str(&trade.ib_commission)
            .ok()
            .unwrap_or(Decimal::ZERO)
            .abs();

        // Determine activity type (BUY or SELL)
        let activity_type = if quantity > Decimal::ZERO {
            "BUY"
        } else {
            "SELL"
        };

        // Parse date
        let date = self.parse_ibkr_date(&trade.date_time)?;

        Some(ActivityImport {
            id: None,
            date,
            symbol: self.normalize_symbol(&trade.symbol),
            activity_type: activity_type.to_string(),
            quantity: quantity.abs(),
            unit_price: trade_price.abs(),
            currency: trade.currency.clone(),
            fee: commission,
            amount: None, // Will be calculated
            comment: trade.description.clone(),
            account_id: None,
            account_name: Some(trade.account_id.clone()),
            symbol_name: None,
            errors: None,
            is_draft: self.import_as_draft,
            is_valid: true,
            line_number: None,
        })
    }

    /// Parse an IBKR cash transaction into an ActivityImport
    fn parse_cash_transaction(&self, txn: &CashTransaction) -> Option<ActivityImport> {
        let amount = Decimal::from_str(&txn.amount).ok()?;

        // Map IBKR transaction type to our activity type
        let activity_type = self.map_cash_transaction_type(&txn.transaction_type)?;

        // Parse date
        let date = self.parse_ibkr_date(&txn.date_time)?;

        // Determine symbol (use symbol if provided, otherwise $CASH-{CURRENCY})
        let symbol = txn
            .symbol
            .clone()
            .unwrap_or_else(|| format!("$CASH-{}", txn.currency));

        Some(ActivityImport {
            id: None,
            date,
            symbol: self.normalize_symbol(&symbol),
            activity_type: activity_type.to_string(),
            quantity: Decimal::ONE,
            unit_price: amount.abs(),
            currency: txn.currency.clone(),
            fee: Decimal::ZERO,
            amount: Some(amount.abs()),
            comment: txn.description.clone(),
            account_id: None,
            account_name: Some(txn.account_id.clone()),
            symbol_name: None,
            errors: None,
            is_draft: self.import_as_draft,
            is_valid: true,
            line_number: None,
        })
    }

    /// Parse an IBKR corporate action into an ActivityImport
    fn parse_corporate_action(&self, action: &CorporateAction) -> Option<ActivityImport> {
        // Map IBKR action type to our activity type
        let activity_type = self.map_corporate_action_type(&action.action_type)?;

        // Parse date
        let date = self.parse_ibkr_date(&action.date_time)?;

        let quantity = action
            .quantity
            .as_ref()
            .and_then(|q| Decimal::from_str(q).ok())
            .unwrap_or(Decimal::ZERO);

        Some(ActivityImport {
            id: None,
            date,
            symbol: self.normalize_symbol(&action.symbol),
            activity_type: activity_type.to_string(),
            quantity: quantity.abs(),
            unit_price: Decimal::ZERO,
            currency: action.currency.clone(),
            fee: Decimal::ZERO,
            amount: Some(Decimal::ZERO),
            comment: action.description.clone(),
            account_id: None,
            account_name: Some(action.account_id.clone()),
            symbol_name: None,
            errors: None,
            is_draft: self.import_as_draft,
            is_valid: true,
            line_number: None,
        })
    }

    /// Map IBKR cash transaction type to our activity type
    fn map_cash_transaction_type(&self, ibkr_type: &str) -> Option<&'static str> {
        match ibkr_type {
            "Dividends" => Some("DIVIDEND"),
            "Payment In Lieu Of Dividends" => Some("DIVIDEND"),
            "Withholding Tax" => Some("TAX"),
            "Broker Interest Paid" => Some("INTEREST"),
            "Broker Interest Received" => Some("INTEREST"),
            "Deposits" | "Deposits/Withdrawals" => Some("DEPOSIT"),
            "Withdrawals" => Some("WITHDRAWAL"),
            "Other Fees" => Some("FEE"),
            "Commission Adjustments" => Some("FEE"),
            _ => {
                warn!("Unknown IBKR cash transaction type: {}", ibkr_type);
                None
            }
        }
    }

    /// Map IBKR corporate action type to our activity type
    fn map_corporate_action_type(&self, ibkr_type: &str) -> Option<&'static str> {
        match ibkr_type {
            "TC" | "FS" | "SO" => Some("SPLIT"), // Stock split
            "DW" => Some("DIVIDEND"),             // Dividend in shares
            _ => {
                warn!("Unknown IBKR corporate action type: {}", ibkr_type);
                None
            }
        }
    }

    /// Parse IBKR date format (YYYY-MM-DD or YYYY-MM-DD;HHMMSS)
    fn parse_ibkr_date(&self, date_str: &str) -> Option<String> {
        // IBKR formats: "2024-01-15" or "2024-01-15;123000"
        let date_part = date_str.split(';').next()?;

        // Validate date format
        NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()?;

        Some(date_part.to_string())
    }

    /// Normalize symbol (handle special cases)
    fn normalize_symbol(&self, symbol: &str) -> String {
        // Handle cash symbols
        if symbol.starts_with("$CASH-") {
            return symbol.to_string();
        }

        // Remove any whitespace
        let normalized = symbol.trim().to_uppercase();

        // IBKR sometimes uses different symbols, map common ones
        match normalized.as_str() {
            // Add symbol mappings as needed
            _ => normalized,
        }
    }
}

impl Default for FlexQueryParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trade() {
        let parser = FlexQueryParser::new();
        let trade = Trade {
            account_id: "U1234567".to_string(),
            symbol: "AAPL".to_string(),
            date_time: "2024-01-15;123000".to_string(),
            quantity: "100".to_string(),
            trade_price: "150.00".to_string(),
            trade_money: "15000.00".to_string(),
            ib_commission: "-1.00".to_string(),
            currency: "USD".to_string(),
            description: Some("Buy AAPL".to_string()),
            asset_category: Some("STK".to_string()),
        };

        let activity = parser.parse_trade(&trade).unwrap();
        assert_eq!(activity.activity_type, "BUY");
        assert_eq!(activity.symbol, "AAPL");
        assert_eq!(activity.quantity, Decimal::from(100));
        assert_eq!(activity.unit_price, Decimal::from(150));
        assert_eq!(activity.fee, Decimal::from(1));
    }

    #[test]
    fn test_parse_dividend() {
        let parser = FlexQueryParser::new();
        let txn = CashTransaction {
            account_id: "U1234567".to_string(),
            transaction_type: "Dividends".to_string(),
            symbol: Some("MSFT".to_string()),
            date_time: "2024-01-20".to_string(),
            amount: "23.00".to_string(),
            currency: "USD".to_string(),
            description: Some("MSFT Dividend".to_string()),
        };

        let activity = parser.parse_cash_transaction(&txn).unwrap();
        assert_eq!(activity.activity_type, "DIVIDEND");
        assert_eq!(activity.symbol, "MSFT");
        assert_eq!(activity.amount, Some(Decimal::from(23)));
    }

    #[test]
    fn test_normalize_symbol() {
        let parser = FlexQueryParser::new();
        assert_eq!(parser.normalize_symbol("aapl"), "AAPL");
        assert_eq!(parser.normalize_symbol("  msft  "), "MSFT");
        assert_eq!(parser.normalize_symbol("$CASH-USD"), "$CASH-USD");
    }

    #[test]
    fn test_parse_ibkr_date() {
        let parser = FlexQueryParser::new();
        assert_eq!(
            parser.parse_ibkr_date("2024-01-15"),
            Some("2024-01-15".to_string())
        );
        assert_eq!(
            parser.parse_ibkr_date("2024-01-15;123000"),
            Some("2024-01-15".to_string())
        );
    }
}
