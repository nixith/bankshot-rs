use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use reqwest::Client;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::{signal, task::AbortHandle};
use tower_sessions::Session;
use tracing::error;
use tracing::info;

const BAD_LLM_RESPONSE: &str = "Sorry, I can only answer questions for Stooge McHonk";
const DB_NAME: &str = "ducks.db";
const AUTH_KEY: &str = "IsAuthed";

pub fn correct_answers() -> &'static Questions {
    static CORRECT_ANSWERS: OnceLock<Questions> = OnceLock::new();
    CORRECT_ANSWERS.get_or_init(|| Questions {
        mother: "Puddles".to_string(),
        firstborn: "Gosling".to_string(),
        pet: "benjamin".to_string(),
        food: "Clover".to_string(),
        residence: "4053 Woking Way, Los Angeles, CA".to_string(),
        school: "All-Goose School of Academic Eggcellence".to_string(),
    })
}

pub fn llm_host() -> &'static str {
    static HOST: OnceLock<String> = OnceLock::new();
    HOST.get_or_init(|| std::env::var("LLM_HOST").unwrap_or("localhost".to_string()))
}

pub fn llm_port() -> &'static str {
    static PORT: OnceLock<String> = OnceLock::new();
    PORT.get_or_init(|| std::env::var("LLM_PORT").unwrap_or("localhost".to_string()))
}

pub fn llm_route() -> &'static str {
    static ROUTE: OnceLock<String> = OnceLock::new();
    ROUTE.get_or_init(|| std::env::var("LLM_ROUTE").unwrap_or("localhost".to_string()))
}

pub fn flag() -> &'static str {
    static FLAG: OnceLock<String> = OnceLock::new();
    FLAG.get_or_init(|| std::env::var("FLAG").unwrap_or("flag{abc}".to_string()))
}

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

pub fn request_client() -> &'static Client {
    static LLM_HTML: OnceLock<Client> = OnceLock::new();
    LLM_HTML.get_or_init(|| reqwest::Client::new())
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

// Security Questions
#[derive(Deserialize, Eq, PartialEq)]
pub struct Questions {
    mother: String,
    firstborn: String,
    pet: String,
    food: String,
    residence: String,
    school: String,
}

#[derive(Serialize)]
pub struct QuestionResponse {
    status: u16,
    flag: String,
}

pub async fn check_questions(Json(payload): Json<Questions>) -> impl IntoResponse {
    if payload.eq(correct_answers()) {
        (
            StatusCode::OK,
            Json(QuestionResponse {
                status: 1,
                flag: flag().to_string(),
            }),
        )
    } else {
        (
            StatusCode::OK,
            Json(QuestionResponse {
                status: 0,
                flag: "One or more of your Answers does not match".to_string(),
            }),
        )
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

#[derive(Deserialize)]
pub struct LlmResponse {
    output: String,
    metadata: serde_json::Value, // strictly speaking, we don't care about metadata
}

#[derive(Serialize)]
pub struct BillResponse {
    status: String,
    output: String,
}

#[derive(Deserialize, Serialize)]
pub struct BillRequest {
    user_input: String,
}

#[derive(Serialize)]
pub struct LangChainRequest {
    user_input: String,
}

pub async fn llm_client(session: Session, Json(payload): Json<BillRequest>) -> impl IntoResponse {
    if !is_authed(session).await {
        // user shouldn't be hitting bill without authorization - early json return
        return (
            StatusCode::UNAUTHORIZED,
            Json(BillResponse {
                status: 401.to_string(),
                output: "not authorized".to_string(),
            }),
        );
    };
    let json_data = format!(
        "{{\"input\": {{\"user_input\": \"{}\" }} }}",
        payload.user_input,
    );
    let client = request_client();
    let request = client
        .post(format!(
            "http://{host}:{port}/{route}/invoke",
            host = llm_host(),
            port = llm_port(),
            route = llm_route()
        ))
        .header("Content-Type", "application/json")
        .body(json_data.to_owned());
    println!("json data:\n{}\n", json_data);
    let res = request.send().await.unwrap();
    println!("request response:\n{:?}\n", res);
    let data = res.json::<LlmResponse>().await.unwrap();
    // we don't have michael's prompt injection sitting in front as of now, but that's ok - we use
    // good faith
    info!("LLM responded");
    if data.output.to_lowercase().contains(BAD_LLM_RESPONSE) {
        // llm says it doesn't think the user
        // is Stooge McHonk
        (
            StatusCode::OK,
            Json(BillResponse {
                status: 200.to_string(),
                output: BAD_LLM_RESPONSE.to_string(),
            }),
        )
    } else {
        (
            StatusCode::OK,
            Json(BillResponse {
                status: 200.to_string(),
                output: data.output,
            }),
        )
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
