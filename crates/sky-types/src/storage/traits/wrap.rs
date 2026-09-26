//
// pub trait InnerStoreView: Deref<Target = Self::Inner> {
//     type Inner: StoreView;
// }
//
// impl<T: InnerStoreView> TrieQuery for T {
//     async fn query(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError> {
//         self.deref().query(key).await
//     }
//
//     async fn query_u32(&self, key: i32) -> Result<Option<u32>, TrieQueryError> {
//         self.deref().query_u32(key).await
//     }
//
//     async fn query_all(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
//         self.deref().query_all().await
//     }
//
//     async fn deep_query<const N: usize>(
//         &self,
//         key: [i32; N],
//     ) -> Result<Option<TrieValue>, TrieQueryError> {
//         self.deref().deep_query(key).await
//     }
// }
//
// impl<T: InnerStoreView> TrieSnap for T {
//     type Snapshot = T::Inner;
//
//     fn with_new_root(self, new_root: Option<MapBase>) -> Self {
//         //self.deref().snapshot().with_new_root(new_root)
//     }
//
//     fn snapshot(&self) -> Self::Snapshot {
//         self.deref().snapshot()
//     }
// }
