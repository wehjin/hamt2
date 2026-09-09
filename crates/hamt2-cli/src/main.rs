use hamt2::db::find::{AnyAttrIgnore, Find};
use hamt2::db::{datom, val, Attr, Db};
use hamt2::space::mem::MemSpace;

const ATTR_COUNT: Attr = Attr("counter/count");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = Db::new(MemSpace::new(), [ATTR_COUNT]).await?;
    let db = db
        .transact([
            datom::add(1, ATTR_COUNT, val(10)),
            datom::add(2, ATTR_COUNT, val(20)),
            datom::add(3, ATTR_COUNT, val(30)),
        ])
        .await?;

    let mut eins = AnyAttrIgnore::new(ATTR_COUNT).apply_db(&db).await?;
    eins.sort();

    for ein in eins {
        let count = db.find_val(ein, ATTR_COUNT).await?;
        println!("entity {:?} -> {:?}", ein, count);
    }

    Ok(())
}
