use hamt2::db::Attr;

pub const ATTR_VERSION: Attr = Attr("skybase/version");
pub const VERSION_ENT: i32 = 1;

#[derive(Debug, Clone)]
pub struct DbState {
    pub skybase_version: String,
}
