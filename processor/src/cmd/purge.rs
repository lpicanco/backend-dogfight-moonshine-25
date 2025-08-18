use crate::cmd::App;
use crate::payment_client;

pub struct Purge {}

impl Purge {
    pub(crate) async fn execute(self, app: &App) -> crate::Result<()> {
        payment_client::purge(app).await.ok();
        app.db.clear().await?;
        Ok(())
    }
}