use axum::{
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use ctf::*;

use tower_http::services::ServeDir;

use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// actual http server, everything here just specifies which functions handle what
#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "ctf".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // intialize DB
    setup_db().unwrap(); // we panic on db initializaiton failure for now
    let assets_path = std::env::current_dir().unwrap();
    // build our application with a route
    let app = Router::new()
        // GET routes - serve html
        .route("/", get(|| async { Redirect::permanent("/bankshot") }))
        .route("/bankshot", get(|| async { Html(index_html()) }))
        .route(
            "/bankshot/llm",
            get(|| async {
                if authed().await {
                    Html(bill_html())
                } else {
                    Html(auth_html())
                }
            }),
        )
        // POST routes - respond with json
        .route("/login", post(login))
        // extra Tower services - things like serving directories
        .nest_service(
            "/static",
            ServeDir::new(format!("{}/static", assets_path.to_str().unwrap())),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("starting server");
    axum::serve(listener, app).await.unwrap();
    info!("webserver started");
}
