pub mod txid;

pub struct Db {}
impl Db {
    pub fn new() -> Self {
        Self {}
    }
    pub fn transact(self) -> Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn db_works() {
        let db = Db::new();
        db.transact();
    }
}
