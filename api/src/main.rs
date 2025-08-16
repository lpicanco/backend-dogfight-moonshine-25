mod handlers;

use std::env;
use std::os::unix::fs::PermissionsExt;

use axum::routing::post;
use axum::{routing::get, Router};
use deadpool::Runtime;
use tokio::net::{TcpListener, UnixListener};
use tokio::signal;

use moonshine_processor::client::{Manager, Pool};
use crate::handlers::{create_payment, get_payments_summary, reset_handler};

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    let processor_uds_path = env::var("PROCESSOR_UDS_PATH").unwrap_or("/tmp/moonshine-processor".to_string());
    let manager = Manager::new(processor_uds_path);
    let pool = Pool::builder(manager)
        .max_size(10)
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap();

    let app = Router::new()
        .route("/payments", post(create_payment::handle))
        .route("/payments-summary", get(get_payments_summary::handle))
        .route("/purge-payments", post(reset_handler::handle))
        .with_state(pool);

    let uds_path = env::var("UDS_PATH").unwrap_or("/tmp/moonshine-api".to_string());
    std::fs::remove_file(uds_path.clone()).ok();
    let listener = UnixListener::bind(uds_path.clone()).unwrap();
    // let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", 8085)).await.unwrap();

    let mut perms = std::fs::metadata(&uds_path).unwrap().permissions();
    perms.set_mode(0o666);
    std::fs::set_permissions(&uds_path, perms).unwrap();

    println!("⚗️🥂moonshine-api running at http://localhost:{}/", uds_path);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
