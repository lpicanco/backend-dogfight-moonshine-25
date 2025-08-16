pub use cmd::Command;
use serde::{Deserialize, Serialize};

pub mod server;
pub mod db;
pub mod payment_client;

pub mod cmd;
pub mod processor;
pub mod workers;
pub mod client;

pub const MAX_CONNECTIONS: usize = 2048;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentType {
    Default = 0,
    Fallback = 1,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HealthCheck {
    pub failing: bool,
    #[serde(rename = "minResponseTime")]
    pub min_response_time: u32,
}
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub default_health_check: HealthCheck,
    pub fallback_health_check: HealthCheck,
}