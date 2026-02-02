# Koda - Project Planning

## Vision
To create a high-performance, multi-database viewer in Rust, similar to DBeaver but native and lightweight.
**Evolution Path:** CLI Tool -> TUI (Terminal UI) -> GUI (Graphical UI).

## Phase 1: The CLI Foundation
The goal of this phase is to build the core logic for database connectivity and a command-line interface to interact with it. This ensures business logic is decoupled from the presentation layer, facilitating the future addition of a GUI.

### 1. Architecture (Clean Architecture)
To ensure modularity and ease of future migration to GUI:
*   **`koda-core` (Library):** Contains all business logic, database connection handling, and data abstraction. This is the "brain" that will be shared between CLI and GUI.
*   **`koda-cli` (Binary):** The entry point for the terminal. It handles user input, parses arguments, and calls `koda-core`.

### 2. Tech Stack
*   **Database Interface:** [`sqlx`](https://github.com/launchbadge/sqlx)
    *   *Why:* Async, type-safe, supports PostgreSQL, MySQL, SQLite, MSSQL.
*   **Async Runtime:** [`tokio`](https://tokio.rs/)
    *   *Why:* The standard runtime for async Rust.
*   **CLI Parsing:** [`clap`](https://github.com/clap-rs/clap)
    *   *Why:* Declarative, easy to use, generates help messages automatically.
*   **Error Handling:** `anyhow` (for app) & `thiserror` (for lib).
*   **Output Formatting:** [`tabled`](https://github.com/zhiburt/tabled)
    *   *Why:* Pretty-printing SQL results as ASCII tables.

### 3. Roadmap - Step-by-Step

#### Step 1: Project Setup & Dependencies
*   Initialize `Cargo.toml` with the selected stack.
*   Set up the module structure (`lib.rs` vs `main.rs`).

#### Step 2: Database Abstraction Layer
*   Define a trait `DatabaseBackend` (or similar) in `koda-core`.
*   Required methods:
    *   `connect(url: &str) -> Result<Connection>`
    *   `list_tables() -> Result<Vec<String>>`
    *   `execute_query(query: &str) -> Result<Vec<Row>>`

#### Step 3: First Implementation (SQLite/Postgres)
*   Implement the abstraction for **PostgreSQL** (or SQLite) using `sqlx`.
*   Verify connection logic.

#### Step 4: CLI Commands
Implement the following commands using `clap`:
*   `koda connect <URI>`: Test connection to a DB.
*   `koda ls`: List tables in the connected DB.
*   `koda query "<SQL>"`: Run a raw SQL query.

#### Step 5: Data Visualization
*   Take the generic query output and render it using `tabled`.
*   Ensure dynamic column headers based on query results.

## Future Phases
*   **Phase 2:** TUI (Terminal User Interface) with `ratatui`.
*   **Phase 3:** GUI with `Tauri` or `Iced`.
