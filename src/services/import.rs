//! CSV Import service
//!
//! Provides functionality for importing transactions from CSV files,
//! including column mapping, date parsing, duplicate detection, and batch import.

use std::collections::HashMap;

use chrono::NaiveDate;

use crate::error::EnvelopeResult;
use crate::models::{AccountId, CategoryId, Money, TransactionStatus};
use crate::services::TransactionService;
use crate::storage::Storage;
use csv::{Reader, StringRecord};

/// Column mapping configuration for CSV import
#[derive(Debug, Clone)]
pub struct ColumnMapping {
    /// Index of the date column
    pub date_column: usize,
    /// Index of the amount column (or separate inflow/outflow columns)
    pub amount_column: Option<usize>,
    /// Index of the outflow column (if using separate columns)
    pub outflow_column: Option<usize>,
    /// Index of the inflow column (if using separate columns)
    pub inflow_column: Option<usize>,
    /// Index of the payee/description column
    pub payee_column: Option<usize>,
    /// Index of the memo/notes column
    pub memo_column: Option<usize>,
    /// Date format string (e.g., "%Y-%m-%d", "%m/%d/%Y")
    pub date_format: String,
    /// Whether the first row is a header
    pub has_header: bool,
    /// Delimiter character
    pub delimiter: char,
    /// Whether to invert amounts (some banks use positive for debits)
    pub invert_amounts: bool,
}

impl Default for ColumnMapping {
    fn default() -> Self {
        Self {
            date_column: 0,
            amount_column: Some(1),
            outflow_column: None,
            inflow_column: None,
            payee_column: Some(2),
            memo_column: None,
            date_format: "%Y-%m-%d".to_string(),
            has_header: true,
            delimiter: ',',
            invert_amounts: false,
        }
    }
}

impl ColumnMapping {
    /// Create a new column mapping
    pub fn new() -> Self {
        Self::default()
    }

    /// Common mapping for bank CSV exports (date, description, amount)
    pub fn simple_bank() -> Self {
        Self {
            date_column: 0,
            amount_column: Some(2),
            outflow_column: None,
            inflow_column: None,
            payee_column: Some(1),
            memo_column: None,
            date_format: "%m/%d/%Y".to_string(),
            has_header: true,
            delimiter: ',',
            invert_amounts: false,
        }
    }

    /// Common mapping for credit card CSV exports
    pub fn credit_card() -> Self {
        Self {
            date_column: 0,
            amount_column: Some(2),
            outflow_column: None,
            inflow_column: None,
            payee_column: Some(1),
            memo_column: Some(3),
            date_format: "%m/%d/%Y".to_string(),
            has_header: true,
            delimiter: ',',
            invert_amounts: true, // Credit cards often show positive for purchases
        }
    }

    /// Mapping for separate inflow/outflow columns
    pub fn separate_inout(
        date_col: usize,
        outflow_col: usize,
        inflow_col: usize,
        payee_col: usize,
    ) -> Self {
        Self {
            date_column: date_col,
            amount_column: None,
            outflow_column: Some(outflow_col),
            inflow_column: Some(inflow_col),
            payee_column: Some(payee_col),
            memo_column: None,
            date_format: "%Y-%m-%d".to_string(),
            has_header: true,
            delimiter: ',',
            invert_amounts: false,
        }
    }

    /// TD Bank CSV format (no header, date/description/debit/credit/balance)
    pub fn td_bank() -> Self {
        Self {
            date_column: 0,
            amount_column: None,
            outflow_column: Some(2),
            inflow_column: Some(3),
            payee_column: Some(1),
            memo_column: None,
            date_format: "%Y-%m-%d".to_string(),
            has_header: false,
            delimiter: ',',
            invert_amounts: false,
        }
    }

    /// Set the date format
    pub fn with_date_format(mut self, format: &str) -> Self {
        self.date_format = format.to_string();
        self
    }

    /// Set whether first row is header
    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    /// Set the delimiter
    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }
}

/// A parsed row from the CSV before import
#[derive(Debug, Clone)]
pub struct ParsedTransaction {
    /// Transaction date
    pub date: NaiveDate,
    /// Amount (negative for outflow)
    pub amount: Money,
    /// Payee/description
    pub payee: String,
    /// Memo/notes
    pub memo: String,
    /// Original row number in CSV (0-indexed, excluding header)
    pub row_number: usize,
    /// Generated import ID for duplicate detection
    pub import_id: String,
}

impl ParsedTransaction {
    /// Generate an import ID based on the transaction data
    pub fn generate_import_id(date: NaiveDate, amount: Money, payee: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        date.hash(&mut hasher);
        amount.cents().hash(&mut hasher);
        payee.hash(&mut hasher);
        format!("imp-{:016x}", hasher.finish())
    }
}

/// Status of a transaction for import preview
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportStatus {
    /// Transaction will be imported
    New,
    /// Transaction is a duplicate and will be skipped
    Duplicate,
    /// Transaction has an error and cannot be imported
    Error(String),
}

/// Preview entry for import review
#[derive(Debug, Clone)]
pub struct ImportPreviewEntry {
    /// The parsed transaction
    pub transaction: ParsedTransaction,
    /// Import status
    pub status: ImportStatus,
    /// Matching existing transaction ID (for duplicates)
    pub existing_id: Option<String>,
}

/// Result of a completed import
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Number of transactions imported
    pub imported: usize,
    /// Number of duplicates skipped
    pub duplicates_skipped: usize,
    /// Number of rows with errors
    pub errors: usize,
    /// IDs of imported transactions
    pub imported_ids: Vec<String>,
    /// Error messages by row
    pub error_messages: HashMap<usize, String>,
}

/// Service for CSV import
pub struct ImportService<'a> {
    storage: &'a Storage,
}

impl<'a> ImportService<'a> {
    /// Create a new import service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Parse a CSV from a reader into transactions
    pub fn parse_csv_from_reader<R: std::io::Read>(
        &self,
        reader: &mut Reader<R>,
        mapping: &ColumnMapping,
    ) -> EnvelopeResult<Vec<Result<ParsedTransaction, String>>> {
        let mut results = Vec::new();
        for (idx, result) in reader.records().enumerate() {
            let record = match result {
                Ok(record) => record,
                Err(e) => {
                    results.push(Err(format!("Error reading CSV record: {}", e)));
                    continue;
                }
            };
            let result = self.parse_record(&record, idx, mapping);
            results.push(result);
        }
        Ok(results)
    }

    /// Parse a single CSV record
    fn parse_record(
        &self,
        record: &StringRecord,
        row_number: usize,
        mapping: &ColumnMapping,
    ) -> Result<ParsedTransaction, String> {
        // Parse date
        let date_str = record
            .get(mapping.date_column)
            .ok_or_else(|| "Missing date column".to_string())?
            .trim();

        let date = self.parse_date(date_str, &mapping.date_format)?;

        // Parse amount
        let amount = self.parse_amount_from_record(record, mapping)?;

        // Parse payee
        let payee = mapping
            .payee_column
            .and_then(|col| record.get(col))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        // Parse memo
        let memo = mapping
            .memo_column
            .and_then(|col| record.get(col))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        // Generate import ID
        let import_id = ParsedTransaction::generate_import_id(date, amount, &payee);

        Ok(ParsedTransaction {
            date,
            amount,
            payee,
            memo,
            row_number,
            import_id,
        })
    }

    /// Parse amount from a record
    fn parse_amount_from_record(
        &self,
        record: &StringRecord,
        mapping: &ColumnMapping,
    ) -> Result<Money, String> {
        let amount = if let Some(amount_col) = mapping.amount_column {
            // Single amount column
            let amount_str = record
                .get(amount_col)
                .ok_or_else(|| "Missing amount column".to_string())?
                .trim();

            self.parse_amount_string(amount_str)?
        } else {
            // Separate inflow/outflow columns
            let outflow_col = mapping
                .outflow_column
                .ok_or_else(|| "Missing outflow column configuration".to_string())?;
            let inflow_col = mapping
                .inflow_column
                .ok_or_else(|| "Missing inflow column configuration".to_string())?;

            let outflow_str = record.get(outflow_col).map(|s| s.trim()).unwrap_or("");
            let inflow_str = record.get(inflow_col).map(|s| s.trim()).unwrap_or("");

            let outflow = if outflow_str.is_empty() {
                Money::zero()
            } else {
                -self.parse_amount_string(outflow_str)?.abs()
            };

            let inflow = if inflow_str.is_empty() {
                Money::zero()
            } else {
                self.parse_amount_string(inflow_str)?.abs()
            };

            outflow + inflow
        };

        if mapping.invert_amounts {
            Ok(-amount)
        } else {
            Ok(amount)
        }
    }

    /// Parse a date string using multiple format attempts
    fn parse_date(&self, s: &str, primary_format: &str) -> Result<NaiveDate, String> {
        // Try primary format first
        if let Ok(date) = NaiveDate::parse_from_str(s, primary_format) {
            return Ok(date);
        }

        // Try common alternative formats
        let formats = [
            "%Y-%m-%d", "%m/%d/%Y", "%m/%d/%y", "%d/%m/%Y", "%d/%m/%y", "%Y/%m/%d", "%m-%d-%Y",
            "%d-%m-%Y",
        ];

        for format in formats {
            if let Ok(date) = NaiveDate::parse_from_str(s, format) {
                return Ok(date);
            }
        }

        Err(format!("Could not parse date: '{}'", s))
    }

    /// Check if a record looks like data (not headers)
    /// Returns true if first column parses as a date
    fn looks_like_data_row(&self, record: &StringRecord) -> bool {
        if let Some(first) = record.get(0) {
            let first = first.trim();
            // Try to parse as a date - if it succeeds, this is data not a header
            let date_formats = [
                "%Y-%m-%d", "%m/%d/%Y", "%m/%d/%y", "%d/%m/%Y", "%d/%m/%y",
            ];
            for format in date_formats {
                if NaiveDate::parse_from_str(first, format).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    /// Detect column mapping from CSV header record
    pub fn detect_mapping_from_headers(&self, headers: &StringRecord) -> ColumnMapping {
        // First, check if this looks like a data row (no headers)
        if self.looks_like_data_row(headers) {
            // This is likely a headerless CSV like TD Bank
            // Check if it matches TD Bank format: date, desc, debit, credit, balance
            if headers.len() >= 4 {
                // Verify column 2 or 3 looks like a number (debit/credit)
                let col2 = headers.get(2).map(|s| s.trim()).unwrap_or("");
                let col3 = headers.get(3).map(|s| s.trim()).unwrap_or("");
                let col2_is_num = col2.is_empty() || col2.parse::<f64>().is_ok();
                let col3_is_num = col3.is_empty() || col3.parse::<f64>().is_ok();

                if col2_is_num && col3_is_num {
                    return ColumnMapping::td_bank();
                }
            }
        }

        let mut mapping = ColumnMapping::new();

        for (idx, header) in headers.iter().enumerate() {
            let h = header.to_lowercase();
            let h = h.trim();

            if h.contains("date") || h.contains("posted") || h.contains("trans") {
                mapping.date_column = idx;
            } else if h.contains("amount") && mapping.amount_column.is_none() {
                mapping.amount_column = Some(idx);
            } else if h.contains("debit") || h.contains("outflow") || h.contains("withdrawal") {
                mapping.outflow_column = Some(idx);
            } else if h.contains("credit") || h.contains("inflow") || h.contains("deposit") {
                mapping.inflow_column = Some(idx);
            } else if h.contains("description")
                || h.contains("payee")
                || h.contains("merchant")
                || h.contains("name")
            {
                mapping.payee_column = Some(idx);
            } else if h.contains("memo") || h.contains("note") {
                mapping.memo_column = Some(idx);
            }
        }

        // If we have separate inflow/outflow, clear the amount column
        if mapping.outflow_column.is_some() && mapping.inflow_column.is_some() {
            mapping.amount_column = None;
        }

        mapping
    }

    /// Parse an amount string, handling various formats
    fn parse_amount_string(&self, s: &str) -> Result<Money, String> {
        // Remove currency symbols, commas, spaces
        let cleaned: String = s
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '(' || *c == ')')
            .collect();

        // Handle parentheses as negative (accounting format)
        let (is_negative, value) = if cleaned.starts_with('(') && cleaned.ends_with(')') {
            (true, &cleaned[1..cleaned.len() - 1])
        } else if let Some(stripped) = cleaned.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, cleaned.as_str())
        };

        Money::parse(value)
            .map(|m| if is_negative { -m } else { m })
            .map_err(|e| format!("Could not parse amount '{}': {}", s, e))
    }

    /// Generate an import preview, checking for duplicates
    pub fn generate_preview(
        &self,
        parsed: &[Result<ParsedTransaction, String>],
        account_id: AccountId,
    ) -> EnvelopeResult<Vec<ImportPreviewEntry>> {
        let mut preview = Vec::with_capacity(parsed.len());

        // Get existing transactions for duplicate checking
        let existing_txns = self.storage.transactions.get_by_account(account_id)?;
        let existing_import_ids: HashMap<_, _> = existing_txns
            .iter()
            .filter_map(|t| {
                t.import_id
                    .as_ref()
                    .map(|id| (id.clone(), t.id.to_string()))
            })
            .collect();

        for result in parsed {
            match result {
                Ok(txn) => {
                    let status = if let Some(_existing_id) = existing_import_ids.get(&txn.import_id)
                    {
                        ImportStatus::Duplicate
                    } else {
                        ImportStatus::New
                    };

                    let existing_id = existing_import_ids.get(&txn.import_id).cloned();

                    preview.push(ImportPreviewEntry {
                        transaction: txn.clone(),
                        status,
                        existing_id,
                    });
                }
                Err(e) => {
                    preview.push(ImportPreviewEntry {
                        transaction: ParsedTransaction {
                            date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                            amount: Money::zero(),
                            payee: String::new(),
                            memo: String::new(),
                            row_number: 0,
                            import_id: String::new(),
                        },
                        status: ImportStatus::Error(e.clone()),
                        existing_id: None,
                    });
                }
            }
        }

        Ok(preview)
    }

    /// Import transactions from a preview
    pub fn import_from_preview(
        &self,
        preview: &[ImportPreviewEntry],
        account_id: AccountId,
        default_category_id: Option<CategoryId>,
        mark_cleared: bool,
    ) -> EnvelopeResult<ImportResult> {
        let txn_service = TransactionService::new(self.storage);

        let mut result = ImportResult {
            imported: 0,
            duplicates_skipped: 0,
            errors: 0,
            imported_ids: Vec::new(),
            error_messages: HashMap::new(),
        };

        for entry in preview {
            match &entry.status {
                ImportStatus::New => {
                    let input = crate::services::CreateTransactionInput {
                        account_id,
                        date: entry.transaction.date,
                        amount: entry.transaction.amount,
                        payee_name: Some(entry.transaction.payee.clone()),
                        category_id: default_category_id,
                        memo: Some(entry.transaction.memo.clone()),
                        status: if mark_cleared {
                            Some(TransactionStatus::Cleared)
                        } else {
                            None
                        },
                    };

                    match txn_service.create(input) {
                        Ok(mut txn) => {
                            // Set the import ID for duplicate detection
                            txn.import_id = Some(entry.transaction.import_id.clone());
                            self.storage.transactions.upsert(txn.clone())?;
                            result.imported += 1;
                            result.imported_ids.push(txn.id.to_string());
                        }
                        Err(e) => {
                            result.errors += 1;
                            result
                                .error_messages
                                .insert(entry.transaction.row_number, e.to_string());
                        }
                    }
                }
                ImportStatus::Duplicate => {
                    result.duplicates_skipped += 1;
                }
                ImportStatus::Error(e) => {
                    result.errors += 1;
                    result
                        .error_messages
                        .insert(entry.transaction.row_number, e.clone());
                }
            }
        }

        // Save all transactions
        self.storage.transactions.save()?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use crate::models::{Account, AccountType};
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    fn setup_test_account(storage: &Storage) -> AccountId {
        let account = Account::new("Test Account", AccountType::Checking);
        let account_id = account.id;
        storage.accounts.upsert(account).unwrap();
        storage.accounts.save().unwrap();
        account_id
    }

    #[test]
    fn test_parse_simple_csv() {
        let (_temp_dir, storage) = create_test_storage();
        let service = ImportService::new(&storage);

        let csv_data =
            "Date,Amount,Description\n2025-01-15,-50.00,Test Store\n2025-01-16,100.00,Paycheck";
        let mapping = ColumnMapping::new();
        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());

        let results = service
            .parse_csv_from_reader(&mut reader, &mapping)
            .unwrap();
        assert_eq!(results.len(), 2);

        let txn1 = results[0].as_ref().unwrap();
        assert_eq!(txn1.date, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
        assert_eq!(txn1.amount.cents(), -5000);
        assert_eq!(txn1.payee, "Test Store");

        let txn2 = results[1].as_ref().unwrap();
        assert_eq!(txn2.date, NaiveDate::from_ymd_opt(2025, 1, 16).unwrap());
        assert_eq!(txn2.amount.cents(), 10000);
    }

    #[test]
    fn test_parse_separate_inflow_outflow() {
        let (_temp_dir, storage) = create_test_storage();
        let service = ImportService::new(&storage);

        let csv_data = "Date,Outflow,Inflow,Description\n2025-01-15,50.00,,Groceries\n2025-01-16,,100.00,Paycheck";
        let mapping = ColumnMapping::separate_inout(0, 1, 2, 3);
        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());

        let results = service
            .parse_csv_from_reader(&mut reader, &mapping)
            .unwrap();
        assert_eq!(results.len(), 2);

        let txn1 = results[0].as_ref().unwrap();
        assert_eq!(txn1.amount.cents(), -5000);

        let txn2 = results[1].as_ref().unwrap();
        assert_eq!(txn2.amount.cents(), 10000);
    }

    #[test]
    fn test_parse_various_date_formats() {
        let (_temp_dir, storage) = create_test_storage();
        let service = ImportService::new(&storage);

        // MM/DD/YYYY format
        let csv_data = "Date,Amount,Description\n01/15/2025,-50.00,Test";
        let mapping = ColumnMapping::new().with_date_format("%m/%d/%Y");
        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
        let results = service
            .parse_csv_from_reader(&mut reader, &mapping)
            .unwrap();
        assert_eq!(
            results[0].as_ref().unwrap().date,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()
        );
    }

    #[test]
    fn test_parse_accounting_negative_format() {
        let (_temp_dir, storage) = create_test_storage();
        let service = ImportService::new(&storage);

        let csv_data = "Date,Amount,Description\n2025-01-15,(50.00),Test";
        let mapping = ColumnMapping::new();
        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());

        let results = service
            .parse_csv_from_reader(&mut reader, &mapping)
            .unwrap();
        let txn = results[0].as_ref().unwrap();
        assert_eq!(txn.amount.cents(), -5000);
    }

    #[test]
    fn test_duplicate_detection() {
        let (_temp_dir, storage) = create_test_storage();
        let account_id = setup_test_account(&storage);
        let service = ImportService::new(&storage);

        // First import
        let csv_data = "Date,Amount,Description\n2025-01-15,-50.00,Test Store";
        let mapping = ColumnMapping::new();
        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
        let parsed = service
            .parse_csv_from_reader(&mut reader, &mapping)
            .unwrap();

        let preview1 = service.generate_preview(&parsed, account_id).unwrap();
        assert_eq!(preview1[0].status, ImportStatus::New);

        // Import it
        service
            .import_from_preview(&preview1, account_id, None, false)
            .unwrap();

        // Try to import the same transaction again
        let preview2 = service.generate_preview(&parsed, account_id).unwrap();
        assert_eq!(preview2[0].status, ImportStatus::Duplicate);
    }

    #[test]
    fn test_detect_mapping() {
        let (_temp_dir, storage) = create_test_storage();
        let service = ImportService::new(&storage);

        let header_str = "Transaction Date,Debit,Credit,Description,Notes";
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(header_str.as_bytes());
        let headers = reader.headers().unwrap().clone();
        let mapping = service.detect_mapping_from_headers(&headers);

        assert_eq!(mapping.date_column, 0);
        assert_eq!(mapping.outflow_column, Some(1));
        assert_eq!(mapping.inflow_column, Some(2));
        assert_eq!(mapping.payee_column, Some(3));
        assert_eq!(mapping.memo_column, Some(4));
        assert!(mapping.amount_column.is_none());
    }

    #[test]
    fn test_import_result() {
        let (_temp_dir, storage) = create_test_storage();
        let account_id = setup_test_account(&storage);
        let service = ImportService::new(&storage);

        let csv_data =
            "Date,Amount,Description\n2025-01-15,-50.00,Store 1\n2025-01-16,-25.00,Store 2";
        let mapping = ColumnMapping::new();
        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
        let parsed = service
            .parse_csv_from_reader(&mut reader, &mapping)
            .unwrap();
        let preview = service.generate_preview(&parsed, account_id).unwrap();

        let result = service
            .import_from_preview(&preview, account_id, None, false)
            .unwrap();

        assert_eq!(result.imported, 2);
        assert_eq!(result.duplicates_skipped, 0);
        assert_eq!(result.errors, 0);
        assert_eq!(result.imported_ids.len(), 2);
    }
}
