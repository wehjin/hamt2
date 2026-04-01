use crate::space::doc::client::DocsClient;
use iroh::SecretKey;
use iroh_docs::store::Query;
use iroh_docs::NamespaceId;

#[tokio::test]
async fn docs_client_works() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let secret_key = SecretKey::from_bytes(&[0x01u8; 32]);
    let doc_id: NamespaceId;
    {
        let client = DocsClient::connect(&temp_dir, secret_key.clone()).await?;
        let doc = client.docs.create().await?;
        doc_id = doc.id();
        doc.set_bytes(client.author, "key", "value").await?;
        client.router.shutdown().await?;
    }
    {
        let client = DocsClient::connect(&temp_dir, secret_key).await?;
        let doc = client.docs.open(doc_id).await?.expect("doc should exist");
        let entry = doc
            .get_one(Query::key_exact("key"))
            .await?
            .expect("entry should exist");
        assert_eq!("key".as_bytes(), entry.key());
        client.router.shutdown().await?;
    }
    Ok(())
}
