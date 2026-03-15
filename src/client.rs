use crate::base::Change;
use crate::reader::Reader;
use crate::trie::key::TrieKey;
use iroh::endpoint::BindError;
use iroh::protocol::Router;
use iroh::{Endpoint, EndpointAddr};
use iroh_blobs::store::mem::MemStore;
use iroh_blobs::BlobsProtocol;
use iroh_docs::api::Doc;
use iroh_docs::protocol::Docs;
use iroh_gossip::Gossip;
use std::rc::Rc;

pub struct Loader {
    pub doc: Rc<Doc>,
    pub segment: Option<Rc<Segment>>,
    pub root_start: Option<usize>,
}

pub struct Client {
    endpoint: Endpoint,
    _docs: Docs,
    _router: Router,
    loader: Rc<Loader>,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Endpoint bind error: {0}")]
    EndpointBindError(#[from] BindError),

    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {}

impl Client {
    pub async fn connect() -> Result<Self, ClientError> {
        let endpoint = Endpoint::builder().bind().await?;
        let blobs = MemStore::default();
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs = Docs::memory()
            .spawn(endpoint.clone(), (*blobs).clone(), gossip.clone())
            .await?;
        let builder = Router::builder(endpoint.clone());
        let router = builder
            .accept(iroh_blobs::ALPN, BlobsProtocol::new(&blobs, None))
            .accept(iroh_gossip::ALPN, gossip)
            .accept(iroh_docs::ALPN, docs.clone())
            .spawn();
        let doc = docs.create().await?;
        let loader = Rc::new(Loader {
            doc: Rc::new(doc),
            segment: None,
            root_start: None,
        });
        Ok(Self {
            endpoint,
            _docs: docs,
            _router: router,
            loader,
        })
    }

    pub fn to_reader(&self) -> Reader {
        let loader = self.loader.clone();
        Reader::new(loader)
    }

    pub fn to_endpoint_addr(&self) -> EndpointAddr {
        self.endpoint.addr()
    }

    pub fn transact(&mut self, changes: &[Change]) -> Result<(), TransactError> {
        let mut segment = Segment::new();
        let change_starts = segment.write_changes(changes);
        let mut trie = MemTrie::empty();
        for i in 0..change_starts.len() {
            match &changes[i] {
                Change::Deposit(entity, _attribute, _value) => {
                    trie = trie.insert(entity.0, change_starts[i] as u32);
                }
            }
        }
        let trie_start = segment.write_trie(&trie);
        let loader = Loader {
            doc: self.loader.doc.clone(),
            segment: Some(Rc::new(segment)),
            root_start: Some(trie_start),
        };
        self.loader = Rc::new(loader);
        Ok(())
    }
}

pub struct MemTrie {
    map_base: MemMapBase,
}

impl MemTrie {
    pub fn empty() -> Self {
        let map_base = MemMapBase::empty();
        Self { map_base }
    }
    pub fn insert(self, key: u32, value: u32) -> Self {
        let key = TrieKey::new(key);
        let map_base = self.map_base.insert_key_value(key, value);
        Self { map_base }
    }
}

pub struct MemMapBase {
    map: u32,
    base: Vec<MemSlot>,
}
impl MemMapBase {
    pub fn empty() -> Self {
        Self {
            map: 0,
            base: vec![],
        }
    }
    pub fn insert_key_value(self, key: TrieKey, value: u32) -> Self {
        let map_bit = key.to_map_bit();
        assert_eq!(map_bit & self.map, 0);
        let base_index = key.to_base_index(self.map);
        let mut base = self.base;
        base.insert(base_index, MemSlot::KeyValue(key.u32(), value));
        let map = self.map | map_bit;
        Self { map, base }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum MemSlot {
    #[default]
    Empty,
    KeyValue(u32, u32),
}

pub struct SegmentTrie {
    map_base: SegmentMapBase,
}
impl SegmentTrie {
    pub fn query_value(&self, key: TrieKey, segment: Rc<Segment>) -> Option<u32> {
        match self.map_base.load_slot(key, segment) {
            MemSlot::Empty => None,
            MemSlot::KeyValue(_key, value) => Some(value),
        }
    }
}

pub struct SegmentMapBase {
    map: u32,
    base_start: SegmentIndex,
}

impl SegmentMapBase {
    pub fn load_slot(&self, key: TrieKey, segment: Rc<Segment>) -> MemSlot {
        let base_index = key.to_base_index(self.map);
        let (left, right) = segment.read_slot(base_index, self.base_start);
        match left == key.u32() {
            true => MemSlot::KeyValue(left, right),
            false => MemSlot::Empty,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SegmentIndex(pub usize);
impl SegmentIndex {
    pub fn offset(&self, offset: usize) -> SegmentIndex {
        SegmentIndex(self.0 + offset)
    }
}

pub struct Segment {
    bytes: Vec<u8>,
}
impl Segment {
    pub fn new() -> Self {
        Self { bytes: vec![] }
    }

    pub fn write_trie(&mut self, trie: &MemTrie) -> usize {
        let map_base_start = self.write_map_base(&trie.map_base);
        map_base_start
    }

    pub fn read_trie(&self, map_base_start: SegmentIndex) -> SegmentTrie {
        let map_base = self.read_map_base(map_base_start);
        SegmentTrie { map_base }
    }

    pub fn write_map_base(&mut self, map_base: &MemMapBase) -> usize {
        let base_start = self.write_base(&map_base.base);
        let start = self.bytes.len();
        self.write_u32(map_base.map);
        self.write_u32(base_start as u32);
        start
    }

    pub fn read_map_base(&self, start: SegmentIndex) -> SegmentMapBase {
        let (map, base_part_start) = self.read_u32(start);
        let (base_start, _) = self.read_u32(base_part_start);
        SegmentMapBase {
            map,
            base_start: SegmentIndex(base_start as usize),
        }
    }

    pub fn write_base(&mut self, base: &[MemSlot]) -> usize {
        let start = self.bytes.len();
        for i in 0..base.len() {
            match &base[i] {
                MemSlot::Empty => panic!("Empty slot should never be written"),
                MemSlot::KeyValue(key, value) => {
                    self.write_u32(*key);
                    self.write_u32(*value);
                }
            }
        }
        start
    }

    pub fn read_slot(&self, base_index: usize, base_start: SegmentIndex) -> (u32, u32) {
        let slot_start = base_start.offset(base_index * 8);
        let (left, slot_part_start) = self.read_u32(slot_start);
        let (right, _) = self.read_u32(slot_part_start);
        (left, right)
    }

    pub fn write_changes(&mut self, changes: &[Change]) -> Vec<usize> {
        let count = changes.len() as u16;
        self.write_u16(count);
        let mut starts = vec![];
        for change in changes {
            starts.push(self.bytes.len());
            let bytes = change.to_be_bytes();
            self.bytes.extend_from_slice(&bytes);
        }
        starts
    }

    pub fn write_u32(&mut self, value: u32) {
        let bytes = value.to_be_bytes() as [u8; 4];
        self.bytes.extend_from_slice(&bytes);
    }

    pub fn read_u32(&self, start: SegmentIndex) -> (u32, SegmentIndex) {
        let end = start.0 + 4;
        let bytes = &self.bytes[start.0..end];
        let bytes = u32::from_be_bytes(bytes.try_into().expect("Invalid u32 bytes"));
        (bytes, SegmentIndex(end))
    }

    pub fn write_u16(&mut self, value: u16) {
        let bytes = value.to_be_bytes() as [u8; 2];
        self.bytes.extend_from_slice(&bytes);
    }

    pub fn read_u16(&self, start: SegmentIndex) -> (u16, SegmentIndex) {
        let end = start.0 + 2;
        let bytes = &self.bytes[start.0..end];
        let bytes = u16::from_be_bytes(bytes.try_into().expect("Invalid u16 bytes"));
        (bytes, SegmentIndex(end))
    }

    pub fn read_u8(&self, start: SegmentIndex) -> (u8, SegmentIndex) {
        let end = start.0 + 1;
        let bytes = &self.bytes[start.0..end];
        let bytes = bytes[0];
        (bytes, SegmentIndex(end))
    }
}
