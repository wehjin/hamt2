pub mod base;
pub mod client;
pub mod hamt;
pub mod reader;

#[cfg(test)]
mod tests {
    use crate::base::{Attribute, Change, Entity, Value};
    use crate::client::Client;

    #[tokio::test]
    async fn it_works_async() -> anyhow::Result<()> {
        let mut client = Client::connect().await?;
        client.transact(&[Change::Deposit(Entity(1), Attribute(2), Value::UInt(3))])?;
        {
            let reader = client.to_reader();
            let value = reader.query_value(Entity(1), Attribute(2));
            assert_eq!(Some(Value::UInt(3)), value);
        }
        Ok(())
    }
}
