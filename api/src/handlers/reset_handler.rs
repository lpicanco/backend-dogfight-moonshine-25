use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use moonshine_processor::client::Pool;

pub async fn handle(
    State(pool): State<Pool>,
) -> Result<impl IntoResponse, StatusCode> {

    let mut conn = pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    conn.purge().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}
