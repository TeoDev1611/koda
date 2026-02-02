# Koda - Development Roadmap & Todo

## 🚨 Critical Security & Stability
- [x] **SQL Injection Fix:** `fetch_table_as_json` now validates table names against the schema allow-list.
- [x] **Error Handling:** Implemented robust `sqlx::Error` matching (specifically `ForeignKeyViolation`) instead of brittle string matching.

## 🛠 Refactoring & Architecture
- [ ] **Traits for DB Backend:** Split `KodaDb` into a trait-based system (`impl Backend for SqliteBackend`, etc.) to avoid the large `match` blocks in every method.
- [ ] **UI State Separation:** The `App` struct in `src/ui/mod.rs` is too large. Split into:
    - `src/ui/state.rs`: Pure data state.
    - `src/ui/events.rs`: Input handling logic.
    - `src/ui/render.rs`: Drawing logic.
- [ ] **Async UI Handling:** Loading large tables blocks the main thread during `fetch_all`. Implement cursor-based pagination at the database level (current `LIMIT 500` is hardcoded).

## ✨ Features
- [ ] **NoSQL Support:** Add support for non-relational databases (e.g., MongoDB, Redis).
- [ ] **Schema Introspection:** View column types and constraints in the UI (currently just lists table names).
- [ ] **Filter/Sort:** Add UI input to filter rows (`WHERE` clause builder).
- [ ] **Blob Support:** Better visualization for binary data (currently shows `<blob: N bytes>`).
- [ ] **Config File:** Save connection strings to a `~/.config/koda.toml` file for quick access.

## 🧪 Testing
- [ ] **Unit Tests:** Add tests for `lang.rs` and basic `db` logic.
- [ ] **Integration Tests:** Use `test.db` to verify CRUD operations automatically.
