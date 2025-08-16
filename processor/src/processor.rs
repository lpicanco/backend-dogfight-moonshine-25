use bincode::{Decode, Encode};
use serde::Deserialize;

#[derive(Clone, Encode, Decode, Debug, Deserialize)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    pub correlation_id: String,
    pub amount: f64,
}
