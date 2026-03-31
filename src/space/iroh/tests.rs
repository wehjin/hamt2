use crate::space::iroh::client::IrohClient;

#[tokio::test]
async fn iroh_space_works() -> anyhow::Result<()> {
    let client = IrohClient::new().await?;
    Ok(())
}
