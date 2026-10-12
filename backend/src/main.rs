mod auth;
mod config;
mod couch;
mod error;

use std::time::Duration;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub couch: couch::Couch,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("taller_api=info,tower_http=info")),
        )
        .init();

    let cfg = config::Config::from_env();
    let couch = couch::Couch::new(&cfg);

    // CouchDB puede tardar unos segundos en estar listo: se reintenta
    const INTENTOS: u32 = 15;
    for intento in 1..=INTENTOS {
        match couch.init().await {
            Ok(_) => {
                tracing::info!("CouchDB listo, base '{}' disponible", cfg.couch_db);
                break;
            }
            Err(e) if intento < INTENTOS => {
                tracing::warn!("CouchDB no responde ({intento}/{INTENTOS}): {e:?}");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => panic!("No se pudo inicializar CouchDB: {e:?}"),
        }
    }

    let state = AppState {
        couch,
        jwt_secret: cfg.jwt_secret.clone(),
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/me", get(auth::me))
        .layer(CorsLayer::permissive()) // solo para desarrollo
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr)
        .await
        .expect("No se pudo abrir el puerto de la API");
    tracing::info!("API escuchando en {}", cfg.bind_addr);
    axum::serve(listener, app).await.expect("Error en el servidor");
}
