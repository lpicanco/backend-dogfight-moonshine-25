use crate::cmd::App;

const T_CLIENT: u32 = 10_000; // 10s

pub async fn select_endpoint(app: &App) -> Result<String, String> {
    let Ok(health) = app.db.get_health_check().await else {
        return Err("Failed to retrieve health check".to_string());
    };

    let d_viable = !health.default_health_check.failing && 
        health.default_health_check.min_response_time <= T_CLIENT;

    let f_viable = !health.fallback_health_check.failing && 
        health.fallback_health_check.min_response_time <= T_CLIENT;

    match (d_viable, f_viable) {
        // Case 1: D viable → D is optimal (lower cost)
        (true, _) => Ok(app.payment_endpoint.clone()),
        
        // Case 2: D not viable, F viable → F is the only option
        (false, true) => Ok(app.payment_fallback_endpoint.clone()),
        
        // Case 3: No viable → Do not send (wait/reject)
        (false, false) => Err("No viable endpoint available - optimal action is to wait".to_string())
    }
}
