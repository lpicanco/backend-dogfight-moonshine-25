use deadpool::managed;
use deadpool::managed::{Metrics, RecycleError};
use log::warn;

use crate::client::ProcessorClient;
use crate::Error;

#[derive(Debug)]
pub struct Manager(String);
pub type Pool = managed::Pool<Manager>;

impl Manager {
    pub fn new<S: Into<String>>(uds_path: S) -> Self {
        Self(uds_path.into())
    }
}

impl managed::Manager for Manager {
    type Type = ProcessorClient;
    type Error = Error;

    async fn create(&self) -> Result<ProcessorClient, Error> {
        let client = ProcessorClient::connect(&self.0).await?;
        Ok(client)
    }

    async fn recycle(
        &self,
        conn: &mut ProcessorClient,
        _: &Metrics,
    ) -> managed::RecycleResult<Error> {

        if conn.is_closed() {
            warn!("Recycling client");
            return Err(RecycleError::Message(
                "Connection is closed. Connection is considered unusable.".into(),
            ));
        }

        Ok(())
    }
}
