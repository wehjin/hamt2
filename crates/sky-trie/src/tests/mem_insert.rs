use crate::MemView;
use crate::{InsertCursor, TrieInsert, TrieQuery, TrieValue};
use crate::{TrieSnap, TrieStream};
use futures::StreamExt;
use std::collections::HashMap;

#[tokio::test]
async fn max_u32_value_insertions_works() -> anyhow::Result<()> {
    let mut trie = MemView::new();
    trie.edit(async |edit| {
        edit.insert(100, u32::MAX).await;
        Ok(())
    })
    .await?;
    assert_eq!(trie.query_u32(100).await, Some(u32::MAX));
    Ok(())
}

#[tokio::test]
#[should_panic(expected = "assertion failed: value >= 0")]
async fn negative_key_insertions_panic() {
    let mut trie = MemView::new();
    let _ = trie
        .edit(async |edit| {
            edit.insert(-1, 100).await;
            Ok(())
        })
        .await;
}

#[tokio::test]
async fn multiple_insertions_work() {
    let mut trie = MemView::new();
    trie.edit(async |edit| {
        edit.insert(1, 1).await;
        Ok(())
    })
    .await
    .unwrap();
    let values = trie.u32_stream().collect::<HashMap<i32, u32>>().await;
    assert_eq!(values.get(&1), Some(&1));
    trie.edit(async |edit| {
        edit.insert(2, 2).await;
        assert_eq!(TrieValue::U32(1), edit.query(1).await.unwrap());
        assert_eq!(TrieValue::U32(2), edit.query(2).await.unwrap());
        Ok(())
    })
    .await
    .unwrap();
    let values = trie.u32_stream().collect::<HashMap<i32, u32>>().await;
    assert_eq!(values.get(&1), Some(&1));
    assert_eq!(values.get(&2), Some(&2));
}

#[tokio::test]
async fn snapshot_queries_work() {
    let mut trie = MemView::new();
    trie.edit(async |trie| {
        trie.insert(1, TrieValue::U32(42)).await;
        trie.insert_deep([2, 42], TrieValue::U32(242), false).await;
        Ok(())
    })
    .await
    .unwrap();

    let snap = trie.snapshot();
    assert_eq!(Some(42), snap.query_u32(1).await);
    assert_eq!(Some(TrieValue::U32(242)), snap.query_deep([2, 42]).await);
}

#[tokio::test]
async fn multi_depth_saturation_in_multiple_edits_work() {
    // Insert values past root saturation.
    let mut trie = MemView::new();
    trie.edit(async |trie| {
        trie.insert(100, TrieValue::U32(42)).await;
        for a in 0..=32 {
            trie.insert_deep([3, a], TrieValue::U32(a as u32), false)
                .await;
        }
        Ok(())
    })
    .await
    .unwrap();

    // Check insertions are present in the next edit.
    trie.edit(async |trie| {
        assert_eq!(Some(42), trie.query_u32(100).await);
        for a in 0..=32 {
            assert_eq!(
                Some(TrieValue::U32(a as u32)),
                trie.query_deep([3, a]).await
            );
        }
        Ok(())
    })
    .await
    .unwrap();

    // Deep insert values past root-block saturation at multiple sub-trie depths.
    trie.edit(async |trie| {
        // Use at least 33 keys so that the root blook in the first trie is saturated.
        for i in 0..35 {
            let e = 5 + i;
            trie.insert_deep([e, 0], TrieValue::U32(e as u32), false)
                .await;
        }
        // Use at least 33 keys so that the root block in the second trie is saturated.
        for i in 0..35 {
            let a = 3 + i;
            trie.insert_deep([4, a], TrieValue::U32(a as u32), false)
                .await;
        }
        // 3.x should be saturated.  So adding more should trigger at least on hybrid merge.
        for a in 32..=64 {
            trie.insert_deep([3, a], TrieValue::U32(a as u32), false)
                .await;
        }
        Ok(())
    })
    .await
    .unwrap();

    // Test post-commit insertions.
    let snap = trie.snapshot();
    for a in 0..=64 {
        assert_eq!(
            Some(TrieValue::U32(a as u32)),
            snap.query_deep([3, a]).await
        );
    }
}

#[tokio::test]
async fn later_insertion_overwrites_earlier_insertion() {
    let mut trie = MemView::new();
    trie.edit(async |trie| {
        trie.insert(1, 42).await.insert(1, TrieValue::U32(43)).await;
        assert_eq!(Some(43), trie.query_u32(1).await);
        Ok(())
    })
    .await
    .unwrap();
    assert_eq!(Some(43), trie.query_u32(1).await);
}

#[tokio::test]
async fn different_keys_store_different_values() {
    // 33 keys will saturate the root block.
    let keys = (0..=32).collect::<Vec<_>>();
    let mut trie = MemView::new();
    trie.edit(async |trie| {
        for i in &keys {
            trie.insert(*i, *i as u32).await;
        }
        Ok(())
    })
    .await
    .unwrap();
    let values = trie.u32_stream().collect::<HashMap<i32, u32>>().await;
    let expected = keys
        .iter()
        .map(|k| (*k, *k as u32))
        .collect::<HashMap<i32, u32>>();
    assert_eq!(values, expected);
}

#[tokio::test]
async fn deep_insertions_work_basic() {
    let mut trie = MemView::new();
    trie.edit(async |edit| {
        edit.insert_deep([2, 42], 242, false).await;
        Ok(())
    })
    .await
    .unwrap();
    assert_eq!(Some(TrieValue::U32(242)), trie.query_deep([2, 42]).await);
}

#[tokio::test]
async fn deep_insertions_work_with_saturated_root() {
    let mut trie = MemView::new();
    trie.edit(async |trie| {
        for e in 0..=33 {
            trie.insert_deep([e, e], TrieValue::U32(e as u32), false)
                .await;
        }
        Ok(())
    })
    .await
    .unwrap();
    {
        let value = trie.query_deep([4]).await;
        let Some(TrieValue::SubTrie(map_base)) = value else {
            panic!("expected map_base");
        };
        assert_eq!(1, map_base.map.slot_count());
    }
    {
        let value = trie.query_deep([4, 4]).await;
        assert_eq!(Some(TrieValue::U32(4)), value);
    }
    {
        let value = trie.query_deep([4, 1]).await;
        assert_eq!(None, value);
    }
    {
        let value = trie.query_deep([5, 2]).await;
        assert_eq!(None, value);
    }
}

#[tokio::test]
#[should_panic(expected = "assertion failed: value >= 0")]
async fn deep_queries_panic_for_invalid_key() {
    let trie = MemView::new();
    trie.query_deep([4, 4, -1]).await.unwrap();
}
