use eframe::egui;
use koda::KodaDb;
use rfd::FileDialog;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

fn main() -> eframe::Result {
    let rt = Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("Koda"),
        ..Default::default()
    };

    eframe::run_native(
        "Koda",
        options,
        Box::new(|cc| {
            configure_dbeaver_theme(&cc.egui_ctx);
            Ok(Box::new(KodaApp::new(rt)))
        }),
    )
}

fn configure_dbeaver_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.dark_mode = true;
    style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 35);
    style.visuals.panel_fill = egui::Color32::from_rgb(35, 35, 40);
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 35, 40);
    style.visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_gray(60));
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 50);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 100, 200);
    ctx.set_style(style);
}

// --- Syntax Highlighting Logic ---
fn highlight_sql(ui: &egui::Ui, text: &str, _wrap_width: f32) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::default();
    let mut current_word = String::new();

    let color_keyword = egui::Color32::from_rgb(255, 140, 0);
    let color_func = egui::Color32::from_rgb(100, 180, 255);
    let color_string = egui::Color32::from_rgb(150, 255, 150);
    let color_default = egui::Color32::LIGHT_GRAY;

    let keywords = [
        "SELECT", "FROM", "WHERE", "LIMIT", "ORDER", "BY", "AND", "OR", "AS", "UPDATE", "DELETE",
        "INSERT", "INTO", "VALUES", "SET", "CREATE", "TABLE", "DROP", "ALTER", "JOIN", "ON",
        "LEFT", "RIGHT", "INNER", "OUTER", "GROUP", "HAVING",
    ];
    let functions = ["COUNT", "SUM", "AVG", "MAX", "MIN", "NOW", "DATE"];
    let font_id = egui::FontId::monospace(14.0);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_alphanumeric() || c == '_' {
            current_word.push(c);
            i += 1;
        } else {
            if !current_word.is_empty() {
                let upper = current_word.to_uppercase();
                let color = if keywords.contains(&upper.as_str()) {
                    color_keyword
                } else if functions.contains(&upper.as_str()) {
                    color_func
                } else if current_word.chars().all(|c| c.is_numeric()) {
                    egui::Color32::from_rgb(255, 100, 100)
                } else {
                    color_default
                };
                job.append(
                    &current_word,
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
                current_word.clear();
            }
            if c == '\'' {
                let mut s = String::from("'");
                i += 1;
                while i < chars.len() {
                    s.push(chars[i]);
                    if chars[i] == '\'' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                job.append(
                    &s,
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        color: color_string,
                        ..Default::default()
                    },
                );
                continue;
            }
            job.append(
                &c.to_string(),
                0.0,
                egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::GRAY,
                    ..Default::default()
                },
            );
            i += 1;
        }
    }
    if !current_word.is_empty() {
        let upper = current_word.to_uppercase();
        let color = if keywords.contains(&upper.as_str()) {
            color_keyword
        } else if functions.contains(&upper.as_str()) {
            color_func
        } else if current_word.chars().all(|c| c.is_numeric()) {
            egui::Color32::from_rgb(255, 100, 100)
        } else {
            color_default
        };
        job.append(
            &current_word,
            0.0,
            egui::TextFormat {
                font_id,
                color,
                ..Default::default()
            },
        );
    }
    ui.fonts(|f| f.layout_job(job))
}

#[derive(PartialEq)]
enum ConnMode {
    Sqlite,
    Postgres,
    Mysql,
    RawUri,
}

struct KodaApp {
    rt: Runtime,

    // UI Config
    ui_scale: f32,

    // Connection Config
    conn_mode: ConnMode,
    sqlite_path: String,
    net_host: String,
    net_port: String,
    net_user: String,
    net_pass: String,
    net_db: String,
    raw_uri: String,

    // App State
    db: Option<Arc<KodaDb>>,
    is_connected: bool,
    status_msg: String,
    last_query_rows: usize,
    last_query_duration: Option<Duration>,
    error: Option<String>,
    loading: bool,

    // Navigation
    tables: Vec<String>,
    filtered_tables: Vec<String>,
    table_filter: String,

    // Workspace
    query: String,
    limit: u64,
    results: Option<(Vec<String>, Vec<Vec<String>>)>,
}

impl KodaApp {
    fn new(rt: Runtime) -> Self {
        Self {
            rt,
            ui_scale: 1.2, // Fixed comfortable scale

            conn_mode: ConnMode::Sqlite,
            sqlite_path: String::new(),
            net_host: "localhost".to_string(),
            net_port: "5432".to_string(),
            net_user: "postgres".to_string(),
            net_pass: "password".to_string(),
            net_db: "postgres".to_string(),
            raw_uri: String::new(),

            db: None,
            is_connected: false,
            status_msg: "Ready".to_string(),
            last_query_rows: 0,
            last_query_duration: None,
            error: None,
            loading: false,

            tables: vec![],
            filtered_tables: vec![],
            table_filter: String::new(),

            query: "SELECT 1".to_string(),
            limit: 100,
            results: None,
        }
    }

    fn build_uri(&self) -> String {
        match self.conn_mode {
            ConnMode::Sqlite => format!("sqlite:{}", self.sqlite_path),
            ConnMode::Postgres => format!(
                "postgres://{}:{}@{}:{}/{}",
                self.net_user, self.net_pass, self.net_host, self.net_port, self.net_db
            ),
            ConnMode::Mysql => format!(
                "mysql://{}:{}@{}:{}/{}",
                self.net_user, self.net_pass, self.net_host, self.net_port, self.net_db
            ),
            ConnMode::RawUri => self.raw_uri.clone(),
        }
    }

    fn connect(&mut self) {
        let uri = self.build_uri();
        if uri.is_empty() || uri == "sqlite:" {
            self.error = Some("Invalid URI".into());
            return;
        }

        self.loading = true;
        self.status_msg = "Connecting...".into();
        let start = Instant::now();

        match self.rt.block_on(KodaDb::connect(&uri)) {
            Ok(db) => {
                let db_arc = Arc::new(db);
                if let Ok(tables) = self.rt.block_on(db_arc.list_tables()) {
                    self.tables = tables.clone();
                    self.filtered_tables = tables;
                    self.db = Some(db_arc);
                    self.is_connected = true;
                    self.status_msg = format!("Connected in {:.2?}", start.elapsed());
                    self.error = None;
                }
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.status_msg = "Connection Failed".into();
            }
        }
        self.loading = false;
    }

    fn run_query(&mut self) {
        if let Some(db) = &self.db {
            self.loading = true;
            self.status_msg = "Executing...".into();
            let start = Instant::now();
            match self.rt.block_on(db.execute_query(&self.query)) {
                Ok(res) => {
                    self.last_query_rows = res.1.len();
                    self.results = Some(res);
                    self.last_query_duration = Some(start.elapsed());
                    self.status_msg = "Query executed successfully".into();
                    self.error = None;
                }
                Err(e) => {
                    self.error = Some(e.to_string());
                    self.status_msg = "Query failed".into();
                }
            }
            self.loading = false;
        }
    }

    fn load_table(&mut self, table: &str) {
        self.query = format!("SELECT * FROM {} LIMIT {}", table, self.limit);
        self.run_query();
    }

    // --- GUI Components ---

    fn render_connection_modal(&mut self, ctx: &egui::Context) {
        egui::Window::new("🔌 Connect")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(350.0);
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.conn_mode, ConnMode::Sqlite, "SQLite");
                    ui.selectable_value(&mut self.conn_mode, ConnMode::Postgres, "Postgres");
                    ui.selectable_value(&mut self.conn_mode, ConnMode::Mysql, "MySQL");
                    ui.selectable_value(&mut self.conn_mode, ConnMode::RawUri, "Raw URI");
                });
                ui.separator();

                egui::Grid::new("conn_form")
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| match self.conn_mode {
                        ConnMode::Sqlite => {
                            ui.label("Path");
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut self.sqlite_path);
                                if ui.button("📂").clicked() {
                                    if let Some(p) = FileDialog::new()
                                        .add_filter("SQLite", &["db", "sqlite"])
                                        .pick_file()
                                    {
                                        self.sqlite_path = p.display().to_string();
                                    }
                                }
                            });
                            ui.end_row();
                        }
                        ConnMode::Postgres | ConnMode::Mysql => {
                            ui.label("Host");
                            ui.text_edit_singleline(&mut self.net_host);
                            ui.end_row();
                            ui.label("Port");
                            ui.text_edit_singleline(&mut self.net_port);
                            ui.end_row();
                            ui.label("Database");
                            ui.text_edit_singleline(&mut self.net_db);
                            ui.end_row();
                            ui.label("User");
                            ui.text_edit_singleline(&mut self.net_user);
                            ui.end_row();
                            ui.label("Password");
                            ui.add(egui::TextEdit::singleline(&mut self.net_pass).password(true));
                            ui.end_row();
                        }
                        ConnMode::RawUri => {
                            ui.label("URI");
                            ui.text_edit_singleline(&mut self.raw_uri);
                            ui.end_row();
                        }
                    });

                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        self.connect();
                    }
                    if self.loading {
                        ui.spinner();
                    }
                });

                if let Some(e) = &self.error {
                    ui.add_space(5.0);
                    ui.colored_label(egui::Color32::RED, e);
                }
            });
    }

    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_msg);
                ui.separator();

                if let Some(dur) = self.last_query_duration {
                    ui.label(format!("Time: {:.2?}", dur));
                    ui.separator();
                }

                ui.label(format!("Rows: {}", self.last_query_rows));
                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let db_name = if self.sqlite_path.is_empty() {
                        &self.net_db
                    } else {
                        "sqlite"
                    };
                    ui.label(format!("🛢 {} ", db_name));
                    ui.separator();
                    ui.weak("v0.1.0 ");
                });
            });
        });
    }

    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigator")
            .resizable(true)
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.add_space(5.0);
                ui.heading("Database Navigator");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.table_filter)
                            .hint_text("Filter tables..."),
                    );
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let db_label = if self.conn_mode == ConnMode::Sqlite {
                        "SQLite DB"
                    } else {
                        &self.net_db
                    };

                    egui::CollapsingHeader::new(format!("🔌 {}", db_label))
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::CollapsingHeader::new("📁 Tables")
                                .default_open(true)
                                .show(ui, |ui| {
                                    if !self.table_filter.is_empty() {
                                        let f = self.table_filter.to_lowercase();
                                        self.filtered_tables = self
                                            .tables
                                            .iter()
                                            .filter(|t| t.to_lowercase().contains(&f))
                                            .cloned()
                                            .collect();
                                    } else {
                                        self.filtered_tables = self.tables.clone();
                                    }

                                    let mut load_target = None;
                                    for table in &self.filtered_tables {
                                        if ui
                                            .selectable_label(false, format!("  📄 {}", table))
                                            .clicked()
                                        {
                                            load_target = Some(table.clone());
                                        }
                                    }
                                    if let Some(t) = load_target {
                                        self.load_table(&t);
                                    }
                                });

                            egui::CollapsingHeader::new("📂 Views").show(ui, |ui| {
                                ui.weak("  (None)");
                            });
                        });
                });
            });
    }
}

impl eframe::App for KodaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // APPLY ZOOM
        ctx.set_pixels_per_point(self.ui_scale);

        if !self.is_connected {
            self.render_connection_modal(ctx);
            return;
        }

        self.render_status_bar(ctx);
        self.render_sidebar(ctx);

        egui::TopBottomPanel::top("main_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("🔌 Disconnect").clicked() {
                    self.is_connected = false;
                    self.db = None;
                    self.results = None;
                }
                ui.separator();
                if ui
                    .button("▶ Run Script")
                    .on_hover_text("Execute SQL (Ctrl+Enter)")
                    .clicked()
                {
                    self.run_query();
                }
                egui::ComboBox::from_id_salt("limit_sel")
                    .selected_text(format!("Limit: {}", self.limit))
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.limit, 10, "10");
                        ui.selectable_value(&mut self.limit, 100, "100");
                        ui.selectable_value(&mut self.limit, 1000, "1000");
                        ui.selectable_value(&mut self.limit, 10000, "10000");
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Alternative location for Zoom or Settings if needed
                    ui.label(egui::RichText::new("Koda v0.1").weak());
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let editor_height = ui.available_height() * 0.4;

            egui::ScrollArea::vertical()
                .max_height(editor_height)
                .id_salt("editor_scroll")
                .show(ui, |ui| {
                    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                        highlight_sql(ui, string, wrap_width)
                    };

                    let res = ui.add(
                        egui::TextEdit::multiline(&mut self.query)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter)
                            .lock_focus(true),
                    );
                    if res.has_focus()
                        && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter))
                    {
                        self.run_query();
                    }
                });

            ui.separator();

            ui.push_id("results_area", |ui| {
                if let Some((headers, rows)) = &self.results {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Results").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("📋 Copy CSV").clicked() {
                                let mut csv = headers.join(",");
                                csv.push('\n');
                                for row in rows {
                                    csv.push_str(&row.join(","));
                                    csv.push('\n');
                                }
                                ui.output_mut(|o| o.copied_text = csv);
                            }
                        });
                    });

                    egui::ScrollArea::both().show(ui, |ui| {
                        egui::Grid::new("grid_results")
                            .striped(true)
                            .spacing([20.0, 5.0])
                            .min_col_width(60.0)
                            .show(ui, |ui| {
                                ui.label("");
                                for h in headers {
                                    ui.label(
                                        egui::RichText::new(h)
                                            .strong()
                                            .color(egui::Color32::from_rgb(100, 200, 255)),
                                    );
                                }
                                ui.end_row();

                                let mut action = None;
                                for row in rows {
                                    ui.horizontal(|ui| {
                                        ui.style_mut().spacing.item_spacing.x = 0.0;
                                        if ui.small_button("✏️").clicked() {
                                            action = Some((true, row.clone(), headers.clone()));
                                        }
                                        if ui.small_button("🗑").clicked() {
                                            action = Some((false, row.clone(), headers.clone()));
                                        }
                                    });
                                    for cell in row {
                                        let txt = if cell.len() > 60 {
                                            format!("{}...", &cell[0..57])
                                        } else {
                                            cell.clone()
                                        };

                                        let label = if cell == "NULL" {
                                            ui.label(egui::RichText::new("NULL").weak().italics())
                                        } else if cell == "true" || cell == "1" {
                                            ui.label(
                                                egui::RichText::new(txt)
                                                    .color(egui::Color32::GREEN),
                                            )
                                        } else if cell == "false" || cell == "0" {
                                            ui.label(
                                                egui::RichText::new(txt).color(egui::Color32::RED),
                                            )
                                        } else if cell.parse::<f64>().is_ok() {
                                            ui.label(
                                                egui::RichText::new(txt)
                                                    .color(egui::Color32::from_rgb(150, 255, 255)),
                                            )
                                        } else {
                                            ui.label(egui::RichText::new(txt).monospace())
                                        };

                                        label.context_menu(|ui| {
                                            if ui.button("📋 Copy Value").clicked() {
                                                ui.output_mut(|o| o.copied_text = cell.clone());
                                                ui.close_menu();
                                            }
                                        });
                                    }
                                    ui.end_row();
                                }

                                if let Some((is_update, row, headers)) = action {
                                    let w = headers
                                        .iter()
                                        .zip(row.iter())
                                        .map(|(h, v)| {
                                            if v == "NULL" {
                                                format!("{} IS NULL", h)
                                            } else if v.parse::<f64>().is_ok() {
                                                format!("{}={}", h, v)
                                            } else {
                                                format!("{}='{}'", h, v.replace("'", "''"))
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                        .join(" AND ");

                                    if is_update {
                                        self.query =
                                            format!("UPDATE TABLE SET col='val' WHERE {};", w);
                                    } else {
                                        self.query = format!("DELETE FROM TABLE WHERE {};", w);
                                    }
                                }
                            });
                    });
                } else if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", err));
                }
            });
        });
    }
}
