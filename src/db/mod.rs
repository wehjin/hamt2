pub mod txid;

pub struct Db {}
impl Db {
    pub fn new() -> Self {
        Self {}
    }
    pub fn size(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
	use super::*;
	#[tokio::test]
    async fn db_works() {
        let db = Db::new();
        assert_eq!(0, db.size());
    }
}
