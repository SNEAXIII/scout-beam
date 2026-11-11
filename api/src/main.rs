//! Process entry point: read the environment, open the pool, migrate, serve.

use std::net::SocketAddr;

use anyhow::{Context, Result};
use scout_beam::config::Environment;
use scout_beam::shared::db;
use scout_beam::{AppState, Config, build_app};
use tokio::net::TcpListener;
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // A missing .env is fine in production, where the environment is injected.
    let _ = dotenvy::dotenv();

    // Configuration first: what gets logged depends on the environment.
    let config = Config::from_env()?;
    init_tracing(config.environment);

    tracing::info!(env = ?config.environment, "starting scout-beam");

    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool).await?;

    let bind_addr = config.bind_addr;
    let is_dev = config.environment.is_dev();
    let app = build_app(AppState::new(pool, config))?;

    let listener = TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("cannot bind {bind_addr}"))?;
    tracing::info!(%bind_addr, "listening");

    if is_dev {
        let host = reachable_host(bind_addr);
        tracing::info!("swagger ui: http://{host}/docs");
        tracing::info!("openapi:    http://{host}/api-docs/openapi.json");
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")
}

/// `0.0.0.0` is a bind address, not somewhere a browser can go.
fn reachable_host(bind_addr: SocketAddr) -> String {
    if bind_addr.ip().is_unspecified() {
        format!("localhost:{}", bind_addr.port())
    } else {
        bind_addr.to_string()
    }
}

/// `RUST_LOG` wins when it is set; otherwise the environment decides.
fn init_tracing(environment: Environment) {
    let default = if environment.is_dev() {
        "scout_beam=debug,tower_http=debug,info"
    } else {
        "scout_beam=info,warn"
    };

    // An empty `RUST_LOG` is far more often an accident than a wish for silence.
    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .init();
}

/// Stop accepting connections on Ctrl-C or SIGTERM, and let in-flight requests finish.
async fn shutdown_signal() {
    // A handler that cannot be installed is worth a loud log, not a crash.
    let ctrl_c = async {
        if let Err(error) = signal::ctrl_c().await {
            tracing::error!(%error, "cannot listen for Ctrl-C");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "cannot listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("shutting down");
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn an_unspecified_bind_address_is_printed_as_localhost() {
        let bound: SocketAddr = "0.0.0.0:8002".parse().unwrap();

        assert_eq!(reachable_host(bound), "localhost:8002");
    }

    #[test]
    fn a_concrete_address_is_printed_as_is() {
        let bound: SocketAddr = "127.0.0.1:9000".parse().unwrap();

        assert_eq!(reachable_host(bound), "127.0.0.1:9000");
    }
}
