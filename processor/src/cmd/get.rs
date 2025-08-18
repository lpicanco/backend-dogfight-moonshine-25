use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::UnixStream;

use crate::db::PaymentDb;

pub struct Get {
    start_timestamp: i64,
    end_timestamp: i64,
}

impl Get {
    pub(crate) async fn parse_data(stream: &mut BufWriter<UnixStream>) -> crate::Result<Get> {
        let start_timestamp = stream.read_i64().await?;
        let end_timestamp = stream.read_i64().await?;
        Ok(Get { start_timestamp, end_timestamp })
    }

    pub(crate) async fn execute(self, buffer: &mut BufWriter<UnixStream>, db: &PaymentDb) -> crate::Result<()> {
        let payments = db.get_payments_by_date_range(self.start_timestamp, self.end_timestamp)
            .await
            .map_err(|e| format!("Failed to get payments: {}", e))?;

        buffer.write_u16(payments.len() as u16).await?;
        buffer.write_all(payments.as_bytes()).await?;
        
        Ok(())
    }
}