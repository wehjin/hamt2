use crate::hamt::segment::{Segment, SegmentIndex};
use crate::hamt::value::Value;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Txid(usize);

impl Txid {
    pub const FLOOR: Txid = Txid(0);
}

pub struct Attr(pub u32);

pub struct Ent(pub u32);

pub enum Change {
    Deposit(Ent, Attr, Value),
}

impl Change {
    const DEPOSIT: u8 = 1;
    pub fn to_be_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![Self::DEPOSIT];
        match self {
            Change::Deposit(entity, attribute, value) => {
                bytes.extend_from_slice(&entity.0.to_be_bytes());
                bytes.extend_from_slice(&attribute.0.to_be_bytes());
                bytes.extend_from_slice(&value.to_be_bytes());
            }
        }
        bytes
    }

    pub fn from_be_bytes(start: SegmentIndex, segment: &Segment) -> (Self, SegmentIndex) {
        let (change_type, start) = segment.read_u8(start);
        match change_type {
            Self::DEPOSIT => {
                let (entity, start) = segment.read_u32(start);
                let (attribute, start) = segment.read_u32(start);
                let (value, start) = Value::from_be_bytes(start, &segment);
                let change = Change::Deposit(Ent(entity), Attr(attribute), value);
                (change, start)
            }
            _ => panic!("Invalid change type"),
        }
    }
}
