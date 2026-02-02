# Koda - Development Todo

## 🚨 Critical Security & Stability
- [x] **SQL Injection Fix:** Table names are validated against schema.
- [x] **Safe Mutations:** GUI generates SQL for review instead of blind execution.
- [ ] **Saved Credentials:** Encrypt connection strings when stored on disk.

## 🛠 Refactoring & Architecture
- [ ] **Traits for DB Backend:** Clean up `KodaDb` match blocks with a trait-based system.
- [ ] **Modularize GUI:** Split `src/gui.rs` into multiple files (app, editor, navigator, data_grid).
- [ ] **Async Loading:** Ensure long-running queries don't freeze the GUI.

## ✨ Features
- [ ] **Schema Introspection:** Show column types, nullability, and keys in the Navigator.
- [ ] **ERD Visualization:** Basic diagram view for table relationships.
- [ ] **BLOB Support:** Visual preview for binary data (Hex/Images).
- [ ] **Export to File:** Export results directly to CSV/JSON/Excel files.

## 🧪 Testing
- [ ] **Unit Tests:** Add tests for SQL generation logic.
- [ ] **Integration Tests:** Automated tests using a temporary SQLite database.