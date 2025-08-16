use std::time::Duration;
use chrono::DateTime;
use log::debug;
use serde::{Deserialize, Serialize};
use crate::cmd::App;
use crate::{HealthCheck, HealthCheckResult};
use crate::processor::Payment;

pub async fn health_check(app: &App) -> crate::Result<HealthCheckResult> {
    let default_health_check = health_check_endpoint(app, &app.payment_endpoint).await?;
    let fallback_health_check = health_check_endpoint(app, &app.payment_fallback_endpoint).await?;

    Ok(HealthCheckResult {
        default_health_check,
        fallback_health_check,
    })
}

async fn health_check_endpoint(app: &App, endpoint: &str) -> Result<HealthCheck, reqwest::Error> {
    app.http_client
        .get(format!("{}/payments/service-health", endpoint))
        .send()
        .await?
        .json::<HealthCheck>()
        .await
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PaymentDto {
    #[serde(rename = "correlationId")]
    pub correlation_id: String,
    pub amount: f64,
    #[serde(rename = "requestedAt")]
    pub requested_at: String,
}

pub async fn create_payment(
    app: &App,
    endpoint: &str,
    payment: &Payment,
    date: &DateTime<chrono::Utc>,
) -> Result<(), reqwest::Error> {
    // TODO: Use a more sophisticated timeout strategy based on the endpoint
    let timeout = if endpoint.contains("fallback") {
        Duration::from_secs(10)
    } else {
        Duration::from_secs(10)
    };

    let payment = PaymentDto {
        correlation_id: payment.correlation_id.clone(),
        amount: payment.amount,
        requested_at: date.to_rfc3339(),
    };

    app.http_client
        .post(format!("{}/payments", endpoint))
        .timeout(timeout)
        .json(&payment)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn purge(app: &App) -> Result<(), reqwest::Error> {
    purge_endpoint(app, &app.payment_endpoint).await?;
    purge_endpoint(app, &app.payment_fallback_endpoint).await?;

    Ok(())
}

async fn purge_endpoint(app: &App, endpoint: &str) -> Result<(), reqwest::Error> {
    app.http_client
        .post(format!("{}/admin/purge-payments", endpoint))
        .header("X-Rinha-Token", "123")
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
