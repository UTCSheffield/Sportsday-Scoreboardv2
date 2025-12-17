use actix_web::{get, post, web, HttpResponse};
use askama::Template;
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::templates::AdminSqliteTemplate;

#[derive(Serialize, Deserialize)]
pub struct SqliteCommand {
    query: String,
}

#[derive(Serialize, Deserialize)]
pub struct SqliteResult {
    success: bool,
    output: String,
    error: Option<String>,
}

#[get("")]
pub async fn get() -> HttpResponse {
    let command_history = Vec::new(); // We'll start with an empty history

    HttpResponse::Ok().body(
        AdminSqliteTemplate { command_history }
            .render()
            .expect("Template should be valid"),
    )
}

#[post("/execute")]
pub async fn execute(
    _app_state: web::Data<crate::AppState>,
    cmd: web::Json<SqliteCommand>,
) -> HttpResponse {
    // Get the database path from the environment or use default
    let db_path = std::env::var("DB_URL").unwrap_or_else(|_| "./db.sqlite".to_string());

    // Validate the command to prevent dangerous operations
    let query = cmd.query.trim();

    // Block potentially dangerous commands
    if is_dangerous_command(query) {
        return HttpResponse::BadRequest().json(SqliteResult {
            success: false,
            output: String::new(),
            error: Some("Dangerous command blocked for security reasons".to_string()),
        });
    }

    // Execute the SQLite command
    let output = Command::new("sqlite3").arg(&db_path).arg(query).output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            if result.status.success() {
                HttpResponse::Ok().json(SqliteResult {
                    success: true,
                    output: stdout.to_string(),
                    error: None,
                })
            } else {
                HttpResponse::Ok().json(SqliteResult {
                    success: false,
                    output: stdout.to_string(),
                    error: Some(stderr.to_string()),
                })
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(SqliteResult {
            success: false,
            output: String::new(),
            error: Some(format!("Failed to execute command: {}", e)),
        }),
    }
}

fn is_dangerous_command(query: &str) -> bool {
    let query_lower = query.to_lowercase();

    // Block commands that could be dangerous
    let dangerous_patterns = [
        ".quit",
        ".exit",
        ".shell",
        ".system",
        ".load",
        ".import",
        ".output",
        ".backup",
        ".restore",
        "attach database",
        "detach database",
    ];

    for pattern in &dangerous_patterns {
        if query_lower.contains(pattern) {
            return true;
        }
    }

    false
}
