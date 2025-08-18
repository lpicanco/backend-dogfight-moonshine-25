use std::sync::Arc;
use tokio::io::{BufWriter};
use tokio::net::UnixStream;
use async_channel::{Receiver, Sender};

pub use put::Put;
pub use get::Get;
pub use purge::Purge;

use crate::db::PaymentDb;
use crate::processor::Payment;

mod put;
mod get;
mod purge;

pub enum Command {
    Put(Put),
    Get(Get),
    Purge(Purge),
}


pub(crate) const CMD_PUT_OPCODE: u8 = 42;
pub(crate) const CMD_GET_OPCODE: u8 = 43;
pub(crate) const CMD_PURGE_OPCODE: u8 = 44;

impl Command {
    pub(crate) async fn execute(
        self,
        buffer: &mut BufWriter<UnixStream>,
        app: &App,
    ) -> crate::Result<()> {
        match self {
            Command::Put(cmd) => cmd.execute(&app.payment_sender).await,
            Command::Get(cmd) => cmd.execute(buffer, &app.db).await,
            Command::Purge(cmd) => cmd.execute(&app).await,
        }
    }
    pub(crate) async fn from_data(cmd: u8, data: &mut BufWriter<UnixStream>) -> crate::Result<Command> {

        let command = match cmd {
            CMD_PUT_OPCODE => Command::Put(Put::parse_data(data).await?),
            CMD_GET_OPCODE => Command::Get(Get::parse_data(data).await?),
            CMD_PURGE_OPCODE => Command::Purge(Purge { }),
            _ => return Err(format!("Unknown command: {}", cmd).into()),
        };

        Ok(command)
    }
}

#[derive(Clone)]
pub struct App {
    pub http_client: reqwest::Client,
    pub payment_endpoint: String,
    pub payment_fallback_endpoint: String,
    pub db: Arc<PaymentDb>,
    pub payment_sender: Sender<Payment>,
    pub payment_receiver: Receiver<Payment>,
}

impl App {
    pub fn new(
        payment_endpoint: String,
        payment_fallback_endpoint: String,
    ) -> Self {
        let (tx,rx) = async_channel::unbounded();
        App {
            http_client: reqwest::Client::new(),
            payment_endpoint,
            payment_fallback_endpoint,
            db: Arc::new(PaymentDb::new()),
            payment_sender: tx,
            payment_receiver: rx,
        }
    }
}