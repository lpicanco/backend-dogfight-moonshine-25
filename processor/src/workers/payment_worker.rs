use crate::cmd::App;
use crate::processor::Payment;
use crate::workers::endpoint_selector::select_endpoint;
use crate::{db, payment_client, PaymentType};
use async_channel::Receiver;
use log::{debug, error, warn};
use std::time::Duration;
use chrono::SubsecRound;
use tokio::time::sleep;

const WORKER_COUNT: usize = 3;

pub async fn payment_worker(app: App) {    
    for i in 0..WORKER_COUNT {
        let worker_app = app.clone();
        let worker_rx = app.payment_receiver.clone();
        tokio::spawn(async move {
            payment_processor_worker(worker_app, worker_rx, i).await;
        });
    }
    ()
}

async fn payment_processor_worker(app: App, rx: Receiver<Payment>, worker_id: usize) {
    debug!("Payment processor worker {} started", worker_id);

    loop {
        let Ok(payment) = rx.recv().await else {
            error!("Payment processor worker {} shutting down", worker_id);
            break;
        };

        loop {
            match process_payment(&app, &payment).await {
                Ok(_) => {
                    break;
                }
                Err(_) => {
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }
            }
        }
    }
}

async fn process_payment(app: &App, payment: &Payment) -> Result<(), String> {
    debug!("Processing payment: {:?}", payment);

    let endpoint = select_endpoint(&app).await?;
    let created_at = chrono::Utc::now().round_subsecs(0);
    let result = payment_client::create_payment(&app, &endpoint, payment, &created_at).await;
    if let Err(e) = result {
        if e.status() == Some(reqwest::StatusCode::UNPROCESSABLE_ENTITY) {
            warn!("Payment already exists: {}", e);
            return Ok(());
        }

        if e.status() != Some(reqwest::StatusCode::INTERNAL_SERVER_ERROR) {
            error!("Failed to create payment: {}", e);
        }

        return Err(e.to_string());
    }

    let payment_type = if endpoint == app.payment_endpoint {
        PaymentType::Default
    } else {
        PaymentType::Fallback
    };

    let payment_db = db::Payment {
        amount: payment.amount,
        requested_at: created_at.timestamp_millis(),
        payment_type
    };

    app.db.insert(payment_db).await.map_err(|e| e.to_string())?;
    Ok(())
}
