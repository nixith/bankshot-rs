use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Redirect},
};
use rusqlite::{params, Connection, Result, ToSql};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tracing::info;
use tracing::{error, warn};

const DB_NAME: &str = "ducks.db";

// functions to hold html store index.html at runtime
pub fn index_html() -> &'static str {
    static HTML: OnceLock<String> = OnceLock::new();
    HTML.get_or_init(|| std::fs::read_to_string("templates/index.html").unwrap())
}

// store bill.html at runtime and eval if user can acess it
pub fn bill_html() -> &'static str {
    static LLM_HTML: OnceLock<String> = OnceLock::new();
    LLM_HTML.get_or_init(|| std::fs::read_to_string("templates/Bill.html").unwrap())
}

pub fn auth_html() -> &'static str {
    static LLM_HTML: OnceLock<String> = OnceLock::new();
    LLM_HTML.get_or_init(|| std::fs::read_to_string("templates/auth.html").unwrap())
}

pub async fn authed() -> bool {
    true // TODO: implement
}

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn login(Json(payload): Json<LoginForm>) -> (StatusCode, Json<LoginResponse>) {
    //TODO: Store session cookie
    match auth(payload.username, payload.password).await {
        Ok(true) => (
            StatusCode::OK,
            Json(LoginResponse {
                message: "Logged In".to_string(),
            }),
        ),
        Ok(false) => (
            StatusCode::FORBIDDEN,
            Json(LoginResponse {
                message: "Invalid Credentials".to_string(),
            }),
        ),
        Err(s) => (StatusCode::BAD_REQUEST, Json(LoginResponse { message: s })),
    }
}

#[derive(Serialize)]
pub struct LoginResponse {
    message: String,
}

// DB code

#[derive(Debug)]
struct User {
    id: i32,
    name: String,
    password: String,
}

pub fn setup_db() -> Result<()> {
    let conn = Connection::open(DB_NAME).map_err(|a| {
        tracing::error!("Could not connect to DB!"); // log error and propogate
        a
    })?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id   INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            password TEXT NOT NULL
        )",
        (), // empty list of parameters.
    )?;
    let users = [
        User {
            id: 0,
            name: "Swan1".to_string(),
            password: "password1".to_string(),
        },
        User {
            id: 0,
            name: "Swan1".to_string(),
            password: "password1".to_string(),
        },
        User {
            id: 0,
            name: "Swan1".to_string(),
            password: "password1".to_string(),
        },
    ];
    for me in users {
        conn.execute(
            "INSERT OR IGNORE INTO users VALUES (?, ?, ?)",
            (&me.id, &me.name, &me.password),
        )?;
    }
    _ = conn.execute("COMMIT", ()).map_err(|err| {
        warn!("Unable to create BankShot DB on Initialization");
        err
    });
    info!("Database created sucessfully!");
    Ok(())
}

pub async fn auth(username: String, password: String) -> Result<bool, String> {
    let conn = Connection::open(DB_NAME).map_err(|a| {
        tracing::error!("Could not connect to DB!"); // log error and propogate
        a.to_string()
    })?;
    let mut stmt = conn
        .prepare(
            format!(
                "SELECT * FROM users WHERE username = '{}' AND password = '{}'",
                username, password
            )
            .as_str(),
        ) // return error types here
        .map_err(|error| error.to_string())?;
    let mut rows = stmt.query([]).map_err(|err| err.to_string())?;
    match rows.next() {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(s) => Err(s.to_string()),
    }
}
