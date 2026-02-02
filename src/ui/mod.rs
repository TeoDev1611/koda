use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
};
use sqlx;
use tokio::sync::mpsc; // Import sqlx for error handling

use crate::db::KodaDb;
use crate::lang::{Language, Strings};

/// Events that happen in the background and are sent to the UI
pub enum AppEvent {
    Connected(KodaDb),
    ConnectionError(String),
    TablesLoaded(Vec<String>),
    DataLoaded(Vec<String>, Vec<Vec<String>>), // Headers, Rows
    StmtExecuted(u64),                         // Rows affected
    Error(String),
}

/// App State
enum ConnectionStatus {
    Disconnected,
    Connecting(String),
    Connected,
    Failed(String),
}

#[derive(PartialEq)]
enum ActiveBlock {
    Tables,
    Data,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
    Deleting,
}

struct App {
    status: ConnectionStatus,

    // Navigation
    tables: Vec<String>,
    list_state: ListState,
    scrollbar_state: ScrollbarState,
    active_block: ActiveBlock,

    // Data View
    current_table_name: Option<String>,
    data_headers: Vec<String>,
    data_rows: Vec<Vec<String>>,
    table_state: TableState,

    // Insertion Logic
    input_mode: InputMode,
    input: String,
    insert_values: Vec<String>,
    current_col_idx: usize,

    // Editing Logic
    is_editing: bool,
    edit_original_row: Vec<String>,

    // Logic
    db: Option<KodaDb>,
    messages: Vec<String>,

    // UI State
    show_help: bool,
    language: Language,
}

impl App {
    fn new(initial_uri: Option<String>) -> Self {
        let status = if let Some(uri) = initial_uri {
            ConnectionStatus::Connecting(uri)
        } else {
            ConnectionStatus::Disconnected
        };

        Self {
            status,
            tables: vec![],
            list_state: ListState::default(),
            scrollbar_state: ScrollbarState::default(),
            active_block: ActiveBlock::Tables,
            current_table_name: None,
            data_headers: vec![],
            data_rows: vec![],
            table_state: TableState::default(),
            input_mode: InputMode::Normal,
            input: String::new(),
            insert_values: vec![],
            current_col_idx: 0,
            is_editing: false,
            edit_original_row: vec![],
            db: None,
            messages: vec!["Ready.".to_string()],
            show_help: false,
            language: Language::Es, // Default to Spanish as requested
        }
    }

    fn toggle_language(&mut self) {
        self.language = match self.language {
            Language::En => Language::Es,
            Language::Es => Language::En,
        };
    }

    fn next(&mut self) {
        if self.active_block == ActiveBlock::Tables {
            self.next_table();
        } else {
            self.next_row();
        }
    }

    fn previous(&mut self) {
        if self.active_block == ActiveBlock::Tables {
            self.previous_table();
        } else {
            self.previous_row();
        }
    }

    fn next_table(&mut self) {
        if self.tables.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.tables.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    fn previous_table(&mut self) {
        if self.tables.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tables.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    fn next_row(&mut self) {
        if self.data_rows.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.data_rows.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous_row(&mut self) {
        if self.data_rows.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.data_rows.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn toggle_focus(&mut self) {
        self.active_block = match self.active_block {
            ActiveBlock::Tables => ActiveBlock::Data,
            ActiveBlock::Data => ActiveBlock::Tables,
        };
    }

    fn get_selected_table(&self) -> Option<String> {
        self.list_state
            .selected()
            .and_then(|i| self.tables.get(i).cloned())
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    fn next_page(&mut self) {
        let jump = 10;
        if self.active_block == ActiveBlock::Tables {
            if self.tables.is_empty() {
                return;
            }
            let i = match self.list_state.selected() {
                Some(i) => std::cmp::min(i + jump, self.tables.len() - 1),
                None => 0,
            };
            self.list_state.select(Some(i));
            self.scrollbar_state = self.scrollbar_state.position(i);
        } else {
            if self.data_rows.is_empty() {
                return;
            }
            let i = match self.table_state.selected() {
                Some(i) => std::cmp::min(i + jump, self.data_rows.len() - 1),
                None => 0,
            };
            self.table_state.select(Some(i));
        }
    }

    fn previous_page(&mut self) {
        let jump = 10;
        if self.active_block == ActiveBlock::Tables {
            if self.tables.is_empty() {
                return;
            }
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i < jump {
                        0
                    } else {
                        i - jump
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
            self.scrollbar_state = self.scrollbar_state.position(i);
        } else {
            if self.data_rows.is_empty() {
                return;
            }
            let i = match self.table_state.selected() {
                Some(i) => {
                    if i < jump {
                        0
                    } else {
                        i - jump
                    }
                }
                None => 0,
            };
            self.table_state.select(Some(i));
        }
    }
}

pub async fn run(initial_uri: Option<String>) -> Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Setup Channels for Async Communication
    let (tx, mut rx) = mpsc::channel::<AppEvent>(10);

    // 3. Spawn background connection task if URI is present
    if let Some(uri) = initial_uri.clone() {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            match KodaDb::connect(&uri).await {
                Ok(db) => {
                    if let Err(_) = tx_clone.send(AppEvent::Connected(db)).await {
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx_clone
                        .send(AppEvent::ConnectionError(e.to_string()))
                        .await;
                }
            }
        });
    }

    // 4. Run App Loop
    let mut app = App::new(initial_uri);
    let res = run_app(&mut terminal, &mut app, &mut rx, tx).await;

    // 5. Restore Terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: &mut mpsc::Receiver<AppEvent>,
    tx: mpsc::Sender<AppEvent>,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Handle Input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.show_help {
                        if key.code == KeyCode::Char('q')
                            || key.code == KeyCode::Esc
                            || key.code == KeyCode::Char('?')
                        {
                            app.toggle_help();
                        }
                    } else if app.input_mode == InputMode::Editing {
                        match key.code {
                            KeyCode::Enter => {
                                let val = app.input.drain(..).collect::<String>();

                                // Logic: If editing and empty input, keep original value
                                let final_val = if app.is_editing && val.is_empty() {
                                    app.edit_original_row
                                        .get(app.current_col_idx)
                                        .cloned()
                                        .unwrap_or_default()
                                } else {
                                    val
                                };

                                app.insert_values.push(final_val);
                                app.current_col_idx += 1;

                                if app.current_col_idx >= app.data_headers.len() {
                                    // Finish insertion/update
                                    app.input_mode = InputMode::Normal;
                                    if let (Some(db), Some(table)) =
                                        (&app.db, &app.current_table_name)
                                    {
                                        let cols = app.data_headers.clone(); // Clone headers
                                        let vals = app.insert_values.clone(); // Clone values

                                        let quote_char = match db.kind() {
                                            crate::db::DbKind::Mysql => "`",
                                            _ => "\"",
                                        };
                                        let q =
                                            |s: &str| format!("{}{}{}", quote_char, s, quote_char);

                                        let query = if app.is_editing {
                                            // UPDATE Logic
                                            let set_clause = cols
                                                .iter()
                                                .zip(vals.iter())
                                                .map(|(c, v)| {
                                                    format!("{} = '{}'", q(c), v.replace("'", "''"))
                                                })
                                                .collect::<Vec<_>>()
                                                .join(", ");

                                            let where_clause = cols
                                                .iter()
                                                .zip(app.edit_original_row.iter())
                                                .map(|(c, v)| {
                                                    if v == "NULL" {
                                                        format!("{} IS NULL", q(c))
                                                    } else {
                                                        format!(
                                                            "{} = '{}'",
                                                            q(c),
                                                            v.replace("'", "''")
                                                        )
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                                .join(" AND ");

                                            format!(
                                                "UPDATE {} SET {} WHERE {}",
                                                q(table),
                                                set_clause,
                                                where_clause
                                            )
                                        } else {
                                            // INSERT Logic
                                            let col_str = cols
                                                .iter()
                                                .map(|c| q(c))
                                                .collect::<Vec<_>>()
                                                .join(", ");
                                            let val_str = vals
                                                .iter()
                                                .map(|v| format!("'{}'", v.replace("'", "''")))
                                                .collect::<Vec<_>>()
                                                .join(", ");
                                            format!(
                                                "INSERT INTO {} ({}) VALUES ({})",
                                                q(table),
                                                col_str,
                                                val_str
                                            )
                                        };

                                        let tx_clone = tx.clone();
                                        let db_clone = db.clone();
                                        tokio::spawn(async move {
                                            match db_clone.execute_stmt(&query).await {
                                                Ok(affected) => {
                                                    let _ = tx_clone
                                                        .send(AppEvent::StmtExecuted(affected))
                                                        .await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_clone
                                                        .send(AppEvent::Error(format!(
                                                            "Operation failed: {}",
                                                            e
                                                        )))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                    app.insert_values.clear();
                                    app.edit_original_row.clear();
                                    app.current_col_idx = 0;
                                    app.is_editing = false;
                                }
                            }
                            KeyCode::Char(c) => {
                                app.input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.input.clear();
                                app.insert_values.clear();
                                app.is_editing = false;
                            }
                            _ => {}
                        }
                    } else if app.input_mode == InputMode::Deleting {
                        match key.code {
                            KeyCode::Char('y') => {
                                // Execute Delete
                                if let (Some(db), Some(table)) = (&app.db, &app.current_table_name)
                                {
                                    if let Some(selected_row_idx) = app.table_state.selected() {
                                        if let Some(row) = app.data_rows.get(selected_row_idx) {
                                            let quote_char = match db.kind() {
                                                crate::db::DbKind::Mysql => "`",
                                                _ => "\"",
                                            };

                                            let q = |s: &str| {
                                                format!("{}{}{}", quote_char, s, quote_char)
                                            };

                                            let conditions: Vec<String> = app
                                                .data_headers
                                                .iter()
                                                .zip(row.iter())
                                                .map(|(col, val)| {
                                                    if val == "NULL" {
                                                        format!("{} IS NULL", q(col))
                                                    } else {
                                                        format!(
                                                            "{} = '{}'",
                                                            q(col),
                                                            val.replace("'", "''")
                                                        )
                                                    }
                                                })
                                                .collect();

                                            let where_clause = conditions.join(" AND ");
                                            let query = format!(
                                                "DELETE FROM {} WHERE {}",
                                                q(table),
                                                where_clause
                                            );

                                            let tx_clone = tx.clone();
                                            let db_clone = db.clone();
                                            let lang = app.language;
                                            tokio::spawn(async move {
                                                match db_clone.execute_stmt(&query).await {
                                                    Ok(affected) => {
                                                        let _ = tx_clone
                                                            .send(AppEvent::StmtExecuted(affected))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let mut friendly = format!(
                                                            "{}: {}",
                                                            Strings::get(
                                                                &lang,
                                                                "error_delete_failed"
                                                            ),
                                                            e
                                                        );

                                                        if let Some(sqlx_err) =
                                                            e.downcast_ref::<sqlx::Error>()
                                                        {
                                                            if let sqlx::Error::Database(db_err) =
                                                                sqlx_err
                                                            {
                                                                if db_err.kind() == sqlx::error::ErrorKind::ForeignKeyViolation {
                                                                    friendly = Strings::get(&lang, "error_foreign_key");
                                                                }
                                                            }
                                                        }

                                                        let _ = tx_clone
                                                            .send(AppEvent::Error(friendly))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('?') => app.toggle_help(),
                            KeyCode::Char('l') => app.toggle_language(),
                            KeyCode::Tab => app.toggle_focus(),
                            KeyCode::Down => app.next(),
                            KeyCode::Up => app.previous(),
                            KeyCode::PageDown => app.next_page(),
                            KeyCode::PageUp => app.previous_page(),
                            KeyCode::Char('a')
                                if app.active_block == ActiveBlock::Data
                                    && !app.data_headers.is_empty() =>
                            {
                                app.input_mode = InputMode::Editing;
                                app.current_col_idx = 0;
                                app.insert_values.clear();
                                app.input.clear();
                                app.is_editing = false;
                            }
                            KeyCode::Char('e')
                                if app.active_block == ActiveBlock::Data
                                    && !app.data_rows.is_empty() =>
                            {
                                if let Some(selected_idx) = app.table_state.selected() {
                                    if let Some(row) = app.data_rows.get(selected_idx) {
                                        app.edit_original_row = row.clone();
                                        app.input_mode = InputMode::Editing;
                                        app.current_col_idx = 0;
                                        app.insert_values.clear();
                                        app.input.clear();
                                        app.is_editing = true;
                                    }
                                }
                            }
                            KeyCode::Char('x')
                                if app.active_block == ActiveBlock::Data
                                    && !app.data_rows.is_empty() =>
                            {
                                if app.table_state.selected().is_some() {
                                    app.input_mode = InputMode::Deleting;
                                }
                            }
                            KeyCode::Enter if app.active_block == ActiveBlock::Tables => {
                                if let Some(table_name) = app.get_selected_table() {
                                    if let Some(db) = &app.db {
                                        app.messages.push(format!("Loading {}...", table_name));
                                        app.current_table_name = Some(table_name.clone());
                                        app.data_headers.clear();
                                        app.data_rows.clear();

                                        let db_clone = db.clone();
                                        let tx_clone = tx.clone();
                                        let query =
                                            format!("SELECT * FROM {} LIMIT 500", table_name); // Limit increased
                                        tokio::spawn(async move {
                                            match db_clone.execute_query(&query).await {
                                                Ok((h, r)) => {
                                                    let _ = tx_clone
                                                        .send(AppEvent::DataLoaded(h, r))
                                                        .await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_clone
                                                        .send(AppEvent::Error(format!(
                                                            "Query failed: {}",
                                                            e
                                                        )))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Handle Background Events
        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppEvent::Connected(db) => {
                    app.status = ConnectionStatus::Connected;
                    app.db = Some(db.clone());
                    let tx_clone = tx.clone();
                    let db_clone = db.clone();
                    tokio::spawn(async move {
                        if let Ok(tables) = db_clone.list_tables().await {
                            let _ = tx_clone.send(AppEvent::TablesLoaded(tables)).await;
                        }
                    });
                }
                AppEvent::ConnectionError(err) => {
                    app.status = ConnectionStatus::Failed(err.clone());
                    app.messages.push(format!("Error: {}", err));
                }
                AppEvent::TablesLoaded(tables) => {
                    app.tables = tables;
                    app.scrollbar_state = app.scrollbar_state.content_length(app.tables.len());
                    if !app.tables.is_empty() {
                        app.list_state.select(Some(0));
                        app.scrollbar_state = app.scrollbar_state.position(0);
                    }
                }
                AppEvent::DataLoaded(headers, rows) => {
                    app.data_headers = headers;
                    app.data_rows = rows;
                    if !app.data_rows.is_empty() {
                        app.table_state.select(Some(0));
                    }
                    app.messages.push("Data loaded.".to_string());
                }
                AppEvent::StmtExecuted(affected) => {
                    app.messages
                        .push(format!("Success: {} row(s) affected.", affected));
                    // Refresh data
                    if let (Some(db), Some(table)) = (&app.db, &app.current_table_name) {
                        let db_clone = db.clone();
                        let tx_clone = tx.clone();
                        let query = format!("SELECT * FROM {} LIMIT 500", table); // Limit increased
                        tokio::spawn(async move {
                            if let Ok((h, r)) = db_clone.execute_query(&query).await {
                                let _ = tx_clone.send(AppEvent::DataLoaded(h, r)).await;
                            }
                        });
                    }
                }
                AppEvent::Error(err) => {
                    app.messages.push(format!("Error: {}", err));
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Main Block with Double Borders
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " KODA DB VIEWER ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    f.render_widget(main_block, size);

    // Inner Layout (excluding main border)
    let inner_area = size.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    // 2. Main Content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Tables List
            Constraint::Percentage(80), // Data View
        ])
        .split(chunks[0]);

    // --- Styles ---
    let style_active = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let style_inactive = Style::default().fg(Color::DarkGray);
    let style_title = Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::BOLD);
    let style_selected = Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);
    let style_header = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);

    // -- Left Panel: Tables List --
    let items: Vec<ListItem> = app
        .tables
        .iter()
        .map(|t| ListItem::new(t.as_str()))
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if app.active_block == ActiveBlock::Tables {
            style_active
        } else {
            style_inactive
        })
        .title(Span::styled(
            Strings::get(&app.language, "title_tables"),
            style_title,
        ));

    let tables_list = List::new(items)
        .block(list_block)
        .highlight_style(style_selected)
        .highlight_symbol(">>");

    f.render_stateful_widget(tables_list, main_chunks[0], &mut app.list_state);

    // Scrollbar for tables
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"));

    f.render_stateful_widget(
        scrollbar,
        main_chunks[0].inner(ratatui::layout::Margin {
            vertical: 0,
            horizontal: 0,
        }), // Render inside the list area
        &mut app.scrollbar_state,
    );

    // -- Right Panel: Data Table --
    let data_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if app.active_block == ActiveBlock::Data {
            style_active
        } else {
            style_inactive
        })
        .title(Span::styled(
            format!(
                "{}{}",
                Strings::get(&app.language, "title_data"),
                app.current_table_name.as_deref().unwrap_or("-")
            ),
            style_title,
        ));

    let data_area = main_chunks[1];

    if !app.data_headers.is_empty() {
        let headers_cells = app
            .data_headers
            .iter()
            .map(|h| Cell::from(Span::styled(h, style_header)));
        let header = Row::new(headers_cells).height(1).bottom_margin(1);

        let rows = app.data_rows.iter().map(|item| {
            let cells = item.iter().map(|c| {
                let style = if c == "NULL" {
                    Style::default().fg(Color::DarkGray)
                } else if c.parse::<f64>().is_ok() || c.parse::<i64>().is_ok() {
                    Style::default().fg(Color::LightYellow)
                } else if c == "true" {
                    Style::default().fg(Color::LightGreen)
                } else if c == "false" {
                    Style::default().fg(Color::LightRed)
                } else {
                    Style::default().fg(Color::White)
                };
                Cell::from(c.as_str()).style(style)
            });
            Row::new(cells)
        });

        let col_count = app.data_headers.len();
        let constraints: Vec<Constraint> = (0..col_count)
            .map(|_| Constraint::Ratio(1, col_count as u32))
            .collect();

        let t = Table::new(rows, constraints)
            .header(header)
            .block(data_block)
            .row_highlight_style(style_selected)
            .column_spacing(1);

        f.render_stateful_widget(t, data_area, &mut app.table_state);
    } else {
        let p = Paragraph::new(Strings::get(&app.language, "hint_select"))
            .block(data_block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, data_area);
    }

    // 3. Footer (Detailed)
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Handle Input Prompt
    if app.input_mode == InputMode::Editing {
        let col_name = app
            .data_headers
            .get(app.current_col_idx)
            .cloned()
            .unwrap_or_default();
        let action = if app.is_editing {
            Strings::get(&app.language, "action_editing")
        } else {
            Strings::get(&app.language, "action_adding")
        };
        let hint = if app.is_editing {
            format!(
                "({}: {})",
                Strings::get(&app.language, "original"),
                app.edit_original_row
                    .get(app.current_col_idx)
                    .map(|s| s.as_str())
                    .unwrap_or("")
            )
        } else {
            "".to_string()
        };

        let input_text = format!("{} | Column [{}]: {} {}", action, col_name, app.input, hint);
        let p = Paragraph::new(Span::styled(
            input_text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        f.render_widget(p, chunks[1]);
    } else {
        let status_text = match &app.status {
            ConnectionStatus::Disconnected => Span::styled(
                Strings::get(&app.language, "status_disconnected"),
                Style::default().fg(Color::Gray),
            ),
            ConnectionStatus::Connecting(uri) => Span::styled(
                format!(
                    "{} {}...",
                    Strings::get(&app.language, "status_connecting"),
                    uri
                ),
                Style::default().fg(Color::Yellow),
            ),
            ConnectionStatus::Connected => Span::styled(
                Strings::get(&app.language, "status_connected"),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            ConnectionStatus::Failed(err) => Span::styled(
                format!("{}: {}", Strings::get(&app.language, "status_failed"), err),
                Style::default().fg(Color::Red),
            ),
        };

        let left_footer = Line::from(vec![
            Span::raw(" Status: "),
            status_text,
            Span::raw(format!(
                " | Msg: {}",
                app.messages.last().cloned().unwrap_or_default()
            )),
        ]);
        f.render_widget(Paragraph::new(left_footer), footer_chunks[0]);

        let right_text = if app.show_help {
            "Help Open"
        } else {
            &Strings::get(&app.language, "hint_nav")
        };
        let right_footer = Paragraph::new(Span::raw(right_text)).alignment(Alignment::Right);
        f.render_widget(right_footer, footer_chunks[1]);
    }

    // 4. Help Popup
    if app.show_help {
        let area = centered_rect(60, 50, size);
        f.render_widget(Clear, area); // Clear background

        let block = Block::default()
            .title(Strings::get(&app.language, "help_title"))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let text = vec![
            Line::from(Strings::get(&app.language, "help_nav_title")),
            Line::from(Strings::get(&app.language, "help_nav_desc")),
            Line::from(Strings::get(&app.language, "help_tab")),
            Line::from(Strings::get(&app.language, "help_enter")),
            Line::from(""),
            Line::from(Strings::get(&app.language, "help_edit_title")),
            Line::from(Strings::get(&app.language, "help_edit_a")),
            Line::from(Strings::get(&app.language, "help_edit_e")),
            Line::from(Strings::get(&app.language, "help_edit_x")),
            Line::from(""),
            Line::from(Strings::get(&app.language, "help_general")),
            Line::from(Strings::get(&app.language, "help_lang")),
            Line::from("  ?:       Toggle this Help"),
            Line::from("  q:       Quit"),
        ];
        let p = Paragraph::new(text).block(block);
        f.render_widget(p, area);
    }

    // 5. Delete Confirmation Popup
    if app.input_mode == InputMode::Deleting {
        let area = centered_rect(40, 20, size);
        f.render_widget(Clear, area);

        let block = Block::default()
            .title(Strings::get(&app.language, "confirm_delete_title"))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        let text = vec![
            Line::from(""),
            Line::from(Strings::get(&app.language, "confirm_delete_msg")),
            Line::from(""),
            Line::from(Span::styled(
                Strings::get(&app.language, "confirm_yes"),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(Strings::get(&app.language, "confirm_no")),
        ];
        let p = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(p, area);
    }
}

// Helper for centering the popup
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let vertical_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    vertical_layout[1]
}
