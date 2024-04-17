use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};

use reqwest::Client;
use rusqlite::{Connection, Result};
use std::sync::OnceLock;
use tokio::{signal, task::AbortHandle};
use tower_sessions::Session;
use tracing::error;
use tracing::info;

const DB_NAME: &str = "ducks.db";
const AUTH_KEY: &str = "IsAuthed";

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

// serializable struct for if a user is authed
#[derive(Default, Deserialize, Serialize)]
struct AuthSession(bool);

pub async fn is_authed(session: Session) -> bool {
    // return what is heled by auth key, default to false if anything happens
    session
        .get(AUTH_KEY)
        .await
        .unwrap_or(Some(false))
        .unwrap_or(false)
}

pub async fn return_llm_page(session: Session) -> impl IntoResponse {
    if is_authed(session).await {
        Html(bill_html())
    } else {
        Html(auth_html())
    }
}

// the input to our login form handler
#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn login(
    session: Session, // current session
    Json(payload): Json<LoginForm>,
) -> (StatusCode, Json<LoginResponse>) {
    //TODO: Store session cookie
    match auth(payload.username, payload.password).await {
        Ok(true) => {
            // store session cookie
            session.insert(AUTH_KEY, true).await.unwrap();
            //respond with json
            (
                StatusCode::OK,
                Json(LoginResponse {
                    message: "Logged In".to_string(),
                }),
            )
        }
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
            id: 1,
            name: "Swan2".to_string(),
            password: "password2".to_string(),
        },
        User {
            id: 2,
            name: "Swan3".to_string(),
            password: "password3".to_string(),
        },
    ];
    for me in users {
        conn.execute(
            "INSERT OR IGNORE INTO users VALUES (?, ?, ?)",
            (&me.id, &me.name, &me.password),
        )?;
    }
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

pub async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Shutting down Session Store");
            deletion_task_abort_handle.abort()
        },
        _ = terminate => {

            info!("Shutting down Session Store");
            deletion_task_abort_handle.abort() },
    }
}
