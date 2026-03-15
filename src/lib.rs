pub mod client;
pub mod hamt;
pub mod reader;

#[cfg(test)]
mod tests {
    use crate::client::Client;
    use crate::hamt::base::{Attr, Change, Ent, Value};

    #[tokio::test]
    async fn it_works_async() -> anyhow::Result<()> {
        let mut client = Client::connect().await?;
        client.transact(&[Change::Deposit(Ent(1), Attr(2), Value::UInt(3))])?;
        {
            let reader = client.to_reader();
            let value = reader.query_value(Ent(1), Attr(2));
            assert_eq!(Some(Value::UInt(3)), value);
        }
        Ok(())
    }
}
