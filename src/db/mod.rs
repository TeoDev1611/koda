use anyhow::{Result, anyhow};
use serde_json::{Map, Value};
use sqlx::Row;
use sqlx::mysql::MySqlPool;
use sqlx::postgres::PgPool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum DbKind {
    Sqlite,
    Postgres,
    Mysql,
}

#[derive(Clone)]
enum KodaPool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
    Mysql(MySqlPool),
}

#[derive(Clone)]
pub struct KodaDb {
    pool: KodaPool,
    kind: DbKind,
}

impl KodaDb {
    pub async fn connect(uri: &str) -> Result<Self> {
        if uri.starts_with("sqlite:") {
            let opts = SqliteConnectOptions::from_str(uri)?.foreign_keys(true);
            let pool = SqlitePool::connect_with(opts).await?;
            Ok(Self {
                pool: KodaPool::Sqlite(pool),
                kind: DbKind::Sqlite,
            })
        } else if uri.starts_with("postgres:") || uri.starts_with("postgresql:") {
            let pool = PgPool::connect(uri).await?;
            Ok(Self {
                pool: KodaPool::Postgres(pool),
                kind: DbKind::Postgres,
            })
        } else if uri.starts_with("mysql:") || uri.starts_with("mariadb:") {
            let pool = MySqlPool::connect(uri).await?;
            Ok(Self {
                pool: KodaPool::Mysql(pool),
                kind: DbKind::Mysql,
            })
        } else {
            Err(anyhow!("Unsupported database URI: {}", uri))
        }
    }

    pub fn kind(&self) -> DbKind {
        self.kind
    }

    pub async fn ping(&self) -> Result<()> {
        match &self.pool {
            KodaPool::Sqlite(p) => {
                sqlx::query("SELECT 1").execute(p).await?;
            }
            KodaPool::Postgres(p) => {
                sqlx::query("SELECT 1").execute(p).await?;
            }
            KodaPool::Mysql(p) => {
                sqlx::query("SELECT 1").execute(p).await?;
            }
        }
        Ok(())
    }

    pub async fn list_tables(&self) -> Result<Vec<String>> {
        let query = match self.kind {
            DbKind::Sqlite => {
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
            }
            DbKind::Postgres => {
                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'"
            }
            DbKind::Mysql => "SHOW TABLES",
        };

        let mut tables = Vec::new();

        match &self.pool {
            KodaPool::Sqlite(p) => {
                let rows = sqlx::query(query).fetch_all(p).await?;
                for row in rows {
                    tables.push(row.try_get(0)?);
                }
            }
            KodaPool::Postgres(p) => {
                let rows = sqlx::query(query).fetch_all(p).await?;
                for row in rows {
                    tables.push(row.try_get(0)?);
                }
            }
            KodaPool::Mysql(p) => {
                let rows = sqlx::query(query).fetch_all(p).await?;
                for row in rows {
                    tables.push(row.try_get(0)?);
                }
            }
        }

        Ok(tables)
    }

    pub async fn execute_query(&self, query: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        match &self.pool {
            KodaPool::Sqlite(p) => self.execute_sqlite(p, query).await,
            KodaPool::Postgres(p) => self.execute_postgres(p, query).await,
            KodaPool::Mysql(p) => self.execute_mysql(p, query).await,
        }
    }

    /// Executes a statement that doesn't return rows (INSERT, UPDATE, DELETE).
    /// Returns the number of rows affected.
    pub async fn execute_stmt(&self, query: &str) -> Result<u64> {
        let rows_affected = match &self.pool {
            KodaPool::Sqlite(p) => sqlx::query(query).execute(p).await?.rows_affected(),
            KodaPool::Postgres(p) => sqlx::query(query).execute(p).await?.rows_affected(),
            KodaPool::Mysql(p) => sqlx::query(query).execute(p).await?.rows_affected(),
        };
        Ok(rows_affected)
    }

    /// Fetches all data from a table and returns it as a JSON Array of Objects.
    pub async fn fetch_table_as_json(&self, table_name: &str) -> Result<Value> {
        // SECURITY: Verify table exists to prevent SQL injection via table_name
        let tables = self.list_tables().await?;
        if !tables.contains(&table_name.to_string()) {
            return Err(anyhow!("Table '{}' does not exist", table_name));
        }

        let quote_char = match self.kind {
            DbKind::Mysql => "`",
            _ => "\"",
        };
        let query = format!("SELECT * FROM {}{}{}", quote_char, table_name, quote_char);

        match &self.pool {
            KodaPool::Sqlite(p) => self.fetch_json_sqlite(p, &query).await,
            KodaPool::Postgres(p) => self.fetch_json_postgres(p, &query).await,
            KodaPool::Mysql(p) => self.fetch_json_mysql(p, &query).await,
        }
    }

    async fn fetch_json_sqlite(&self, pool: &SqlitePool, query: &str) -> Result<Value> {
        use sqlx::Column;
        let rows = sqlx::query(query).fetch_all(pool).await?;
        let mut json_rows = Vec::new();

        for row in rows {
            let mut map = Map::new();
            for col in row.columns() {
                let name = col.name();
                let i = col.ordinal();

                let val = if let Ok(v) = row.try_get::<i64, _>(i) {
                    Value::Number(v.into())
                } else if let Ok(v) = row.try_get::<f64, _>(i) {
                    serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                } else if let Ok(v) = row.try_get::<bool, _>(i) {
                    Value::Bool(v)
                } else if let Ok(v) = row.try_get::<String, _>(i) {
                    Value::String(v)
                } else {
                    Value::Null
                };
                map.insert(name.to_string(), val);
            }
            json_rows.push(Value::Object(map));
        }
        Ok(Value::Array(json_rows))
    }

    async fn fetch_json_postgres(&self, pool: &PgPool, query: &str) -> Result<Value> {
        use sqlx::Column;
        let rows = sqlx::query(query).fetch_all(pool).await?;
        let mut json_rows = Vec::new();

        for row in rows {
            let mut map = Map::new();
            for col in row.columns() {
                let name = col.name();
                let i = col.ordinal();

                // Postgres types mapping
                let val = if let Ok(v) = row.try_get::<i64, _>(i) {
                    Value::Number(v.into())
                } else if let Ok(v) = row.try_get::<i32, _>(i) {
                    Value::Number(v.into())
                } else if let Ok(v) = row.try_get::<bool, _>(i) {
                    Value::Bool(v)
                } else if let Ok(v) = row.try_get::<String, _>(i) {
                    Value::String(v)
                } else if let Ok(v) = row.try_get::<f64, _>(i) {
                    serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                } else {
                    Value::Null
                };

                map.insert(name.to_string(), val);
            }
            json_rows.push(Value::Object(map));
        }
        Ok(Value::Array(json_rows))
    }

    async fn fetch_json_mysql(&self, pool: &MySqlPool, query: &str) -> Result<Value> {
        use sqlx::Column;
        let rows = sqlx::query(query).fetch_all(pool).await?;
        let mut json_rows = Vec::new();

        for row in rows {
            let mut map = Map::new();
            for col in row.columns() {
                let name = col.name();
                let i = col.ordinal();

                let val = if let Ok(v) = row.try_get::<i64, _>(i) {
                    Value::Number(v.into())
                } else if let Ok(v) = row.try_get::<i32, _>(i) {
                    Value::Number(v.into())
                } else if let Ok(v) = row.try_get::<u64, _>(i) {
                    Value::Number(v.into())
                } else if let Ok(v) = row.try_get::<bool, _>(i) {
                    Value::Bool(v)
                } else if let Ok(v) = row.try_get::<String, _>(i) {
                    Value::String(v)
                } else if let Ok(v) = row.try_get::<f64, _>(i) {
                    serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                } else {
                    Value::Null
                };

                map.insert(name.to_string(), val);
            }
            json_rows.push(Value::Object(map));
        }
        Ok(Value::Array(json_rows))
    }

    async fn execute_sqlite(
        &self,
        pool: &SqlitePool,
        query: &str,
    ) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        use sqlx::Column;
        let rows = sqlx::query(query).fetch_all(pool).await?;
        if rows.is_empty() {
            return Ok((vec![], vec![]));
        }

        let headers: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();
        let mut results = Vec::new();

        for row in rows {
            let mut current = Vec::new();
            for i in 0..headers.len() {
                // SQLite specific extraction
                let val = if let Ok(v) = row.try_get::<String, _>(i) {
                    v
                } else if let Ok(v) = row.try_get::<i64, _>(i) {
                    v.to_string()
                } else if let Ok(v) = row.try_get::<f64, _>(i) {
                    v.to_string()
                } else if let Ok(v) = row.try_get::<bool, _>(i) {
                    v.to_string()
                } else if let Ok(v) = row.try_get::<Vec<u8>, _>(i) {
                    format!("<blob: {} bytes>", v.len())
                } else if row.try_get::<Option<String>, _>(i).is_ok() {
                    "NULL".to_string()
                } else {
                    "<unknown>".to_string()
                };
                current.push(val);
            }
            results.push(current);
        }
        Ok((headers, results))
    }

    /// Imports a generic JSON/YAML Value (Array of Objects) into a table.
    /// If table doesn't exist, it attempts to infer schema from the first row.
    pub async fn import_table(&self, table_name: &str, data: &Value) -> Result<u64> {
        let rows = data
            .as_array()
            .ok_or_else(|| anyhow!("Data is not an array"))?;
        if rows.is_empty() {
            return Ok(0);
        }

        let quote_char = match self.kind {
            DbKind::Mysql => "`",
            _ => "\"",
        };
        let q = |s: &str| format!("{}{}{}", quote_char, s, quote_char);

        // 1. Infer Schema from first row
        let first_row = rows[0]
            .as_object()
            .ok_or_else(|| anyhow!("Row is not an object"))?;
        let columns: Vec<String> = first_row.keys().cloned().collect();

        // Create Table SQL (Basic inference)
        let mut create_sql = format!("CREATE TABLE IF NOT EXISTS {} (", q(table_name));
        for (i, (k, v)) in first_row.iter().enumerate() {
            let type_str = if v.is_i64() {
                "INTEGER"
            } else if v.is_f64() {
                "REAL"
            } else if v.is_boolean() {
                "BOOLEAN"
            } else {
                "TEXT"
            };

            create_sql.push_str(&format!("{} {}", q(k), type_str));
            if i < first_row.len() - 1 {
                create_sql.push_str(", ");
            }
        }
        create_sql.push_str(")");

        self.execute_stmt(&create_sql).await?;

        // 2. Insert Data
        let mut count = 0;
        let col_names = columns.iter().map(|c| q(c)).collect::<Vec<_>>().join(", ");

        for row in rows {
            let obj = row.as_object().unwrap();
            let vals: Vec<String> = columns
                .iter()
                .map(|col| {
                    match obj.get(col) {
                        Some(Value::String(s)) => format!("'{}'", s.replace("'", "''")),
                        Some(Value::Number(n)) => n.to_string(),
                        Some(Value::Bool(b)) => {
                            if *b {
                                "1".to_string()
                            } else {
                                "0".to_string()
                            }
                        } // SQLite boolean mapping
                        Some(Value::Null) | None => "NULL".to_string(),
                        _ => "'?'".to_string(),
                    }
                })
                .collect();

            let query = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                q(table_name),
                col_names,
                vals.join(", ")
            );

            self.execute_stmt(&query).await?;
            count += 1;
        }

        Ok(count)
    }

    async fn execute_postgres(
        &self,
        pool: &PgPool,
        query: &str,
    ) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        use sqlx::Column;
        let rows = sqlx::query(query).fetch_all(pool).await?;
        if rows.is_empty() {
            return Ok((vec![], vec![]));
        }

        let headers: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();
        let mut results = Vec::new();

        for row in rows {
            let mut current = Vec::new();
            for i in 0..headers.len() {
                // Postgres specific extraction
                let val = if let Ok(v) = row.try_get::<String, _>(i) {
                    v
                } else if let Ok(v) = row.try_get::<i32, _>(i) {
                    v.to_string()
                } else if let Ok(v) = row.try_get::<i64, _>(i) {
                    v.to_string()
                } else if let Ok(v) = row.try_get::<bool, _>(i) {
                    v.to_string()
                } else if let Ok(v) = row.try_get::<f64, _>(i) {
                    v.to_string()
                } else if row.try_get::<Option<String>, _>(i).is_ok() {
                    "NULL".to_string()
                } else {
                    "<unknown>".to_string()
                };
                current.push(val);
            }
            results.push(current);
        }
        Ok((headers, results))
    }

    async fn execute_mysql(
        &self,
        pool: &MySqlPool,
        query: &str,
    ) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        use sqlx::Column;
        let rows = sqlx::query(query).fetch_all(pool).await?;
        if rows.is_empty() {
            return Ok((vec![], vec![]));
        }

        let headers: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();
        let mut results = Vec::new();

        for row in rows {
            let mut current = Vec::new();
            for i in 0..headers.len() {
                let val = if let Ok(v) = row.try_get::<String, _>(i) {
                    v
                } else if let Ok(v) = row.try_get::<i64, _>(i) {
                    v.to_string()
                } else if let Ok(v) = row.try_get::<i32, _>(i) {
                    v.to_string()
                } else if let Ok(v) = row.try_get::<f64, _>(i) {
                    v.to_string()
                } else if let Ok(v) = row.try_get::<bool, _>(i) {
                    v.to_string()
                } else if row.try_get::<Option<String>, _>(i).is_ok() {
                    "NULL".to_string()
                } else {
                    "<unknown>".to_string()
                };
                current.push(val);
            }
            results.push(current);
        }
        Ok((headers, results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_connection() {
        let db = KodaDb::connect("sqlite::memory:").await;
        assert!(db.is_ok());
        let db = db.unwrap();
        let ping = db.ping().await;
        assert!(ping.is_ok());
    }
}
