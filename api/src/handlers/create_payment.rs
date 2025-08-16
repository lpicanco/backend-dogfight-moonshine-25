use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;

use moonshine_processor::client::Pool;
use moonshine_processor::processor::Payment;

pub async fn handle(
    State(pool): State<Pool>,
    Json(payment): Json<Payment>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut conn = pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    conn.put_payment(&payment).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}
