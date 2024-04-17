use axum::{
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use ctf::*;

use tower_http::services::ServeDir;

use tower_sessions::{session_store::ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_rusqlite_store::tokio_rusqlite::Connection;
use tower_sessions_rusqlite_store::RusqliteStore;

use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// actual http server, everything here just specifies which functions handle what
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "ctf".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // intialize DB
    setup_db().unwrap(); // we panic on db initializaiton failure for now

    // SESSION MANAGEMENT
    // make session connection
    //
    // initialize session key store
    //
    // This can be a DB on disk, memory for now since we're just testing
    let conn = Connection::open_in_memory().await.unwrap();
    let session_store = RusqliteStore::new(conn);
    session_store.migrate().await?;

    // check every minute, delete cookies that are an hour old
    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(5)),
    );

    // initialize sesstion_store
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::hours(1),
        ))
        .with_signed(tower_sessions::cookie::Key::generate());

    // Ensure we use a shutdown signal to abort the deletion task.

    let assets_path = std::env::current_dir().unwrap();
    // build our application with a route
    let app = Router::new()
        // GET routes - serve html
        .route("/", get(|| async { Redirect::to("/bankshot") }))
        .route("/bankshot", get(|| async { Html(index_html()) }))
        .route("/bankshot/llm", get(return_llm_page))
        // POST routes - respond with json
        .route("/login", post(login))
        // extra Tower services - things like serving directories
        .nest_service(
            "/static",
            ServeDir::new(format!("{}/static", assets_path.to_str().unwrap())),
        )
        // add session store layer
        .layer(session_layer);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("starting server");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
        .await
        .unwrap();

    // cleanup session store;
    deletion_task.await??;
    info!("webserver stopped");

    Ok(())
}
