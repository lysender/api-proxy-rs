use std::time::Duration;

use axum::extract::FromRef;
use axum::http::header::{ACCEPT, AUTHORIZATION};
use axum::http::Method;
use axum::Router;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::info;
use tracing::Level;

use crate::config::Config;
use crate::error::Result;
use crate::proxy::routes_proxy;
use crate::web::{routes_fallback, routes_index};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Config,
}

pub async fn run(config: Config) -> Result<()> {
    let state = AppState {
        config: config.clone(),
    };

    let mut routes_all = Router::new()
        .merge(routes_index(state.clone()))
        .merge(routes_proxy(state.clone()))
        .fallback_service(routes_fallback(state))
        .layer(
            ServiceBuilder::new().layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            ),
        );

    if config.cors {
        let cors = CorsLayer::very_permissive();
        routes_all = routes_all.layer(cors);
    }

    // Setup the server
    let ip = "127.0.0.1";
    let addr = format!("{}:{}", ip, config.port);
    info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}