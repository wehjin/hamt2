use crate::Trie;
use sky_types::storage::FileStorage;
use sky_types::trie::{TrieInsert, TrieQuery, TrieValue};

#[tokio::test]
async fn file_trie_works() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    {
        let storage = FileStorage::new(dir.path())?;
        let mut trie = Trie::connect(storage);
        trie.insert(1, TrieValue::U32(1)).await?;
        trie.commit().await?;
    }
    {
        let storage = FileStorage::load(dir.path())?;
        let trie = Trie::connect(storage);
        let value = trie.query(1).await?;
        assert_eq!(Some(TrieValue::U32(1)), value);
    }
    Ok(())
}
