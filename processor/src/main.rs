use std::env;
use std::error::Error;
use std::os::unix::fs::PermissionsExt;

use log::info;
use tokio::net::UnixListener;
use moonshine_processor::cmd::App;
use moonshine_processor::server;
use moonshine_processor::workers::health_check_worker::health_check_worker;
use moonshine_processor::workers::payment_worker::payment_worker;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os("RUST_LOG").is_none() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    let uds_path = env::var("UDS_PATH").unwrap_or("/tmp/moonshine-processor".to_string());
    let payment_endpoint =
        env::var("PAYMENT_ENDPOINT").unwrap_or("http://dev-server:8001".to_string());
    let payment_fallback_endpoint =
        env::var("PAYMENT_FALLBACK_ENDPOINT").unwrap_or("http://dev-server:8002".to_string());

    let app_state = App::new(payment_endpoint, payment_fallback_endpoint);

    let worker_app = app_state.clone();
    tokio::spawn(async move {
        health_check_worker(worker_app).await;
    });

    let payment_worker_app = app_state.clone();
    tokio::spawn(async move {
        payment_worker(payment_worker_app).await;
    });

    std::fs::remove_file(uds_path.clone()).ok();
    let listener = UnixListener::bind(uds_path.clone()).unwrap();

    let mut perms = std::fs::metadata(&uds_path).unwrap().permissions();
    perms.set_mode(0o666);
    std::fs::set_permissions(&uds_path, perms).unwrap();

    info!("⚗️💾moonshine-processor running at {}", uds_path);

    // Run server with graceful shutdown
    tokio::select! {
        result = server::run(listener, app_state) => {
            match result {
                Ok(_) => info!("✅ Server shutdown completed successfully"),
                Err(e) => {
                    eprintln!("❌ Server error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ = server::shutdown_signal() => {
            info!("🛑 Shutdown signal received from main");
        }
    }

    info!("👋 Moonshine Processor shutdown complete");
    Ok(())
}

