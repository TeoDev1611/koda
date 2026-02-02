# koda

**koda** is a universal database viewer and manager written in Rust. It provides a unified interface for interacting with SQLite, PostgreSQL, and MySQL databases through CLI, TUI, and GUI.

## Features

-   **Multi-Database Support:** Connect to SQLite, PostgreSQL, and MySQL/MariaDB.
-   **Graphical UI (GUI):** Professional, DBeaver-inspired interface built with `egui`.
-   **Terminal UI (TUI):** Interactive terminal interface for remote servers or quick access.
-   **CLI Commands:** Fast command-line access for scripting and automation.
-   **Safe CRUD:** Generate SQL for updates and deletes with previsualization before execution.
-   **Export:** Copy results to clipboard as CSV for easy integration with other tools.

## Installation

Ensure you have Rust installed. Clone the repository and build:

```bash
git clone https://github.com/yourusername/koda.git
cd koda
cargo build --release
```

## Usage

### 1. Graphical UI (GUI) - Recommended
For a modern, professional experience:

```bash
cargo run --release --bin koda-gui
```
- **Connection:** Use the built-in file browser for SQLite or enter credentials for network DBs.
- **SQL Editor:** Real-time syntax highlighting for SQL keywords.
- **Navigation:** Hierarchical tree view for databases and tables.
- **Actions:** Right-click cells to copy values; use ✏️/🗑️ to generate safe SQL updates/deletes.

### 2. Interactive TUI Mode
For terminal lovers:

```bash
cargo run --release -- --uri sqlite:data.db
```
*(Controls: Use arrows to navigate, Tab to switch panels, 'q' to quit)*

### 3. CLI Mode
For quick queries or listing:

```bash
# List tables
koda --uri sqlite:data.db ls

# Run SQL
koda --uri sqlite:data.db query "SELECT * FROM users"
```

## Architecture

-   **Core (`src/db`)**: Shared logic for database connections and query execution using `sqlx`.
-   **GUI (`src/gui.rs`)**: High-performance graphical interface using `egui`.
-   **TUI (`src/ui`)**: Terminal interface using `ratatui`.
-   **CLI (`src/main.rs`)**: Command-line entry point using `clap`.

## License

MIT
