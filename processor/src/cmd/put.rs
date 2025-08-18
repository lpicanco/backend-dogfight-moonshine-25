use tokio::io::{AsyncReadExt, BufWriter};
use tokio::net::UnixStream;
use async_channel::Sender;

use crate::processor::Payment;

pub struct Put {
    payment: Payment,
}

impl Put {
    pub(crate) async fn parse_data(stream: &mut BufWriter<UnixStream>) -> crate::Result<Put> {
        let data_size = stream.read_u16().await?;
        let mut data = vec![0; data_size as usize];
        stream.read_exact(&mut data).await?;
        
        let payment = bincode::decode_from_slice(&data, bincode::config::standard())
            .map_err(|e| format!("Failed to deserialize payment: {}", e))?
            .0;
        
        Ok(Put { payment })
    }

    pub(crate) async fn execute(self, payment_sender: &Sender<Payment>) -> crate::Result<()> {
        payment_sender.send(self.payment.clone()).await
            .map_err(|e| format!("Failed to send payment to channel: {}", e))?;
        
        log::debug!("Sent payment to channel: amount: {}", self.payment.amount);
        Ok(())
    }
}