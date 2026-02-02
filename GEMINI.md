# koda

## Project Overview
`koda` is a generic database viewer tool written in Rust. It currently functions as a CLI but is architected to support TUI and GUI interfaces in the future.

## Technologies
- **Language:** Rust (Edition 2024)
- **Database Engine:** `sqlx` (Supports SQLite, Postgres, MySQL)
- **CLI Framework:** `clap`
- **Async Runtime:** `tokio`
- **Formatting:** `tabled`

## Architecture
The project is divided into:
1.  **Core (`src/lib.rs`):** Holds the `KodaDb` struct. Responsible for connection pooling, executing queries, and introspecting schemas.
2.  **CLI (`src/main.rs`):** Handles user input and displays output using standard streams.

## Key Commands
- `cargo run -- --uri <URI> connect` - Test connection.
- `cargo run -- --uri <URI> ls` - List tables.
- `cargo run -- --uri <URI> query "<SQL>"` - Run SQL.

## Development Status
**Current Phase:** Phase 1 (CLI) Completed.
**Next Phase:** Phase 2 (TUI) - Implementing interactive terminal UI with `ratatui`.