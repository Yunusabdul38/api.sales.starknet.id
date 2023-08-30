#[macro_use]
mod utils;
mod config;
mod endpoints;
mod logger;
mod models;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use logger::Logger;
use mongodb::{bson::doc, options::ClientOptions, Client};
use starknet::core::types::AddTransactionResultCode;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    println!(
        "starting v{} of api.sales.starknet.id",
        env!("CARGO_PKG_VERSION")
    );
    let conf = config::load();
    let logger = Logger::new(&conf.watchtower);
    let client_options = ClientOptions::parse(&conf.database.connection_string)
        .await
        .unwrap();
    let shared_state = Arc::new(models::AppState {
        conf: conf.clone(),
        logger: logger.clone(),
        db: Client::with_options(client_options)
            .unwrap()
            .database(&conf.database.name),
    });
    if shared_state
        .db
        .run_command(doc! {"ping": 1}, None)
        .await
        .is_err()
    {
        logger.severe("unable to connect to database");
        return;
    } else {
        logger.info("database: connected")
    }

    let cors = CorsLayer::new().allow_headers(Any).allow_origin(Any);
    let app = Router::new()
        .route("/", get(root))
        .route("/add", post(endpoints::add_metadata::handler))
        .with_state(shared_state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], conf.server.port));
    logger.info(format!("listening on http://0.0.0.0:{}", conf.server.port,));
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn root() -> (StatusCode, String) {
    (
        StatusCode::ACCEPTED,
        format!("server v{}", env!("CARGO_PKG_VERSION")),
    )
}
