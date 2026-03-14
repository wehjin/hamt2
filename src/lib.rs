pub mod base;
pub mod client;
pub mod reader;

#[cfg(test)]
mod tests {
	use crate::base::Txid;
	use crate::client::Client;

	#[tokio::test]
    async fn it_works_async() -> anyhow::Result<()> {
        let client = Client::connect().await?;
        let addr = client.to_endpoint_addr();
        dbg!(&addr);
        let reader = client.to_reader();
        assert_eq!(Txid::FLOOR, reader.top_txid());
        Ok(())
    }
}
