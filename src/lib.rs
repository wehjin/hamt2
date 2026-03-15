pub mod base;
pub mod client;
pub mod hamt;
pub mod reader;

#[cfg(test)]
mod tests {
    use crate::base::{Attr, Datom, Ent, Val};
    use crate::client::Client;

    #[tokio::test]
    async fn it_works_async() {
        let mut client = Client::connect().await.expect("connect");

        static COUNT_ATTR: Attr = Attr("name");

        client
            .transact(&[Datom::Add(Ent(1), COUNT_ATTR, Val::UInt(42))])
            .await
            .expect("transact");
        {
            let value = client.query_value(Ent(1), COUNT_ATTR).await.expect("query");
            assert_eq!(Some(Val::UInt(42)), value);
        }
    }
}
