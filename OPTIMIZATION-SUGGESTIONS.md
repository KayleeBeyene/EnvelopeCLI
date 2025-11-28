# Code Optimization Suggestions

This document outlines key areas for code improvement, focusing on performance, robustness, and maintainability. Each section details an issue and provides a clear path to resolution that can be followed by a development agent.

## 1. Refactor CSV Import to Use Streaming

-   **Priority**: Critical
-   **File**: `src/cli/import.rs`
-   **Line**: `32`
-   **Issue**: The current implementation reads the entire CSV file into memory using `std::fs::read_to_string` before processing. This approach is inefficient and will fail for large files due to high memory consumption.
-   **Solution**: Modify the `ImportService` and the `handle_import_command` function to use a streaming parser. The `csv` crate's `Reader` should be used to process the file record by record. This will keep memory usage low and constant, regardless of the input file size.

    **Actionable Steps:**
    1.  Modify the `ImportService` to accept a `Reader` object instead of a string slice.
    2.  In `cli/import.rs`, create a `csv::Reader` from the file path.
    3.  Pass the reader to the service to parse the data in a streaming fashion.

    **Example (Conceptual):**
    ```rust
    // In services/import.rs, the service should be adapted to use a reader
    pub fn parse_csv_from_reader<R: std::io::Read>(
        &self,
        reader: &mut csv::Reader<R>,
        mapping: &CsvMapping,
    ) -> EnvelopeResult<Vec<ImportedTransaction>> {
        // ... new logic to iterate over records from the reader ...
    }

    // In cli/import.rs, the handler should be updated
    let path = Path::new(file);
    let mut reader = csv::Reader::from_path(path)
        .map_err(|e| EnvelopeError::Import(format!("Failed to open file: {}", e)))?;

    let headers = reader.headers()
        .map_err(|e| EnvelopeError::Import(format!("Failed to read CSV headers: {}", e)))?
        .clone();
    
    // The mapping detection logic would also need to be adapted
    let mapping = import_service.detect_mapping_from_headers(&headers);

    let parsed = import_service.parse_csv_from_reader(&mut reader, &mapping)?;
    // ...
    ```

## 2. Prevent Transfers to the Same Account

-   **Priority**: Important
-   **File**: `src/cli/transfer.rs`
-   **Line**: Approximately `34` (after fetching accounts).
-   **Issue**: The application does not prevent a user from transferring funds from an account to itself. This action creates confusing, offsetting transactions that serve no practical purpose and clutter the transaction history.
-   **Solution**: Add a validation check at the beginning of `handle_transfer_command` that compares the source and destination account IDs. If the IDs match, return a user-friendly validation error and halt the operation.

    **Actionable Steps:**
    1.  After retrieving `from_account` and `to_account`, compare their `id` fields.
    2.  If they are identical, return an `EnvelopeError::Validation` with an explanatory message.

    **Implementation:**
    ```rust
    // In src/cli/transfer.rs
    pub fn handle_transfer_command(...) -> EnvelopeResult<()> {
        // ...
        let from_account = account_service.find(from)?.ok_or_else(...);
        let to_account = account_service.find(to)?.ok_or_else(...);

        if from_account.id == to_account.id {
            return Err(EnvelopeError::Validation(
                "Source and destination accounts cannot be the same.".to_string(),
            ));
        }
        // ... continue with transfer logic
    }
    ```

## 3. Refactor `handle_import_command` for Clarity and Maintainability

-   **Priority**: Nice to Have
-   **File**: `src/cli/import.rs`
-   **Lines**: `13-104`
-   **Issue**: The `handle_import_command` function is overly long and handles multiple distinct responsibilities: file I/O, user interaction (printing previews and results), and orchestrating service layer calls. This monolithic structure reduces readability, testability, and maintainability.
-   **Solution**: Decompose the function into smaller, single-purpose private helper functions within the same module. This will improve separation of concerns and make the control flow easier to follow.

    **Actionable Steps:**
    1.  Create a private function to handle reading the file and parsing the CSV data.
    2.  Create a second private function to generate the import preview and display it to the user.
    3.  Create a third private function to execute the final import and print the results.
    4.  Update the main `handle_import_command` function to call these helpers in sequence.

    **Suggested Refactoring:**
    ```rust
    // In src/cli/import.rs

    pub fn handle_import_command(...) -> EnvelopeResult<()> {
        let (import_service, account_service) = ...;
        let (parsed, target_account) = read_and_parse_csv(&import_service, &account_service, file, account)?;
        
        if parsed.is_empty() {
            println!("No transactions found in CSV file.");
            return Ok(());
        }

        let preview = generate_and_display_preview(&import_service, &parsed, &target_account)?;

        if preview.iter().any(|e| e.status == ImportStatus::New) {
            execute_import(&import_service, &preview, target_account.id)?;
        }
        Ok(())
    }

    fn read_and_parse_csv(...) -> EnvelopeResult<(Vec<ImportedTransaction>, Account)> {
        // ... logic for finding account, reading file, parsing ...
    }

    fn generate_and_display_preview(...) -> EnvelopeResult<Vec<ImportPreviewEntry>> {
        // ... logic for generating preview and printing summary to console ...
    }

    fn execute_import(...) -> EnvelopeResult<()> {
        // ... logic for calling import_service.import_from_preview and printing final results ...
    }
    ```
