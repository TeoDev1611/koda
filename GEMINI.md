# koda

## Project Overview
`koda` is a generic database viewer tool written in Rust. It supports CLI, TUI, and GUI interfaces.

## Technologies
- **Language:** Rust (Edition 2024)
- **Database Engine:** `sqlx` (Supports SQLite, Postgres, MySQL)
- **GUI Framework:** `egui` / `eframe`
- **TUI Framework:** `ratatui`
- **CLI Framework:** `clap`
- **Async Runtime:** `tokio`

## Architecture
1.  **Core (`src/db/`):** Holds the `KodaDb` struct. Responsible for connection pooling, executing queries, and introspecting schemas.
2.  **GUI (`src/gui.rs`):** The primary graphical interface. Professional layout with syntax highlighting and safe SQL generation.
3.  **CLI (`src/main.rs`):** Handles command-line arguments and standard output.
4.  **TUI (`src/ui/`):** Terminal-based interactive interface.

## Key Commands
- `cargo run --bin koda-gui` - Launch the GUI.
- `cargo run -- --uri <URI> ls` - CLI: List tables.
- `cargo run -- --uri <URI> query "<SQL>"` - CLI: Run SQL.

## Development Status
**Current Phase:** Phase 2 (GUI) Prototype completed and refined.
**Next Phase:** Phase 3 (Advanced Features) - Schema introspection, ERD diagrams, and saved connections.
