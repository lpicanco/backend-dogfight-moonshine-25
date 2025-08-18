use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::UnixStream;
use chrono::{DateTime, Utc};
use std::path::Path;

use crate::processor::Payment;

pub struct ProcessorClient {
    stream: BufWriter<UnixStream>,
}

impl ProcessorClient {
    pub async fn connect<P: AsRef<Path>>(uds_path: P) -> crate::Result<ProcessorClient> {
        let socket = UnixStream::connect(uds_path).await?;
        let stream = BufWriter::new(socket);
        Ok(ProcessorClient { stream })
    }

    pub async fn purge(&mut self) -> crate::Result<()> {
        self.stream.write_u8(crate::cmd::CMD_PURGE_OPCODE).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn put_payment(&mut self, payment: &Payment) -> crate::Result<()> {
        self.stream.write_u8(crate::cmd::CMD_PUT_OPCODE).await?;
        
        let serialized = bincode::encode_to_vec(payment, bincode::config::standard())
            .map_err(|e| format!("Failed to serialize payment: {}", e))?;
        
        self.stream.write_u16(serialized.len() as u16).await?;
        self.stream.write_all(&serialized).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn get_payments_by_date_range(
        &mut self, 
        start_date: DateTime<Utc>, 
        end_date: DateTime<Utc>
    ) -> crate::Result<String> {
        self.stream.write_u8(crate::cmd::CMD_GET_OPCODE).await?;
        
        self.stream.write_i64(start_date.timestamp_millis()).await?;
        self.stream.write_i64(end_date.timestamp_millis()).await?;
        self.stream.flush().await?;
        
        let response_len = self.stream.read_u16().await?;
        let mut response = vec![0; response_len as usize];
        self.stream.read_exact(&mut response).await?;
        
        String::from_utf8(response)
            .map_err(|e| format!("Failed to parse response as UTF-8: {}", e).into())
    }

    pub fn is_closed(&self) -> bool {
        self.stream.get_ref().peer_cred().is_err()
    }
}