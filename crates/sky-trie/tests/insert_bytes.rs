use sky_trie::SkyKvs;
use sky_trie::{Insert, Query, TrieValue};

#[tokio::test]
async fn sequential_lengths() {
    let tests = [
        (1, &[7u8; 1] as &[u8]),
        (2, &[7u8; 2] as &[u8]),
        (3, &[7u8; 3] as &[u8]),
        (4, &[7u8; 4] as &[u8]),
        (5, &[7u8; 5] as &[u8]),
        (6, &[7u8; 6] as &[u8]),
        (7, &[7u8; 7] as &[u8]),
        (8, &[7u8; 8] as &[u8]),
        (9, &[7u8; 9] as &[u8]),
        (10, &[7u8; 10] as &[u8]),
        (11, &[7u8; 11] as &[u8]),
        (12, &[7u8; 12] as &[u8]),
        (13, &[7u8; 13] as &[u8]),
    ];
    assert_byte_tests(tests).await;
}

#[tokio::test]
async fn long_short_lengths() {
    let tests = [
        (0, &[7u8; 1] as &[u8]),
        (1, &[7u8; 3] as &[u8]),
        (2, &[7u8; 9] as &[u8]),
        (3, &[7u8; 81] as &[u8]),
        (4, &[7u8; 243] as &[u8]),
        (5, &[7u8; 729] as &[u8]),
        (6, &[7u8; 2187] as &[u8]),
        (7, &[7u8; 6561] as &[u8]),
        (8, &[7u8; 19783] as &[u8]),
    ];
    assert_byte_tests(tests).await;
}

#[tokio::test]
async fn lengths_around_dword_boundaries() {
    let tests = [
        (0, &[7u8; 6] as &[u8]),
        (1, &[7u8; 7] as &[u8]),
        (2, &[7u8; 8] as &[u8]),
        (3, &[7u8; 9] as &[u8]),
        //
        (4, &[7u8; 14] as &[u8]),
        (5, &[7u8; 15] as &[u8]),
        (6, &[7u8; 16] as &[u8]),
        (7, &[7u8; 17] as &[u8]),
        //
        (8, &[7u8; 30] as &[u8]),
        (9, &[7u8; 31] as &[u8]),
        (10, &[7u8; 32] as &[u8]),
        (11, &[7u8; 33] as &[u8]),
    ];
    assert_byte_tests(tests).await;
}

async fn assert_byte_tests<const N: usize>(tests: [(i32, &[u8]); N]) {
    let mut kvs = SkyKvs::new();
    kvs.edit(async |kvs| {
        for i in 0..tests.len() {
            let (id, value) = tests[i];
            let key = 100 + id;
            kvs.insert(key, value).await;
        }
        Ok(())
    })
    .await
    .unwrap();
    for i in 0..tests.len() {
        let (id, value) = tests[i];
        let key = 100 + id;
        let TrieValue::Bytes(bytes) = kvs.query(key).await.unwrap() else {
            panic!("value should be bytes");
        };
        assert_eq!(&bytes, value)
    }
}
