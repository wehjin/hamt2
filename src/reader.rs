use crate::base::Txid;
use iroh_docs::api::Doc;
use std::rc::Rc;

pub struct Reader {
    _doc: Rc<Doc>,
    index_root: IndexRoot,
}

impl From<Rc<Doc>> for Reader {
    fn from(doc: Rc<Doc>) -> Self {
        Reader {
            _doc: doc,
            index_root: IndexRoot {
                top_txid: Txid::FLOOR,
            },
        }
    }
}

impl Reader {
    pub fn top_txid(&self) -> Txid {
        self.index_root.top_txid
    }
}

struct IndexRoot {
    top_txid: Txid,
}
