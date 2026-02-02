# Koda - Project Planning & Design

## Vision
To create a high-performance, multi-database viewer in Rust, similar to DBeaver but native, lightweight, and fast.
**Evolution Path:** CLI Tool -> TUI (Terminal UI) -> GUI (Graphical UI).

## Architecture
Koda follows a modular approach:
*   **Core Logic:** Single source of truth for DB interaction in `src/db/`.
*   **Presentations:** 
    *   **CLI:** Fast, scriptable.
    *   **TUI:** Efficient for remote/SSH work.
    *   **GUI:** Rich, feature-complete for local development.

## Tech Stack Selection
*   **Database:** `sqlx` (Async, multi-backend).
*   **GUI:** `egui` (Immediate mode, lightweight, no heavy C++ dependencies).
*   **TUI:** `ratatui` (Modern Rust TUI).
*   **CLI:** `clap` (Standard CLI parser).

## Principles
1.  **Safety First:** Don't delete or update data without showing the user the SQL first.
2.  **Performance:** Keep memory footprint low; use immediate mode UI for instant feedback.
3.  **Hacker Friendly:** Keep it simple, monospace-focused, and fast.