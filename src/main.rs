use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use koda::KodaDb;
use tabled::{builder::Builder, settings::Style};

// mod lang; // Removed, internal to lib
// mod tui; // Removed, using library export
use koda::ui; // Use the exposed module from lib

#[derive(Parser)]
#[command(name = "koda")]
#[command(about = "A multi-database viewer in Rust", long_about = None)]
struct Cli {
    /// Database connection string (URI)
    /// Examples:
    /// - sqlite::memory:
    /// - sqlite:data.db
    /// - postgres://user:pass@localhost/db
    #[arg(long, short, global = true)]
    uri: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test connection to a database
    Connect,
    /// List tables in the database
    Ls,
    /// Execute a raw SQL query
    Query {
        /// SQL query to execute
        sql: String,
    },
    /// Launch Terminal UI
    Tui,
    /// Export database to JSON/YAML files
    Export {
        /// Target directory
        #[arg(long = "output", short = 'o')]
        output: String,
        /// Format: json or yaml
        #[arg(long = "format", short = 'f', default_value = "json")]
        format: String,
    },
    /// Import database from directory
    Import {
        /// Source directory
        #[arg(long = "input", short = 'i')]
        input: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Tui) | None => {
            return ui::run(cli.uri).await;
        }
        _ => {}
    }

    // For other commands, URI is required (mostly)
    let uri = match cli.uri {
        Some(u) => u,
        None => {
            eprintln!("❌ Error: --uri <URI> is required for this command.");
            std::process::exit(1);
        }
    };

    match &cli.command {
        Some(Commands::Connect) => {
            println!("🔌 Intentando conectar a: {}...", uri);
            match KodaDb::connect(&uri).await {
                Ok(db) => match db.ping().await {
                    Ok(_) => println!("✅ ¡Conexión exitosa! La base de datos responde."),
                    Err(e) => println!("⚠️ Conectado, pero falló el ping: {}", e),
                },
                Err(e) => {
                    println!("❌ Error al conectar: {}", e);
                }
            }
        }
        Some(Commands::Ls) => {
            println!("📋 Listando tablas en: {}...", uri);
            let db = KodaDb::connect(&uri).await?;
            let tables = db.list_tables().await?;

            if tables.is_empty() {
                println!("(No tables found)");
            } else {
                // Using a simple builder for a single column list
                let mut builder = Builder::default();
                builder.push_record(["Table Name"]);

                for table in tables {
                    builder.push_record([table]);
                }

                let mut table = builder.build();
                table.with(Style::rounded());
                println!("{}", table);
            }
        }
        Some(Commands::Query { sql }) => {
            let db = KodaDb::connect(&uri).await?;
            let (headers, rows) = db.execute_query(sql).await?;

            if rows.is_empty() {
                println!("(No results)");
                return Ok(());
            }

            let mut builder = Builder::default();
            builder.push_record(headers);

            for row in rows {
                builder.push_record(row);
            }

            let mut table = builder.build();
            table.with(Style::rounded());
            println!("{}", table);
        }
        Some(Commands::Export { output, format }) => {
            use serde_json::json;
            use std::fs;
            use std::path::Path;

            println!(
                "{} {} ({})",
                "🚀 Iniciando exportación a:".cyan().bold(),
                output,
                format
            );
            let db = KodaDb::connect(&uri).await?;

            // Create directory
            fs::create_dir_all(output)?;

            // List tables
            let tables = db.list_tables().await?;
            println!("{} {}", "📦 Tablas encontradas:".yellow(), tables.len());

            for table in &tables {
                println!("   - Exportando tabla: {}...", table.cyan());
                match db.fetch_table_as_json(table).await {
                    Ok(json_data) => {
                        let file_path = if format == "yaml" {
                            Path::new(output).join(format!("{}.yaml", table))
                        } else {
                            Path::new(output).join(format!("{}.json", table))
                        };

                        let content = if format == "yaml" {
                            serde_yaml::to_string(&json_data).map_err(|e| anyhow::anyhow!(e))?
                        } else {
                            serde_json::to_string_pretty(&json_data)?
                        };

                        fs::write(file_path, content)?;
                    }
                    Err(e) => {
                        eprintln!("     {} {}: {}", "❌ Error exportando".red(), table, e);
                    }
                }
            }

            // Create Metadata
            let metadata = json!({
                "source_uri": uri,
                "exported_at": chrono::Local::now().to_rfc3339(),
                "tables_count": tables.len(),
                "tables": tables,
                "format": format
            });

            let meta_path = if format == "yaml" {
                Path::new(output).join("_metadata.yaml")
            } else {
                Path::new(output).join("_metadata.json")
            };

            let meta_content = if format == "yaml" {
                serde_yaml::to_string(&metadata).map_err(|e| anyhow::anyhow!(e))?
            } else {
                serde_json::to_string_pretty(&metadata)?
            };

            fs::write(meta_path, meta_content)?;

            println!("{}", "✅ ¡Exportación completada!".green().bold());
        }
        Some(Commands::Import { input }) => {
            use serde_json::Value;
            use std::fs;
            use std::path::Path;

            println!(
                "{} {}",
                "📥 Iniciando importación desde:".cyan().bold(),
                input
            );
            let db = KodaDb::connect(&uri).await?;
            let dir = Path::new(input);

            // Look for metadata to guide us, or scan files
            // Simple scan: look for .json or .yaml files
            let entries = fs::read_dir(dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let fname = path.file_name().unwrap().to_str().unwrap();
                    if fname.starts_with("_metadata") {
                        continue;
                    }

                    let table_name = path.file_stem().unwrap().to_str().unwrap();
                    let ext = path.extension().unwrap_or_default().to_str().unwrap_or("");

                    let content = fs::read_to_string(&path)?;
                    let data: Value = if ext == "yaml" || ext == "yml" {
                        serde_yaml::from_str(&content).map_err(|e| anyhow::anyhow!(e))?
                    } else if ext == "json" {
                        serde_json::from_str(&content)?
                    } else {
                        continue;
                    };

                    println!(
                        "   - Importando tabla: {} (desde {})...",
                        table_name.cyan(),
                        fname
                    );
                    match db.import_table(table_name, &data).await {
                        Ok(count) => println!("     {} Insertadas {} filas.", "✅".green(), count),
                        Err(e) => println!("     {} Error: {}", "❌".red(), e),
                    }
                }
            }
            println!("{}", "✅ ¡Importación finalizada!".green().bold());
        }
        _ => {} // Tui or None handled above
    }

    Ok(())
}
