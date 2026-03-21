pub mod client;
pub mod core;
pub mod reader;

#[cfg(test)]
mod tests {
    use crate::iroh_db::client::Client;
    use crate::iroh_db::core::{Attr, Datom, Ent, Tx, Val};
    #[tokio::test]
    async fn client_works_async() {
        let mut client = Client::connect().await.expect("connect");

        static AT_COUNT: Attr = Attr("count");
        static AT_PRICE: Attr = Attr("price");
        static AT_RANK: Attr = Attr("rank");

        {
            let reader = client.to_reader();
            let db = reader.query_db().await.expect("query");
            assert_eq!(db.max_ent, Ent(0));
            assert_eq!(db.max_tx, Tx(0));
        }

        client
            .transact(vec![
                Datom::Add(Ent(1), AT_COUNT, Val::Uint(42)),
                Datom::Id(
                    Ent(2),
                    vec![(AT_PRICE, Val::Uint(78)), (AT_RANK, Val::Uint(1))],
                ),
            ])
            .await
            .expect("transact");
        {
            let reader = client.to_reader();
            let db = reader.query_db().await.expect("query");
            assert_eq!(db.max_ent, Ent(2));
            assert_eq!(db.max_tx, Tx(1));
            let count = AT_COUNT.query_val(Ent(1), &reader).await.expect("query");
            let price = AT_PRICE.query_val(Ent(2), &reader).await.expect("query");
            let rank = AT_RANK.query_val(Ent(2), &reader).await.expect("query");
            assert_eq!(Some(Val::Uint(42)), count);
            assert_eq!(Some(Val::Uint(78)), price);
            assert_eq!(Some(Val::Uint(1)), rank);
        }
    }
}
