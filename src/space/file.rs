use crate::hamt::trie::mem::slot::MemSlot;
use crate::space::mem::MemSegment;
use crate::space::seg::Seg;
use crate::space::table::TableRoot;
use crate::space::{Reader, Space, Value};
use crate::{ReadError, TransactError};
use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Red database error: {0:?}")]
    RedDatabase(#[from] redb::DatabaseError),

    #[error("Red transaction error: {0:?}")]
    RedTransaction(#[from] redb::TransactionError),

    #[error("Red table error: {0:?}")]
    RedTable(#[from] redb::TableError),

    #[error("Red storage error: {0:?}")]
    RedStorage(#[from] redb::StorageError),

    #[error("Red commit error: {0:?}")]
    RedCommit(#[from] redb::CommitError),
}

pub struct FileSpace {
    db: Database,
    max_seg: Seg,
}

const COUNTS_TABLE: TableDefinition<&str, u32> = TableDefinition::new("counts");
const SEGMENTS_TABLE: TableDefinition<u32, Vec<u8>> = TableDefinition::new("segments");

impl FileSpace {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let db = Database::create(path)?;
        let max_seg = Seg(0);
        let seg = Seg(0);
        let write = db.begin_write()?;
        {
            let mut counts = write.open_table(COUNTS_TABLE)?;
            counts.insert("segments", seg.0)?;
        }
        write.commit()?;
        ();
        Ok(Self { db, max_seg })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let db = Database::open(path)?;
        let max_seg = {
            let read = db.begin_read()?;
            let table = read.open_table(COUNTS_TABLE)?;
            let seg = table.get("segments")?.expect("get segment count").value();
            Seg(seg)
        };
        Ok(Self { db, max_seg })
    }
}

impl Space for FileSpace {
    fn max_seg(&self) -> Seg {
        self.max_seg
    }

    fn add_segment(
        &mut self,
        seg: Seg,
        values: Vec<Value>,
        table: Vec<MemSlot>,
        root: Option<TableRoot>,
    ) -> Result<(), TransactError> {
        if seg != self.max_seg {
            return Err(TransactError::SegConflict(seg));
        }
        let write = self.db.begin_write().expect("begin write");
        {
            let segment = FileSegment {
                values,
                table,
                root,
            };
            let bytes = postcard::to_allocvec(&segment).expect("serialize segment");
            let mut segments = write
                .open_table(SEGMENTS_TABLE)
                .expect("open segments table");
            segments.insert(seg.0, bytes).expect("insert segment");
        }
        {
            self.max_seg = seg + 1;
            let mut counts = write.open_table(COUNTS_TABLE).expect("open counts table");
            counts
                .insert("segments", self.max_seg.0)
                .expect("insert segment count");
        }
        write.commit().expect("commit");
        Ok(())
    }

    fn read(&self) -> Result<Reader, ReadError> {
        // For now, we just read the whole database.
        let mut segments = Vec::new();
        for seg in 0..self.max_seg.0 {
            let bytes = {
                let read = self.db.begin_read().expect("begin read");
                let table = read
                    .open_table(SEGMENTS_TABLE)
                    .expect("open segments table");
                table
                    .get(seg)
                    .expect("get segment")
                    .expect("get segment bytes")
                    .value()
            };
            let segment: FileSegment = postcard::from_bytes(&bytes).expect("deserialize segment");
            let mem_segment = MemSegment {
                values: segment.values,
                table: segment.table,
                root: segment.root,
            };
            segments.push(Rc::new(mem_segment));
        }
        let reader = Reader::new(segments);
        Ok(reader)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FileSegment {
    values: Vec<Value>,
    table: Vec<MemSlot>,
    root: Option<TableRoot>,
}
