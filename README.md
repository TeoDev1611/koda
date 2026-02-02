# koda

**koda** is a terminal-based database viewer and manager written in Rust. It aims to provide a unified interface for interacting with SQLite, PostgreSQL, and MySQL databases directly from your terminal.

## Features

-   **Multi-Database Support:** Connect to SQLite, PostgreSQL, and MySQL/MariaDB using standard connection strings.
-   **Terminal UI (TUI):** Interactive interface to browse tables and data without writing SQL.
-   **CRUD Operations:** Insert, update, and delete rows directly from the TUI.
-   **Import/Export:** Export tables to JSON/YAML and import data from files.
-   **Cross-Platform:** Runs on Linux, macOS, and Windows.

## Installation

Ensure you have Rust installed. Clone the repository and build:

```bash
git clone https://github.com/yourusername/koda.git
cd koda
cargo build --release
```

## Usage

### 1. Interactive TUI Mode (Recommended)

Start the interactive viewer by providing a database URI:

```bash
# SQLite
./target/release/koda --uri sqlite:my_database.db

# PostgreSQL
./target/release/koda --uri postgres://user:pass@localhost/dbname

# MySQL
./target/release/koda --uri mysql://user:pass@localhost/dbname
```

**Navigation Controls:**
-   `Up/Down`: Select tables or rows.
-   `Tab`: Switch focus between Table List and Data View.
-   `Enter`: Load data for selected table.
-   `?`: Toggle Help menu.
-   `q`: Quit.

**Editing Controls:**
-   `a`: Add a new row.
-   `e`: Edit the selected row.
-   `x`: Delete the selected row.

### 2. CLI Commands

**List Tables:**
```bash
koda --uri sqlite:data.db ls
```

**Run SQL Query:**
```bash
koda --uri sqlite:data.db query "SELECT * FROM users WHERE active = 1"
```

**Export Data:**
```bash
# Export all tables to JSON files in ./backup directory
koda --uri sqlite:data.db export --output ./backup --format json
```

## Architecture

The project is modularized into:
-   `src/db`: Database connection pooling and abstraction logic.
-   `src/ui`: Terminal UI rendering and state management using `ratatui`.
-   `src/lang`: Localization support (English/Spanish).

## License

MIT