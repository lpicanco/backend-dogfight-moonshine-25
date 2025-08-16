use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Utc};
use moonshine_processor::client::Pool;
use std::collections::HashMap;

pub async fn handle(
    State(pool): State<Pool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let from = parse_date(params.get("from"), "2025-01-01T00:00:00Z")?;
    let to = parse_date(params.get("to"), "2030-12-01T00:00:00Z")?;

    let mut conn = pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = conn.get_payments_by_date_range(from, to).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::OK,
        [("content-type", "application/json")],
        result
    ))
}

fn parse_date(param: Option<&String>, default: &str) -> Result<DateTime<Utc>, StatusCode> {
    let param_value = param.map(|s| s.as_str()).unwrap_or(default);
    let date = DateTime::parse_from_rfc3339(param_value)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);
    Ok(date)
}