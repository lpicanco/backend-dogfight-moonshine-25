use crate::{HealthCheck, HealthCheckResult, PaymentType};
use std::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Payment {
    pub amount: f64,
    pub requested_at: i64,
    pub payment_type: PaymentType,
}

pub struct PaymentDb {
    payments: RwLock<Vec<Payment>>,
    health: RwLock<HealthCheckResult>,
}

impl PaymentDb {
    pub fn new() -> Self {
        Self {
            payments: RwLock::new(Vec::with_capacity(60_000)),
            health: RwLock::new(HealthCheckResult {
                default_health_check: HealthCheck {
                    failing: false,
                    min_response_time: 0,
                },
                fallback_health_check: HealthCheck {
                    failing: false,
                    min_response_time: 0,
                },
            }),
        }
    }

    pub async fn set_health_check(&self, health_check: HealthCheckResult) -> crate::Result<()> {
        let mut health = self.health.write().map_err(|_| "Failed to acquire health lock")?;
        *health = health_check;
        Ok(())
    }

    pub async fn get_health_check(&self) -> crate::Result<HealthCheckResult> {
        let health = self.health.read().map_err(|_| "Failed to acquire health lock")?;
        Ok(health.clone())
    }

    pub async fn insert(&self, payment: Payment) -> Result<(), String> {
        let mut payments = self.payments.write().map_err(|_| "Failed to acquire payments lock")?;
        payments.push(payment);

        Ok(())
    }

    pub async fn get_payments_by_date_range(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<String, String> {
        let payments = self.payments.read().map_err(|_| "Failed to acquire payments lock")?;
        let filtered: Vec<Payment> = payments
            .iter()
            .filter(|payment| {
                payment.requested_at >= start_timestamp && payment.requested_at <= end_timestamp
            })
            .cloned()
            .collect();

        let default_payments: Vec<_> = filtered.iter()
            .filter(|payment| payment.payment_type == PaymentType::Default)
            .map(|payment| payment.amount)
            .collect();

        let fallback_payments: Vec<_> = filtered.iter()
            .filter(|payment| payment.payment_type == PaymentType::Fallback)
            .map(|payment| payment.amount)
            .collect();

        let default_summary = default_payments.len();
        let fallback_summary = fallback_payments.len();

        let total_amount_default = default_payments.iter().sum::<f64>().abs();
        let total_amount_fallback = fallback_payments.iter().sum::<f64>().abs();

        let response = format!(
            r#"{{"default":{{"totalRequests":{},"totalAmount":{:.2}}},"fallback":{{"totalRequests":{},"totalAmount":{:.2}}}}}"#,
            default_summary, total_amount_default, fallback_summary, total_amount_fallback
        );

        Ok(response)
    }

    pub async fn clear(&self) -> Result<(), String> {
        let mut payments = self.payments.write().map_err(|_| "Failed to acquire payments lock")?;
        payments.clear();
        Ok(())
    }
}