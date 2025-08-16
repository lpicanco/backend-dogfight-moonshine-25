use crate::cmd::App;
use crate::payment_client;
use log::error;
use std::time::Duration;
use tokio::time::sleep;

pub async fn health_check_worker(app: App) {
    loop {
        let health = match payment_client::health_check(&app).await {
            Ok(health) => health,
            Err(e) => {
                error!("Health check failed: {}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        app.db.set_health_check(health).await.unwrap_or_else(|e| {
            error!("Failed to set health check in database: {}", e);
        });
        sleep(Duration::from_secs(5)).await;
    }
}
