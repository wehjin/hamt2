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

        static AT_COUNT: Attr = Attr("name");

        client
            .transact(&[Datom::Add(Ent(1), AT_COUNT, Val::UInt(42))])
            .await
            .expect("transact");
        {
            let reader = client.to_reader();
            let value = AT_COUNT.query_val(Ent(1), &reader).await.expect("query");
            assert_eq!(Some(Val::UInt(42)), value);
        }
    }
}
