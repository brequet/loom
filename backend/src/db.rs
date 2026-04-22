use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;

fn default_db_url() -> String {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_or_else(|_| PathBuf::from("."), PathBuf::from);
    let db_path = home.join(".loom").join("loom.db");
    format!("sqlite:{}?mode=rwc", db_path.to_string_lossy())
}

pub async fn create_pool() -> Result<SqlitePool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| default_db_url());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
