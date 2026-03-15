use crate::hamt::segment::{Segment, SegmentIndex};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Txid(usize);

impl Txid {
    pub const FLOOR: Txid = Txid(0);
}

pub struct Entity(pub u32);

pub struct Attribute(pub u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Value {
    UInt(u32),
}

impl Value {
    const UINT: u8 = 1;
    pub fn to_be_bytes(&self) -> Vec<u8> {
        match self {
            Value::UInt(value) => {
                let mut bytes = vec![Self::UINT];
                bytes.extend_from_slice(&value.to_be_bytes());
                bytes
            }
        }
    }
    pub fn from_be_bytes(start: SegmentIndex, segment: &Segment) -> (Self, SegmentIndex) {
        let (value_type, start) = segment.read_u8(start);
        match value_type {
            Self::UINT => {
                let (value, start) = segment.read_u32(start);
                (Value::UInt(value), start)
            }
            _ => panic!("Invalid value type"),
        }
    }
}

pub enum Change {
    Deposit(Entity, Attribute, Value),
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
                let change = Change::Deposit(Entity(entity), Attribute(attribute), value);
                (change, start)
            }
            _ => panic!("Invalid change type"),
        }
    }
}
